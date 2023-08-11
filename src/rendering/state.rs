use super::texture::Texture;
use crate::{
    event::hl_attr_define::Rgb,
    text::{cache::FontCache, font::Font},
    ui::Ui,
    util::vec2::Vec2,
};
use bytemuck::{cast_slice, Pod, Zeroable};
use std::{
    num::NonZeroU32,
    sync::{mpsc::Receiver, Arc, Mutex},
};
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
};
use winit::{dpi::PhysicalSize, window::Window};

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: Mutex<wgpu::SurfaceConfiguration>,
    size: Mutex<PhysicalSize<u32>>,
    window: Arc<Window>,
    font_cache: FontCache,
    font: Font,
    font_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: Mutex<wgpu::Buffer>,
    clear_color: Mutex<wgpu::Color>,
    grid_render: Mutex<Option<GridRender>>,
    grid_bind_group_layout: wgpu::BindGroupLayout,
    vertex_count: Mutex<u32>,
}

impl State {
    pub async fn new(window: Arc<Window>, rx: Receiver<Ui>, font: Font) -> Arc<Self> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(window.as_ref()) }.unwrap();

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
                    features: wgpu::Features::TEXTURE_BINDING_ARRAY | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING | wgpu::Features::UNIFORM_BUFFER_AND_STORAGE_TEXTURE_ARRAY_NON_UNIFORM_INDEXING | wgpu::Features::PUSH_CONSTANTS,
                    limits: adapter.limits(),
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

        let font_cache = FontCache::from_font(font.as_ref(), 24.0);
        let textures: Vec<_> = font_cache
            .data
            .iter()
            .zip(font_cache.info.iter())
            .map(|(data, info)| Texture::new(&device, &queue, data, info.size))
            .collect();

        let views: Vec<_> = textures.iter().map(|texture| &texture.view).collect();

        let font_info_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Font info buffer"),
            contents: cast_slice(font_cache.info.as_slice()),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let grid_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Grid bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let tex_count = Some(NonZeroU32::new(textures.len() as u32).unwrap());
        let font_bind_group_layout =
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
                        count: tex_count,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: tex_count,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let font_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Font bind group"),
            layout: &font_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(views.as_slice()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &font_info_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&font_bind_group_layout, &grid_bind_group_layout],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX,
                range: 0..GridInfo::SIZE as u32,
            }],
        });

        let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: cast_slice(&[GlyphVertex::default(); 0]),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render pipeline"),
            layout: Some(&pipeline_layout),
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
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let window_handle = window.clone();
        let this = Arc::new(Self {
            font_cache,
            vertex_count: Mutex::new(0),
            render_pipeline,
            font_bind_group,
            grid_bind_group_layout,
            window,
            surface,
            device,
            queue,
            config: Mutex::new(config),
            size: Mutex::new(size),
            font,
            grid_render: Mutex::new(None),
            vertex_buffer: Mutex::new(vertex_buffer),
            clear_color: Mutex::new(wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }),
        });

        {
            let this = this.clone();
            std::thread::spawn(move || {
                while let Ok(ui) = rx.recv() {
                    this.update_text(ui);
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

        let grid_render = self.grid_render.lock().unwrap();
        let vertex_buffer = self.vertex_buffer.lock().unwrap();
        let clear_color = *self.clear_color.lock().unwrap();
        let vertex_count = *self.vertex_count.lock().unwrap();
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None, // No multisampling
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        if let Some(grid_render) = grid_render.as_ref() {
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_bind_group(0, &self.font_bind_group, &[]);
            render_pass.set_bind_group(1, &grid_render.bind_group, &[]);
            render_pass.set_push_constants(
                wgpu::ShaderStages::VERTEX,
                0,
                cast_slice(&[grid_render.info]),
            );
            render_pass.draw(0..vertex_count, 0..1);
        }
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

    fn update_text(&self, ui: Ui) {
        let grid = ui.composite();
        // TODO: Should only rebuild the pipeline as the result of a resize

        let size = *self.size.lock().unwrap();
        let clip_x = |n| (n / size.width as f32) * 2.0 - 1.0;
        let clip_y = |n| (n / size.height as f32) * -2.0 + 1.0;
        let font = self.font.as_ref();
        let charmap = font.charmap();
        let mut vertices = vec![];
        let metrics = font.metrics(&[]).linear_scale(24.0);
        let advance = (metrics.average_width / metrics.units_per_em as f32).round();

        let fg_default = ui.default_colors.rgb_fg.unwrap_or(Rgb::new(255, 255, 255));
        let bg_default = ui.default_colors.rgb_bg.unwrap_or(Rgb::new(0, 0, 0));
        *self.clear_color.lock().unwrap() = wgpu::Color {
            r: (bg_default.r() as f64 / 255.0).powf(2.2),
            g: (bg_default.g() as f64 / 255.0).powf(2.2),
            b: (bg_default.b() as f64 / 255.0).powf(2.2),
            a: 1.0,
        };
        let mut glyph_info = vec![];

        for (row_i, (cell_line, hl_line)) in
            grid.cells.rows().zip(grid.highlights.rows()).enumerate()
        {
            let mut offset_x = 0.0;
            let offset_y = row_i as f32 * 24.0;
            for (c, hl) in cell_line.zip(hl_line) {
                let (fg, _bg) = if let Some(hl) = ui.highlights.get(&hl) {
                    (
                        hl.rgb_attr.foreground.unwrap_or(fg_default),
                        hl.rgb_attr.background.unwrap_or(bg_default),
                    )
                } else {
                    (Rgb::WHITE, Rgb::BLACK)
                };

                let mul = [
                    (fg.r() as f32 / 255.0).powf(2.2),
                    (fg.g() as f32 / 255.0).powf(2.2),
                    (fg.b() as f32 / 255.0).powf(2.2),
                    1.0,
                ];

                let id = charmap.map(c);
                let glyph_index = match self.font_cache.lut.get(&id) {
                    Some(glyph) => glyph,
                    None => {
                        vertices.extend_from_slice(&[GlyphVertex::default(); 6]);
                        glyph_info.push(GlyphInfo {
                            color: mul,
                            texture_index: [0, 0, 0, 0],
                        });
                        offset_x += advance;
                        continue;
                    }
                };
                let info = &self.font_cache.info[*glyph_index];

                let left = offset_x + info.placement_offset.x as f32;
                let right = left + info.size.x as f32;
                let top = offset_y - info.placement_offset.y as f32 + 24.0;
                let bottom = top + info.size.y as f32;

                let left = clip_x(left);
                let right = clip_x(right);
                let top = clip_y(top);
                let bottom = clip_y(bottom);

                glyph_info.push(GlyphInfo {
                    color: mul,
                    texture_index: [*glyph_index as u32, 0, 0, 0],
                });

                vertices.extend_from_slice(&[
                    GlyphVertex {
                        pos: [left, bottom],
                    },
                    GlyphVertex { pos: [right, top] },
                    GlyphVertex { pos: [left, top] },
                    GlyphVertex { pos: [right, top] },
                    GlyphVertex {
                        pos: [left, bottom],
                    },
                    GlyphVertex {
                        pos: [right, bottom],
                    },
                ]);
                offset_x += advance;
            }
        }

        let grid_info = GridInfo {
            surface_size: Vec2::new(size.width, size.height),
            grid_size: grid.size().into(),
            glyph_size: Vec2::new(advance as u32, 24),
        };

        let pipeline = GridRender::new(
            &self.device,
            &self.grid_bind_group_layout,
            glyph_info,
            grid_info,
        );

        *self.vertex_buffer.lock().unwrap() =
            self.device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Vertex buffer"),
                contents: cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        *self.grid_render.lock().unwrap() = Some(pipeline);
        *self.vertex_count.lock().unwrap() = vertices.len() as u32;
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct GlyphVertex {
    pos: [f32; 2],
}

impl GlyphVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x2];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct GridRender {
    pub bind_group: wgpu::BindGroup,
    pub info_buffer: wgpu::Buffer,
    pub info: GridInfo,
}

impl GridRender {
    pub fn new(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        data: Vec<GlyphInfo>,
        info: GridInfo,
    ) -> Self {
        let info_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("info buffer"),
            usage: wgpu::BufferUsages::STORAGE,
            contents: cast_slice(&data),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("glyph info bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &info_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        Self {
            bind_group,
            info_buffer,
            info,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct GlyphInfo {
    color: [f32; 4],
    // TODO: Do SOA layout so alignment doesn't take up a bunch of excess space
    texture_index: [u32; 4],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct GridInfo {
    // The dimensions of the texture we're drawing to
    surface_size: Vec2<u32>,
    // The dimensions of the Neovim grid
    grid_size: Vec2<u32>,
    // The dimensions of a single glyph. (font_height, advance)
    glyph_size: Vec2<u32>,
}

impl GridInfo {
    pub const SIZE: usize = std::mem::size_of::<Self>();
}
