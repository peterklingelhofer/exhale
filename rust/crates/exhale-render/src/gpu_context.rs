use std::sync::Arc;

use anyhow::{Context, Result};
use log::info;

/// Shared wgpu device and queue — created once, cloned cheaply via Arc.
///
/// Passed to every renderer (overlay windows + settings window) so all GPU
/// work goes through the same device/queue pair.
pub struct GpuContext {
    /// Used to create additional surfaces for new windows.
    pub instance: wgpu::Instance,
    pub device:   Arc<wgpu::Device>,
    pub queue:    Arc<wgpu::Queue>,
}

impl GpuContext {
    /// Initialise a shared GPU context compatible with the given surface.
    /// Call once on startup; clone the returned `Arc` for each renderer.
    pub fn new_for_surface(
        instance: wgpu::Instance,
        surface:  &wgpu::Surface<'_>,
    ) -> Result<Arc<Self>> {
        pollster::block_on(Self::new_async(instance, Some(surface)))
    }

    /// Initialise a shared GPU context with no surface (for headless rendering,
    /// e.g. CPU benchmarks that render to an offscreen texture).
    pub fn new_headless() -> Result<Arc<Self>> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        pollster::block_on(Self::new_async(instance, None))
    }

    async fn new_async(
        instance: wgpu::Instance,
        surface:  Option<&wgpu::Surface<'_>>,
    ) -> Result<Arc<Self>> {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference:       wgpu::PowerPreference::LowPower,
                compatible_surface:     surface,
                force_fallback_adapter: false,
            })
            .await
            .context("no suitable GPU adapter")?;

        info!(
            "GPU: {} ({:?})",
            adapter.get_info().name,
            adapter.get_info().backend
        );

        // `downlevel_defaults` caps `max_texture_dimension_2d` at 2048, which
        // rejects surface configuration on any ≥4K display.  Request the
        // adapter's actual limits so the overlay can span real hardware
        // resolutions on modern GPUs (Metal/DX12/Vulkan all report ≥8192).
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label:             Some("exhale-device"),
                    required_features: wgpu::Features::empty(),
                    required_limits:   adapter.limits(),
                    memory_hints:      wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .context("request_device")?;

        Ok(Arc::new(Self {
            instance,
            device: Arc::new(device),
            queue:  Arc::new(queue),
        }))
    }
}
