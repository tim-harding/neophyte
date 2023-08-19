use super::{
    grid::{self, GridInfo},
    highlights,
    shared::Shared,
};
use crate::{rendering::texture::Texture, text::cache::FontCache};
use bytemuck::cast_slice;
use std::num::NonZeroU32;
use wgpu::{include_wgsl, util::DeviceExt};

// TODO: Resizable glyph info buffer
// TODO: Separate out resources

pub struct Write {
    textures: Vec<Texture>,
    next_glyph_to_upload: usize,
    pub sampler: wgpu::Sampler,
    pub glyph_shader: wgpu::ShaderModule,
    pub bind_group: Option<wgpu::BindGroup>,
    pub pipeline: Option<wgpu::RenderPipeline>,
}

impl Write {
    pub fn new(device: &wgpu::Device) -> Self {
        Write {
            textures: vec![],
            next_glyph_to_upload: 0,
            sampler: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Texture sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }),
            glyph_shader: device.create_shader_module(include_wgsl!("glyph.wgsl")),
            bind_group: None,
            pipeline: None,
        }
    }

    pub fn updates(
        &mut self,
        shared: &Shared,
        font_cache: &FontCache,
        grid_constant: &grid::Constant,
        highlights_constant: &highlights::HighlightsBindGroupLayout,
    ) {
        // Only update pipeline if there are textures to upload
        if self.next_glyph_to_upload == font_cache.data.len() {
            return;
        }

        for (data, size) in font_cache
            .data
            .iter()
            .zip(font_cache.size.iter())
            .skip(self.next_glyph_to_upload)
        {
            self.textures.push(Texture::new(
                &shared.device,
                &shared.queue,
                data.as_slice(),
                *size,
                wgpu::TextureFormat::R8Unorm,
            ));
        }

        self.next_glyph_to_upload = self.textures.len();

        let views: Vec<_> = self.textures.iter().map(|texture| &texture.view).collect();

        let tex_count = Some(NonZeroU32::new(self.textures.len() as u32).unwrap());
        let font_bind_group_layout =
            shared
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let font_info_buffer =
            shared
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Font info buffer"),
                    contents: cast_slice(font_cache.size.as_slice()),
                    usage: wgpu::BufferUsages::STORAGE,
                });

        let glyph_pipeline_layout =
            shared
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[
                        &highlights_constant.bind_group_layout,
                        &grid_constant.bind_group_layout,
                        &font_bind_group_layout,
                    ],
                    push_constant_ranges: &[wgpu::PushConstantRange {
                        stages: wgpu::ShaderStages::VERTEX,
                        range: 0..GridInfo::SIZE as u32,
                    }],
                });

        self.pipeline = Some(shared.device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("Render pipeline"),
                layout: Some(&glyph_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &self.glyph_shader,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &self.glyph_shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: shared.surface_config.format,
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
            },
        ));

        self.bind_group = Some(shared.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Font bind group"),
            layout: &font_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(views.as_slice()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
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
        }));
    }
}
