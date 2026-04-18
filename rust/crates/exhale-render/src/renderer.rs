use std::sync::Arc;

use anyhow::{Context, Result};
use bytemuck::cast_slice;
use exhale_core::{controller::BreathingState, settings::Settings};
use log::{debug, warn};
use wgpu::util::DeviceExt;

use crate::{gpu_context::GpuContext, uniforms::OverlayUniforms};

const SHADER_SRC: &str = include_str!("shaders/overlay.wgsl");

// ─── Public API ───────────────────────────────────────────────────────────────

/// wgpu renderer for a single breathing overlay window.
///
/// Each screen gets its own `OverlayRenderer`; all share the same
/// [`GpuContext`] (device + queue).
pub struct OverlayRenderer {
    gpu:            Arc<GpuContext>,
    surface:        wgpu::Surface<'static>,
    config:         wgpu::SurfaceConfiguration,
    pipeline:       wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group:     wgpu::BindGroup,
}

impl OverlayRenderer {
    /// Create a renderer for an existing surface, sharing the GPU context.
    pub fn new(
        gpu:    Arc<GpuContext>,
        surface: wgpu::Surface<'static>,
        width:  u32,
        height: u32,
    ) -> Result<Self> {
        let surface_caps   = surface.get_capabilities(&{
            // Re-request adapter compatible with this surface.
            // In practice the same adapter supports all monitors on one GPU.
            pollster::block_on(gpu.instance.request_adapter(
                &wgpu::RequestAdapterOptions {
                    power_preference:       wgpu::PowerPreference::LowPower,
                    compatible_surface:     Some(&surface),
                    force_fallback_adapter: false,
                }
            )).context("adapter for surface")?
        });

        let surface_format = prefer_format(&surface_caps);
        let alpha_mode     = pick_alpha_mode(&surface_caps);

        let config = wgpu::SurfaceConfiguration {
            usage:                         wgpu::TextureUsages::RENDER_ATTACHMENT,
            format:                        surface_format,
            width:                         width.max(1),
            height:                        height.max(1),
            present_mode:                  wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode,
            view_formats:                  vec![],
        };
        surface.configure(&gpu.device, &config);

        let (pipeline, uniform_buffer, bind_group) =
            build_pipeline(&gpu.device, surface_format)?;

        Ok(Self { gpu, surface, config, pipeline, uniform_buffer, bind_group })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 { return; }
        self.config.width  = width;
        self.config.height = height;
        self.surface.configure(&self.gpu.device, &self.config);
        debug!("overlay resized to {width}×{height}");
    }

    pub fn render(
        &mut self,
        state:            &BreathingState,
        settings:         &Settings,
        max_circle_scale: f32,
    ) -> Result<()> {
        let output = match self.surface.get_current_texture() {
            Ok(t)  => t,
            Err(wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost) => {
                self.surface.configure(&self.gpu.device, &self.config);
                warn!("overlay surface lost — reconfigured");
                return Ok(());
            }
            Err(e) => return Err(e).context("overlay get_current_texture"),
        };

        let view = output.texture.create_view(&Default::default());

        let uniforms = OverlayUniforms::from_state(
            state, settings, self.config.width, self.config.height, max_circle_scale,
        );
        self.gpu.queue.write_buffer(&self.uniform_buffer, 0, cast_slice(&[uniforms]));

        let mut enc = self.gpu.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: Some("overlay-frame") }
        );
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label:                    Some("overlay-pass"),
                color_attachments:        &[Some(wgpu::RenderPassColorAttachment {
                    view:           &view,
                    resolve_target: None,
                    ops:            wgpu::Operations {
                        load:  wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes:         None,
                occlusion_query_set:      None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        self.gpu.queue.submit(std::iter::once(enc.finish()));
        output.present();
        Ok(())
    }

    pub fn width(&self)  -> u32 { self.config.width }
    pub fn height(&self) -> u32 { self.config.height }
    pub fn surface_format(&self) -> wgpu::TextureFormat { self.config.format }
}

// ─── Pipeline ─────────────────────────────────────────────────────────────────

pub(crate) fn build_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
) -> Result<(wgpu::RenderPipeline, wgpu::Buffer, wgpu::BindGroup)> {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label:  Some("overlay-shader"),
        source: wgpu::ShaderSource::Wgsl(SHADER_SRC.into()),
    });

    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label:    Some("overlay-uniforms"),
        contents: bytemuck::bytes_of(&OverlayUniforms::zeroed()),
        usage:    wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label:   Some("overlay-bgl"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding:    0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty:         wgpu::BindingType::Buffer {
                ty:                 wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size:   None,
            },
            count: None,
        }],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label:   Some("overlay-bg"),
        layout:  &bgl,
        entries: &[wgpu::BindGroupEntry {
            binding:  0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label:                Some("overlay-layout"),
        bind_group_layouts:   &[&bgl],
        push_constant_ranges: &[],
    });

    let premult = wgpu::BlendState {
        color: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation:  wgpu::BlendOperation::Add,
        },
        alpha: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation:  wgpu::BlendOperation::Add,
        },
    };

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label:    Some("overlay-pipeline"),
        layout:   Some(&layout),
        vertex:   wgpu::VertexState {
            module:              &shader,
            entry_point:         "vs_main",
            buffers:             &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module:              &shader,
            entry_point:         "fs_main",
            targets:             &[Some(wgpu::ColorTargetState {
                format:     format,
                blend:      Some(premult),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive:     wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, ..Default::default() },
        depth_stencil: None,
        multisample:   wgpu::MultisampleState::default(),
        multiview:     None,
        cache:         None,
    });

    Ok((pipeline, uniform_buffer, bind_group))
}

// ─── Surface helpers ──────────────────────────────────────────────────────────

/// Prefer non-sRGB (linear byte) formats so color values fed into the shader
/// land in the framebuffer 1:1 — matching Swift's `MTKView.colorPixelFormat =
/// .bgra8Unorm`.  With a `*UnormSrgb` surface wgpu would gamma-encode the
/// shader output (linear → sRGB) and mid-tone blends would render noticeably
/// brighter than the Swift reference.
fn prefer_format(caps: &wgpu::SurfaceCapabilities) -> wgpu::TextureFormat {
    for &f in &[
        wgpu::TextureFormat::Bgra8Unorm,
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureFormat::Bgra8UnormSrgb,
        wgpu::TextureFormat::Rgba8UnormSrgb,
    ] {
        if caps.formats.contains(&f) { return f; }
    }
    caps.formats[0]
}

fn pick_alpha_mode(caps: &wgpu::SurfaceCapabilities) -> wgpu::CompositeAlphaMode {
    use wgpu::CompositeAlphaMode as M;
    for &m in &[M::PreMultiplied, M::PostMultiplied, M::Inherit] {
        if caps.alpha_modes.contains(&m) { return m; }
    }
    caps.alpha_modes[0]
}

// ─── Zeroed helper ────────────────────────────────────────────────────────────

impl OverlayUniforms {
    pub(crate) fn zeroed() -> Self { bytemuck::Zeroable::zeroed() }
}
