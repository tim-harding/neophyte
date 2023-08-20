use super::{grid_bind_group_layout::GridBindGroupLayout, shared::Shared};
use crate::{
    event::hl_attr_define::Attributes,
    text::{
        cache::{CacheValue, FontCache, GlyphKind},
        fonts::{FontStyle, Fonts},
    },
    ui::{self, Highlights},
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
use wgpu::util::DeviceExt;

pub struct Grid {
    pub glyph_bind_group: Option<wgpu::BindGroup>,
    pub bg_bind_group: Option<wgpu::BindGroup>,
    pub emoji_bind_group: Option<wgpu::BindGroup>,
    pub grid_info: GridInfo,
    pub glyph_count: u32,
    pub bg_count: u32,
    pub emoji_count: u32,
}

impl Grid {
    pub fn new(
        shared: &Shared,
        grid: &ui::grid::Grid,
        highlights: &Highlights,
        fonts: &mut Fonts,
        font_cache: &mut FontCache,
        shape_context: &mut ShapeContext,
        grid_bind_group_layout: &GridBindGroupLayout,
    ) -> Self {
        let mut glyph_info = vec![];
        let mut bg_info = vec![];
        let mut emoji_info = vec![];

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
                                let mut shaper = shape_context
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
                                        let style = highlights
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
                                        let CacheValue { index, kind } = match font_cache.get(
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
                                        let glyph_index = index as u32;

                                        let offset = match kind {
                                            GlyphKind::Monochrome => {
                                                font_cache.monochrome.offset[index]
                                            }
                                            GlyphKind::Emoji => font_cache.emoji.offset[index],
                                        };
                                        let position = offset * Vec2::new(1, -1)
                                            + Vec2::new(
                                                (glyph.x * scale_factor).round() as i32 + x as i32,
                                                (glyph.y * scale_factor
                                                    + (cell_line_i as u32 * cell_height_px) as f32)
                                                    .round()
                                                    as i32,
                                            );
                                        match kind {
                                            GlyphKind::Monochrome => glyph_info.push(GlyphInfo {
                                                glyph_index,
                                                highlight_index: glyph.data,
                                                position,
                                            }),
                                            GlyphKind::Emoji => emoji_info.push(EmojiCell {
                                                position,
                                                glyph_index,
                                            }),
                                        }
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
                            let style = highlights
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

        // TODO: Share a buffer between glyphs and BG cells. Probably don't need
        // optionals then.

        let glyph_bind_group = bind_group_from_buffer(
            cast_slice(glyph_info.as_slice()),
            &shared.device,
            &grid_bind_group_layout.bind_group_layout,
            "glyph buffer/bind group",
        );

        let bg_bind_group = bind_group_from_buffer(
            cast_slice(bg_info.as_slice()),
            &shared.device,
            &grid_bind_group_layout.bind_group_layout,
            "bg buffer/bind group",
        );

        let emoji_bind_group = bind_group_from_buffer(
            cast_slice(emoji_info.as_slice()),
            &shared.device,
            &grid_bind_group_layout.bind_group_layout,
            "emoji buffer/bind group",
        );

        let cell_size = Vec2::new(fonts.size(), cell_height_px);
        let grid_info = GridInfo {
            surface_size: shared.surface_size(),
            cell_size,
            // TODO: Relative to anchor grid
            offset: (grid.window.offset(grid.size) * cell_size.into()).into(),
            grid_width: grid.size.x as u32,
            baseline: em_px,
        };

        Self {
            glyph_bind_group,
            bg_bind_group,
            emoji_bind_group,
            grid_info,
            glyph_count: glyph_info.len() as u32,
            bg_count: bg_info.len() as u32,
            emoji_count: emoji_info.len() as u32,
        }
    }
}

fn bind_group_from_buffer(
    data: &[u8],
    device: &wgpu::Device,
    grid_bind_group_layout: &wgpu::BindGroupLayout,
    label: &str,
) -> Option<wgpu::BindGroup> {
    if data.is_empty() {
        None
    } else {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            usage: wgpu::BufferUsages::STORAGE,
            contents: data,
        });

        Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label),
            layout: grid_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        }))
    }
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
pub struct EmojiCell {
    position: Vec2<i32>,
    glyph_index: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct GridInfo {
    pub surface_size: Vec2<u32>,
    pub cell_size: Vec2<u32>,
    pub offset: Vec2<f32>,
    pub grid_width: u32,
    pub baseline: u32,
}

impl GridInfo {
    pub const SIZE: usize = std::mem::size_of::<Self>();
}
