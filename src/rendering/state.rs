use super::{
    state_read::StateRead, state_surface_config::StateSurfaceConfig, state_write::StateWrite,
    texture::Texture,
};
use crate::{
    text::{cache::FontCache, font::Font},
    ui::Ui,
    util::vec2::Vec2,
};
use bytemuck::{cast_slice, Pod, Zeroable};
use std::{
    num::NonZeroU32,
    sync::{Arc, RwLock},
};
use wgpu::{include_wgsl, util::DeviceExt};
use winit::{dpi::PhysicalSize, window::Window};

pub struct State {
    surface_config: StateSurfaceConfig,
    constant: Arc<StateConstant>,
    read: Arc<RwLock<Option<StateRead>>>,
    write: StateWrite,
}

impl State {
    pub async fn new(window: Arc<Window>, font: Font) -> Self {
        let mut font_cache = FontCache::new();
        let font_ref = font.as_ref();
        font_ref.charmap().enumerate(|_c, id| {
            font_cache.get(font_ref, 24.0, id);
        });

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
                    features:
                        wgpu::Features::TEXTURE_BINDING_ARRAY |
                        wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING |
                        wgpu::Features::UNIFORM_BUFFER_AND_STORAGE_TEXTURE_ARRAY_NON_UNIFORM_INDEXING |
                        wgpu::Features::PUSH_CONSTANTS,
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

        let textures: Vec<_> = font_cache
            .data
            .iter()
            .zip(font_cache.info.iter())
            .map(|(data, info)| Texture::new(&device, &queue, data, info.size))
            .collect();

        let views: Vec<_> = textures.iter().map(|texture| &texture.view).collect();

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
                        count: None,
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

        let highlights_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Highlights bind group layout"),
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

        let cell_fill_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Cell fill pipeline layout"),
                bind_group_layouts: &[&highlights_bind_group_layout, &grid_bind_group_layout],
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::VERTEX,
                    range: 0..GridInfo::SIZE as u32,
                }],
            });

        let glyph_shader = device.create_shader_module(include_wgsl!("glyph.wgsl"));
        let cell_fill_shader = device.create_shader_module(include_wgsl!("cell_fill.wgsl"));

        let cell_fill_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Cell fill render pipeline"),
                layout: Some(&cell_fill_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &cell_fill_shader,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &cell_fill_shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
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

        let font_info_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Font info buffer"),
            contents: cast_slice(font_cache.info.as_slice()),
            usage: wgpu::BufferUsages::STORAGE,
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

        let glyph_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &highlights_bind_group_layout,
                    &grid_bind_group_layout,
                    &font_bind_group_layout,
                ],
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::VERTEX,
                    range: 0..GridInfo::SIZE as u32,
                }],
            });

        let glyph_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render pipeline"),
                layout: Some(&glyph_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &glyph_shader,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &glyph_shader,
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

        Self {
            surface_config: StateSurfaceConfig::new(config),
            constant: Arc::new(StateConstant {
                device,
                queue,
                surface,
                grid_bind_group_layout,
                highlights_bind_group_layout,
                cell_fill_render_pipeline,
                glyph_render_pipeline,
                font_bind_group,
            }),
            read: Arc::new(RwLock::new(None)),
            write: StateWrite::new(font, font_cache),
        }
    }

    pub fn update_text(&mut self, ui: Ui) {
        let read = self
            .write
            .update_text(ui, &self.constant, self.surface_config.size());
        // Separate statement so that the lock is taken as late as possible
        *self.read.write().unwrap() = Some(read);
    }

    pub fn winit_state(&self) -> StateWinit {
        StateWinit {
            surface_config: self.surface_config.clone(),
            constant: self.constant.clone(),
            read: self.read.clone(),
        }
    }
}

pub struct StateConstant {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface,
    pub grid_bind_group_layout: wgpu::BindGroupLayout,
    pub highlights_bind_group_layout: wgpu::BindGroupLayout,
    pub cell_fill_render_pipeline: wgpu::RenderPipeline,
    pub glyph_render_pipeline: wgpu::RenderPipeline,
    pub font_bind_group: wgpu::BindGroup,
}

pub struct StateWinit {
    surface_config: StateSurfaceConfig,
    constant: Arc<StateConstant>,
    read: Arc<RwLock<Option<StateRead>>>,
}

impl StateWinit {
    pub fn resize(&self, size: PhysicalSize<u32>) {
        self.surface_config.resize(size, self.constant.as_ref());
    }

    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let read = self.read.read().unwrap();
        if let Some(read) = read.as_ref() {
            read.render(self.constant.as_ref())
        } else {
            Ok(())
        }
    }

    pub fn rebuild_swap_chain(&self) {
        let size = self.surface_config.size();
        let size = PhysicalSize::new(size.x, size.y);
        self.surface_config.resize(size, self.constant.as_ref())
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct GlyphInfo {
    pub glyph_index: u32,
    pub highlight_index: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct GridInfo {
    pub surface_size: Vec2<u32>,
    pub cell_size: Vec2<u32>,
    pub grid_width: u32,
    pub baseline: u32,
}

impl GridInfo {
    pub const SIZE: usize = std::mem::size_of::<Self>();
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct HighlightInfo {
    pub fg: [f32; 4],
    pub bg: [f32; 4],
}
