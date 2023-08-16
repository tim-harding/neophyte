use std::collections::HashMap;

use super::{highlights, ConstantState};
use crate::{
    text::{cache::FontCache, font::metrics, fonts::Fonts},
    ui::Ui,
    util::vec2::Vec2,
};
use bytemuck::{cast_slice, Pod, Zeroable};
use swash::{
    shape::ShapeContext,
    text::{
        cluster::{CharCluster, Parser, Token},
        Script,
    },
};
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
        fonts: &mut Fonts,
        font_cache: &mut FontCache,
    ) -> Option<Read> {
        if ui.options.guifont.1 > 0 {
            // TODO: Also need to resize grid and clear font cache
            fonts.reload(ui.options.guifont.0.clone(), ui.options.guifont.1)
        }
        let grid = ui.composite();
        let default_font = fonts.first_regular().unwrap();
        let default_font = default_font.as_ref();
        let charmap = default_font.charmap();
        let metrics = metrics(default_font, fonts.size() as f32);
        let mut glyph_info = vec![];
        // TODO: Cache
        let mut shape_context = ShapeContext::new();

        for (cell_line, mut hl_line) in grid.cells.rows().zip(grid.highlights.rows()) {
            let mut shaper = shape_context
                .builder(default_font)
                .script(Script::Latin)
                .build();
            let mut cluster = CharCluster::new();
            let mut parser = Parser::new(
                Script::Latin,
                cell_line.enumerate().map(|(i, c)| Token {
                    ch: c,
                    // We essentially just store UTF-32, so each character is one code unit.
                    offset: i as u32,
                    len: 1,
                    info: c.into(),
                    data: 0,
                }),
            );

            while parser.next(&mut cluster) {
                // NOTE: Why does the shaper builder take a font if we select the best font here?
                cluster.map(|c| charmap.map(c));
                shaper.add_cluster(&cluster);
            }

            shaper.shape_with(|glyph_cluster| {
                for glyph in glyph_cluster.glyphs {
                    let hl = hl_line.next().unwrap_or(0);
                    let glyph_index = match font_cache.get(fonts, fonts.size() as f32, glyph.id) {
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
            });
            println!();
        }

        if glyph_info.is_empty() {
            return None;
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
