use super::{highlights, shared::Shared};
use crate::{
    event::hl_attr_define::Attributes,
    text::{
        cache::FontCache,
        fonts::{FontStyle, Fonts},
    },
    ui::Ui,
    util::vec2::Vec2,
};
use bytemuck::{cast_slice, Pod, Zeroable};
use swash::{
    shape::ShapeContext,
    text::{
        cluster::{CharCluster, Parser, Status, Token},
        Script,
    },
};
use wgpu::{include_wgsl, util::DeviceExt};

pub struct Read {
    pub glyph_bind_group: wgpu::BindGroup,
    pub bg_bind_group: wgpu::BindGroup,
    pub grid_info: GridInfo,
    pub glyph_count: usize,
    pub bg_count: usize,
}

#[derive(Default)]
pub struct Write {
    pub shape_context: ShapeContext,
}

impl Write {
    pub fn updates(
        &mut self,
        grid_constant: &Constant,
        shared: &Shared,
        ui: &Ui,
        fonts: &mut Fonts,
        font_cache: &mut FontCache,
    ) -> Option<Read> {
        if ui.options.guifont.1 > 0 {
            // TODO: Also need to resize grid
            // TODO: Clear font cache
            // TODO: Also reload textures on GPU
            fonts.reload(ui.options.guifont.0.clone(), ui.options.guifont.1)
        }

        let grid = ui.composite();
        let mut glyph_info = vec![];
        let mut bg_info = vec![];

        let metrics = fonts
            .with_style(FontStyle::Regular)
            .unwrap()
            .as_ref()
            .metrics(&[]);

        let scale_factor = fonts.size() as f32 / metrics.average_width;
        let em = metrics.units_per_em as f32 * scale_factor;
        let em_px = em.ceil() as u32;
        let descent = metrics.descent * scale_factor;
        let descent_px = descent.ceil() as u32;
        let cell_height_px = em_px + descent_px;

        let grid_info = GridInfo {
            surface_size: shared.surface_size(),
            cell_size: Vec2::new(fonts.size(), cell_height_px),
            grid_width: grid.size.x as u32,
            baseline: em_px,
        };

        for (cell_line_i, cell_line) in grid.rows().enumerate() {
            let mut cluster = CharCluster::new();
            let mut parser = Parser::new(
                Script::Latin,
                cell_line.enumerate().flat_map(|(cell_i, cell)| {
                    cell.text.chars().map(move |c| Token {
                        ch: c,
                        offset: cell_i as u32,
                        len: 0,
                        info: c.into(),
                        data: cell.highlight as u32,
                    })
                }),
            );

            let mut current_font: Option<(usize, FontStyle)> = None;
            let mut is_parser_empty = false;
            while !is_parser_empty {
                match current_font {
                    Some(current_font_unwrapped) => {
                        let font_info = fonts.iter().nth(current_font_unwrapped.0).unwrap();
                        match &font_info.style(current_font_unwrapped.1) {
                            Some(font) => {
                                let mut shaper = self
                                    .shape_context
                                    .builder(font.as_ref())
                                    .script(Script::Arabic)
                                    .build();

                                shaper.add_cluster(&cluster);

                                loop {
                                    if !parser.next(&mut cluster) {
                                        is_parser_empty = true;
                                        break;
                                    }

                                    let mut best_font = None;
                                    for (i, font_info) in fonts.iter().enumerate() {
                                        let style = ui
                                            .highlights
                                            .get(&(cluster.user_data() as u64))
                                            .map(|highlight| {
                                                let Attributes { bold, italic, .. } =
                                                    highlight.rgb_attr;
                                                let bold = bold.unwrap_or_default();
                                                let italic = italic.unwrap_or_default();
                                                FontStyle::new(bold, italic)
                                            })
                                            .unwrap_or_default();

                                        if let Some(font) = &font_info.style(style) {
                                            match cluster.map(|c| font.charmap().map(c)) {
                                                Status::Discard => {}
                                                Status::Keep => best_font = Some((i, style)),
                                                Status::Complete => {
                                                    best_font = Some((i, style));
                                                    break;
                                                }
                                            }
                                        } else if style != FontStyle::Regular {
                                            if let Some(font) = &font_info.regular {
                                                match cluster.map(|c| font.charmap().map(c)) {
                                                    Status::Discard => {}
                                                    Status::Keep => best_font = Some((i, style)),
                                                    Status::Complete => {
                                                        best_font = Some((i, style));
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    match best_font {
                                        Some(best_font) => {
                                            if current_font_unwrapped == best_font {
                                                shaper.add_cluster(&cluster);
                                            } else {
                                                current_font = Some(best_font);
                                                break;
                                            }
                                        }

                                        None => {
                                            current_font = best_font;
                                            break;
                                        }
                                    }
                                }

                                shaper.shape_with(|glyph_cluster| {
                                    let x = glyph_cluster.source.start * fonts.size();
                                    if glyph_cluster.data > 0 {
                                        bg_info.push(BgInfo {
                                            x: x as i32,
                                            y: cell_line_i as i32 * cell_height_px as i32,
                                            highlight_index: glyph_cluster.data,
                                            width: fonts.size(),
                                        });
                                    }

                                    for glyph in glyph_cluster.glyphs {
                                        let glyph_index = match font_cache.get(
                                            font.as_ref(),
                                            em,
                                            glyph.id,
                                            current_font_unwrapped.1,
                                        ) {
                                            Some(glyph) => glyph,
                                            None => {
                                                continue;
                                            }
                                        };

                                        let offset = font_cache.offset[glyph_index];
                                        glyph_info.push(GlyphInfo {
                                            glyph_index: glyph_index as u32,
                                            highlight_index: glyph.data,
                                            position: offset * Vec2::new(1, -1)
                                                + Vec2::new(
                                                    (glyph.x * scale_factor).round() as i32
                                                        + x as i32,
                                                    (glyph.y * scale_factor
                                                        + (cell_line_i as u32
                                                            * grid_info.cell_size.y)
                                                            as f32)
                                                        .round()
                                                        as i32,
                                                ),
                                        });
                                    }
                                });
                            }
                            None => todo!(),
                        }
                    }

                    None => loop {
                        if !parser.next(&mut cluster) {
                            is_parser_empty = true;
                            break;
                        }

                        let mut best_font = None;
                        for (i, font_info) in fonts.iter().enumerate() {
                            let style = ui
                                .highlights
                                .get(&(cluster.user_data() as u64))
                                .map(|highlight| {
                                    let Attributes { bold, italic, .. } = highlight.rgb_attr;
                                    let bold = bold.unwrap_or_default();
                                    let italic = italic.unwrap_or_default();
                                    FontStyle::new(bold, italic)
                                })
                                .unwrap_or_default();

                            if let Some(font) = &font_info.style(style) {
                                match cluster.map(|c| font.charmap().map(c)) {
                                    Status::Discard => {}
                                    Status::Keep => best_font = Some((i, style)),
                                    Status::Complete => {
                                        best_font = Some((i, style));
                                        break;
                                    }
                                }
                            } else if style != FontStyle::Regular {
                                if let Some(font) = &font_info.regular {
                                    match cluster.map(|c| font.charmap().map(c)) {
                                        Status::Discard => {}
                                        Status::Keep => best_font = Some((i, style)),
                                        Status::Complete => {
                                            best_font = Some((i, style));
                                            break;
                                        }
                                    }
                                }
                            }
                        }

                        if current_font != best_font {
                            current_font = best_font;
                            break;
                        }
                    },
                }
            }
        }

        if glyph_info.is_empty() {
            return None;
        }

        let glyph_info_buffer =
            shared
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("glyph info buffer"),
                    usage: wgpu::BufferUsages::STORAGE,
                    contents: cast_slice(glyph_info.as_slice()),
                });

        let bg_info_buffer = shared
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("bg info buffer"),
                usage: wgpu::BufferUsages::STORAGE,
                contents: cast_slice(bg_info.as_slice()),
            });

        let glyph_bind_group = shared.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("glyph info bind group"),
            layout: &grid_constant.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &glyph_info_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        let bg_bind_group = shared.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bg info bind group"),
            layout: &grid_constant.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &bg_info_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        Some(Read {
            glyph_bind_group,
            bg_bind_group,
            grid_info,
            glyph_count: glyph_info.len(),
            bg_count: bg_info.len(),
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
    highlights: &highlights::HighlightsBindGroupLayout,
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
        Write::default(),
        Constant {
            bind_group_layout,
            cell_fill_render_pipeline,
        },
    )
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct BgInfo {
    pub x: i32,
    pub y: i32,
    pub highlight_index: u32,
    pub width: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct GlyphInfo {
    pub glyph_index: u32,
    pub highlight_index: u32,
    pub position: Vec2<i32>,
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
