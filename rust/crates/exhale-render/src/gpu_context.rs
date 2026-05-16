use std::sync::Arc;

use anyhow::{Context, Result};
use log::info;

/// Shared wgpu instance + adapter + a default device/queue.
///
/// The instance + adapter are shared across the whole app — they're stateless
/// "this is the GPU you're using" handles.  The default device + queue stay
/// for headless / benchmarking code that doesn't need its own isolated
/// command queue.
///
/// **Per-window renderers should call [`GpuContext::new_render_device`] to
/// mint their own (`Device`, `Queue`) pair** instead of using the default
/// one.  Each `wgpu::Device` maps to a separate `ID3D12CommandQueue` (or
/// Metal/Vulkan queue) under the hood; modern GPUs schedule those
/// concurrently, but commands submitted to the *same* queue serialize.
/// Sharing the default device across the overlay and settings windows
/// meant every hover-driven settings repaint blocked the overlay's next
/// present on the shared queue — visible as breath-animation stutter on
/// Windows.  Per-window devices remove that contention entirely.
pub struct GpuContext {
    /// Used to create additional surfaces for new windows.
    pub instance: wgpu::Instance,
    /// The physical adapter every device is requested from.  Exposed so
    /// per-window renderers can mint their own devices via
    /// `new_render_device()`, and so each window can query surface
    /// capabilities for its own surface.
    pub adapter:  Arc<wgpu::Adapter>,
    /// Default device — convenient for headless / one-off rendering.
    /// Per-window renderers should NOT use this directly; call
    /// `new_render_device()` instead so each window gets its own queue.
    pub device:   Arc<wgpu::Device>,
    /// Default queue (see `device`).
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

    /// Mint a fresh (`Device`, `Queue`) pair from the shared adapter.
    /// Each window's renderer calls this once at construction and keeps
    /// the pair for all its GPU work, so its command submissions don't
    /// serialize behind other windows' on a single shared queue.
    pub fn new_render_device(&self) -> Result<(Arc<wgpu::Device>, Arc<wgpu::Queue>)> {
        pollster::block_on(async {
            let (device, queue) = self.adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label:             Some("exhale-window-device"),
                        required_features: wgpu::Features::empty(),
                        required_limits:   self.adapter.limits(),
                        memory_hints:      wgpu::MemoryHints::default(),
                    },
                    None,
                )
                .await
                .context("request_device (per-window)")?;
            Ok((Arc::new(device), Arc::new(queue)))
        })
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
                    label:             Some("exhale-device-default"),
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
            adapter: Arc::new(adapter),
            device:  Arc::new(device),
            queue:   Arc::new(queue),
        }))
    }
}
