use wgpu::{
    Backends, Color, CommandEncoderDescriptor, Device, DeviceDescriptor, Features, Instance,
    InstanceDescriptor, Limits, LoadOp, Operations, PowerPreference, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, Surface,
    SurfaceConfiguration, SurfaceError, TextureUsages, TextureViewDescriptor,
};
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

pub struct State {
    surface: Surface,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    size: PhysicalSize<u32>,
    window: Window,
}

impl State {
    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();

        // Used to create adapters and surfaces
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        // This is what we draw to. Surface needs to live as long as the window.
        // Since both are managed by Self, this is okay.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        // Handle to the graphics card
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    features: Features::empty(),
                    limits: Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0], // Vsync
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn size(&self) -> &PhysicalSize<u32> {
        &self.size
    }

    /// Returns whether the input was handled by State
    pub fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    pub fn update(&mut self) {}

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        let output = self.surface.get_current_texture()?;
        // Controls how the render code interacts with the texture
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render encoder"),
            });

        let render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}
