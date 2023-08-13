use super::{highlights, ConstantState};
use crate::{
    text::{
        cache::FontCache,
        font::{metrics, Font},
    },
    ui::Ui,
    util::vec2::Vec2,
};
use bytemuck::{cast_slice, Pod, Zeroable};
use wgpu::{include_wgsl, util::DeviceExt};

pub struct Read {
    pub bind_group: wgpu::BindGroup,
    pub grid_info: GridInfo,
    pub vertex_count: u32,
}

pub struct Write;

impl Write {
    pub fn updates(
        &mut self,
        constant: &ConstantState,
        surface_size: Vec2<u32>,
        ui: &Ui,
        font: &Font,
        font_cache: &mut FontCache,
    ) -> Option<Read> {
        let grid = ui.composite();
        let font = font.as_ref();
        let charmap = font.charmap();
        let metrics = metrics(font, 24.0);
        let mut glyph_info = vec![];
        for (cell_line, hl_line) in grid.cells.rows().zip(grid.highlights.rows()) {
            for (c, hl) in cell_line.zip(hl_line) {
                let id = charmap.map(c);
                let glyph_index = match font_cache.get(font, 24.0, id) {
                    Some(glyph) => glyph,
                    None => {
                        glyph_info.push(GlyphInfo {
                            glyph_index: 0,
                            highlight_index: hl,
                        });
                        continue;
                    }
                };

                glyph_info.push(GlyphInfo {
                    glyph_index: glyph_index as u32,
                    highlight_index: hl,
                });
            }
        }

        let grid_info = GridInfo {
            surface_size,
            cell_size: Vec2::new(metrics.advance as u32, metrics.cell_height()),
            grid_width: grid.size().x as u32,
            baseline: metrics.ascent as u32,
        };

        let vertex_count = glyph_info.len() as u32 * 6;

        let glyph_info_buffer =
            constant
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("info buffer"),
                    usage: wgpu::BufferUsages::STORAGE,
                    contents: cast_slice(glyph_info.as_slice()),
                });

        let bind_group = constant
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("glyph info bind group"),
                layout: &constant.grid.bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &glyph_info_buffer,
                        offset: 0,
                        size: None,
                    }),
                }],
            });

        Some(Read {
            bind_group,
            grid_info,
            vertex_count,
        })
    }
}

pub struct Constant {
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub cell_fill_render_pipeline: wgpu::RenderPipeline,
}

pub fn init(
    device: &wgpu::Device,
    texture_format: wgpu::TextureFormat,
    highlights: &highlights::Constant,
) -> (Write, Constant) {
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

    let cell_fill_pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Cell fill pipeline layout"),
            bind_group_layouts: &[&highlights.bind_group_layout, &bind_group_layout],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX,
                range: 0..GridInfo::SIZE as u32,
            }],
        });

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
                    format: texture_format,
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

    (
        Write,
        Constant {
            bind_group_layout,
            cell_fill_render_pipeline,
        },
    )
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
