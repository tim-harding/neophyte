use crate::{
    text::{atlas::FontAtlas, font::Font},
    ui::grid::Grid,
};
use std::sync::{mpsc::Receiver, Arc, Mutex};
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
};
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: Mutex<wgpu::SurfaceConfiguration>,
    size: Mutex<PhysicalSize<u32>>,
    window: Arc<Window>,
    render_pipeline: wgpu::RenderPipeline,
    texture_bind_group: wgpu::BindGroup,
    font: Font,
    atlas: FontAtlas,
    vertex_buffer: Mutex<wgpu::Buffer>,
    index_buffer: Mutex<wgpu::Buffer>,
    index_count: Mutex<u32>,
}

impl State {
    pub async fn new(window: Arc<Window>, rx: Receiver<Grid>) -> Arc<Self> {
        let size = window.inner_size();

        // Used to create adapters and surfaces
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        // This is what we draw to. Surface needs to live as long as the window.
        // Since both are managed by Self, this is okay.
        let surface = unsafe { instance.create_surface(window.as_ref()) }.unwrap();

        // Handle to the graphics card
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
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
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0], // Vsync
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let font = Font::from_file("/usr/share/fonts/OTF/CascadiaCode-Regular.otf", 0).unwrap();
        let atlas = FontAtlas::from_font(font.as_ref(), 24.0);
        let dim = atlas.size() as u32;
        let texture_size = wgpu::Extent3d {
            width: dim,
            height: dim,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("Texture"),
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTextureBase {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            atlas.data(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(dim),
                rows_per_image: Some(dim),
            },
            texture_size,
        );

        let texture_view = texture.create_view(&Default::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Texture sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: bytemuck::cast_slice(&[GlyphVertex::default(); 0]),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index buffer"),
            contents: bytemuck::cast_slice(&[GlyphVertex::default(); 0]),
            usage: wgpu::BufferUsages::INDEX,
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture bind group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[GlyphVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            // How to interpret vertices when converting to triangles
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,                         // Which samples to activate (all of them)
                alpha_to_coverage_enabled: false, // aa-related
            },
            multiview: None, // Involved in array textures
        });

        let window_handle = window.clone();
        let this = Arc::new(Self {
            window,
            surface,
            device,
            queue,
            config: Mutex::new(config),
            size: Mutex::new(size),
            render_pipeline,
            texture_bind_group,
            atlas,
            font,
            index_buffer: Mutex::new(index_buffer),
            vertex_buffer: Mutex::new(vertex_buffer),
            index_count: Mutex::new(0),
        });

        {
            let this = this.clone();
            std::thread::spawn(move || {
                while let Ok(grid) = rx.recv() {
                    this.update_text(grid);
                    window_handle.request_redraw();
                }
            });
        }

        this
    }

    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        // Controls how the render code interacts with the texture
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render encoder"),
            });

        let vertex_buffer = self.vertex_buffer.lock().unwrap();
        let index_buffer = self.index_buffer.lock().unwrap();
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None, // No multisampling
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
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
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..*self.index_count.lock().unwrap(), 0, 0..1);
        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    pub fn resize(&self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            *self.size.lock().unwrap() = new_size;
            let mut lock = self.config.lock().unwrap();
            lock.width = new_size.width;
            lock.height = new_size.height;
            self.surface.configure(&self.device, &*lock);
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        *self.size.lock().unwrap()
    }

    pub fn update(&self) {}

    // TODO: Preallocate and reuse buffer
    fn update_text(&self, grid: Grid) {
        let size = *self.size.lock().unwrap();
        let clip_x = |n| (n / size.width as f32) * 2.0 - 1.0;
        let clip_y = |n| (n / size.height as f32) * -2.0 + 1.0;
        let font = self.font.as_ref();
        let charmap = font.charmap();
        let mut vertices = vec![];
        let mut indices = vec![];
        let metrics = font.metrics(&[]).linear_scale(24.0);
        let advance = (metrics.average_width / metrics.units_per_em as f32).round();
        for (row_i, line) in grid.cells.rows().enumerate() {
            let mut offset_x = 0.0;
            let offset_y = row_i as f32 * 24.0;
            for c in line {
                let id = charmap.map(c);
                let glyph = match self.atlas.get(id) {
                    Some(glyph) => glyph,
                    None => {
                        offset_x += advance;
                        continue;
                    }
                };

                let left = offset_x + glyph.placement.left as f32;
                let right = left + glyph.placement.width as f32;
                let top = offset_y + -glyph.placement.top as f32 + 24.0;
                let bottom = top + glyph.placement.height as f32;

                let left = clip_x(left);
                let right = clip_x(right);
                let top = clip_y(top);
                let bottom = clip_y(bottom);

                let u_min = glyph.origin.x as f32 / self.atlas.size() as f32;
                let u_max = (glyph.origin.x as f32 + glyph.placement.width as f32)
                    / self.atlas.size() as f32;
                let v_min = glyph.origin.y as f32 / self.atlas.size() as f32;
                let v_max = (glyph.origin.y as f32 + glyph.placement.height as f32)
                    / self.atlas.size() as f32;

                let base = vertices.len() as u16;
                vertices.extend_from_slice(&[
                    GlyphVertex {
                        p: [left, top],
                        t: [u_min, v_min],
                    },
                    GlyphVertex {
                        p: [right, top],
                        t: [u_max, v_min],
                    },
                    GlyphVertex {
                        p: [left, bottom],
                        t: [u_min, v_max],
                    },
                    GlyphVertex {
                        p: [right, bottom],
                        t: [u_max, v_max],
                    },
                ]);
                indices.extend_from_slice(&[
                    base + 2,
                    base + 1,
                    base,
                    base + 1,
                    base + 2,
                    base + 3,
                ]);
                offset_x += advance;
            }
        }

        *self.vertex_buffer.lock().unwrap() =
            self.device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Vertex buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        *self.index_buffer.lock().unwrap() =
            self.device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Index buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        *self.index_count.lock().unwrap() = indices.len() as u32;
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlyphVertex {
    p: [f32; 2],
    t: [f32; 2],
}

impl GlyphVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}
