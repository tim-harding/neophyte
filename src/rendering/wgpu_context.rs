use crate::util::vec2::PixelVec;
use std::sync::Arc;
use winit::window::Window;

pub struct WgpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
}

impl WgpuContext {
    pub fn new(window: Arc<Window>, transparent: bool) -> Self {
        let surface_size: PixelVec<u32> = window.inner_size().into();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            dx12_shader_compiler: Default::default(),
            flags: wgpu::InstanceFlags::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });

        let surface = instance
            .create_surface(window)
            .expect("Failed to create graphics surface");

        let adapter = pollster::block_on(async { instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await })
            .expect("Failed to get a graphics adapter. Make sure you are using either Vulkan, Metal, or DX12.");

        let (device, queue) = pollster::block_on(async {
            adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: None,
                        required_features: wgpu::Features::PUSH_CONSTANTS,
                        required_limits: adapter.limits(),
                        memory_hints: wgpu::MemoryHints::Performance,
                    },
                    None,
                )
                .await
        })
        .expect("Failed to get a graphics device");

        let surface_caps = surface.get_capabilities(&adapter);

        let alpha_mode = if transparent
            && surface_caps
                .alpha_modes
                .contains(&wgpu::CompositeAlphaMode::PreMultiplied)
        {
            wgpu::CompositeAlphaMode::PreMultiplied
        } else {
            surface_caps.alpha_modes[0]
        };

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_caps
                .formats
                .iter()
                .copied()
                .find(|f| f.is_srgb())
                .unwrap_or(surface_caps.formats[0]),
            width: surface_size.0.x,
            height: surface_size.0.y,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        Self {
            device,
            queue,
            surface,
            surface_config,
        }
    }

    pub fn resize(&mut self, new_size: PixelVec<u32>) {
        self.surface_config.width = new_size.0.x;
        self.surface_config.height = new_size.0.y;
        self.surface.configure(&self.device, &self.surface_config);
    }

    pub fn surface_size(&self) -> PixelVec<u32> {
        PixelVec::new(self.surface_config.width, self.surface_config.height)
    }
}
