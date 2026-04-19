use std::sync::Arc;

use anyhow::Result;
use bytemuck::cast_slice;
use exhale_core::{controller::BreathingState, settings::Settings};

use crate::{gpu_context::GpuContext, renderer::build_pipeline, uniforms::OverlayUniforms};

/// wgpu renderer that draws into an offscreen texture (no `wgpu::Surface`).
///
/// Used by the CPU benchmark to measure render cost without the presentation
/// path (swapchain acquire + present + compositor work).  The pipeline mirrors
/// [`crate::OverlayRenderer`] exactly so the work-per-frame is comparable.
pub struct HeadlessRenderer {
    gpu:            Arc<GpuContext>,
    /// Owned to keep the render target alive — `view` borrows it internally.
    #[allow(dead_code)]
    texture:        wgpu::Texture,
    view:           wgpu::TextureView,
    format:         wgpu::TextureFormat,
    width:          u32,
    height:         u32,
    pipeline:       wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group:     wgpu::BindGroup,
}

impl HeadlessRenderer {
    pub fn new(gpu: Arc<GpuContext>, width: u32, height: u32) -> Result<Self> {
        // Match the format used by the real overlay on macOS/Windows.
        let format = wgpu::TextureFormat::Bgra8Unorm;

        let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label:           Some("headless-overlay-target"),
            size:            wgpu::Extent3d { width: width.max(1), height: height.max(1), depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count:    1,
            dimension:       wgpu::TextureDimension::D2,
            format,
            usage:           wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats:    &[],
        });
        let view = texture.create_view(&Default::default());
        let (pipeline, uniform_buffer, bind_group) = build_pipeline(&gpu.device, format)?;

        Ok(Self { gpu, texture, view, format, width, height, pipeline, uniform_buffer, bind_group })
    }

    pub fn render(
        &mut self,
        state:            &BreathingState,
        settings:         &Settings,
        max_circle_scale: f32,
    ) -> Result<()> {
        let uniforms = OverlayUniforms::from_state(
            state, settings, self.width, self.height, max_circle_scale,
        );
        self.gpu.queue.write_buffer(&self.uniform_buffer, 0, cast_slice(&[uniforms]));

        let mut enc = self.gpu.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: Some("headless-frame") }
        );
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label:                    Some("headless-pass"),
                color_attachments:        &[Some(wgpu::RenderPassColorAttachment {
                    view:           &self.view,
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
        // No present — the texture is the render target.
        Ok(())
    }

    pub fn width(&self)  -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }
    pub fn format(&self) -> wgpu::TextureFormat { self.format }
}
