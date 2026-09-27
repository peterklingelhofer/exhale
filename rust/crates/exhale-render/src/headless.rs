use std::sync::Arc;

use anyhow::{Context, Result};
use bytemuck::cast_slice;
use exhale_core::{controller::BreathingState, settings::Settings};

use crate::{gpu_context::GpuContext, renderer::build_pipeline, uniforms::OverlayUniforms};

/// wgpu renderer that draws into an offscreen texture (no `wgpu::Surface`)
///
/// Used by the CPU benchmark to measure render cost without the presentation
/// path (swapchain acquire + present + compositor work).  The pipeline mirrors
/// [`crate::OverlayRenderer`] so the work-per-frame is comparable
pub struct HeadlessRenderer {
    /// Own isolated device + queue, minted from the shared `GpuContext`'s
    /// adapter. See `GpuContext::new_render_device` for the rationale
    device:         Arc<wgpu::Device>,
    queue:          Arc<wgpu::Queue>,
    /// Owned to keep the render target alive: `view` borrows it internally
    #[allow(dead_code)]
    texture:        wgpu::Texture,
    view:           wgpu::TextureView,
    width:          u32,
    height:         u32,
    pipeline:       wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group:     wgpu::BindGroup,
}

impl HeadlessRenderer {
    pub fn new(gpu: Arc<GpuContext>, width: u32, height: u32) -> Result<Self> {
        // Match the format used by the real overlay on macOS/Windows
        let format = wgpu::TextureFormat::Bgra8Unorm;

        let (device, queue) = gpu.new_render_device().context("headless per-window device")?;

        let texture = device.create_texture(&wgpu::TextureDescriptor {
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
        let (pipeline, uniform_buffer, bind_group) = build_pipeline(&device, format)?;

        Ok(Self { device, queue, texture, view, width, height, pipeline, uniform_buffer, bind_group })
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
        self.queue.write_buffer(&self.uniform_buffer, 0, cast_slice(&[uniforms]));

        let mut enc = self.device.create_command_encoder(
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
        self.queue.submit(std::iter::once(enc.finish()));
        // No present: the texture is the render target
        Ok(())
    }
}
