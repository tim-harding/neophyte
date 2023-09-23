use super::shared::Shared;
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
use std::num::NonZeroU64;
use swash::{
    shape::ShapeContext,
    text::{
        cluster::{CharCluster, Parser, Status, Token},
        Script,
    },
};

#[derive(Default)]
pub struct Grid {
    pub id: u64,
    glyphs: Vec<GlyphInfo>,
    emoji: Vec<EmojiCell>,
    bg: Vec<u32>,
    buffer_capacity: u64,
    pub buffer: Option<wgpu::Buffer>,
    pub bg_bind_group: Option<wgpu::BindGroup>,
    pub monochrome_bind_group: Option<wgpu::BindGroup>,
    pub emoji_bind_group: Option<wgpu::BindGroup>,
    pub grid_info: PushConstants,
    pub glyph_count: u32,
    pub bg_count: u32,
    pub emoji_count: u32,
}

impl Grid {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_content(
        &mut self,
        shared: &Shared,
        grid: &ui::grid::Grid,
        highlights: &Highlights,
        fonts: &mut Fonts,
        font_cache: &mut FontCache,
        shape_context: &mut ShapeContext,
        grid_bind_group_layout: &wgpu::BindGroupLayout,
    ) {
        self.glyphs.clear();
        self.emoji.clear();
        self.bg.clear();
        for cell in grid.buffer.iter() {
            self.bg.push(cell.highlight as u32);
        }

        let metrics = fonts.with_style(FontStyle::Regular).metrics(fonts.size());

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

                                    for (i, font_info) in fonts.iter().enumerate() {
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
                                                    Status::Keep => {
                                                        best_font = Some((i, FontStyle::Regular))
                                                    }
                                                    Status::Complete => {
                                                        best_font = Some((i, FontStyle::Regular));
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
                                    for glyph in glyph_cluster.glyphs {
                                        let CacheValue { index, kind } = match font_cache.get(
                                            font.as_ref(),
                                            metrics.em,
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
                                                (glyph.x * metrics.scale_factor).round() as i32
                                                    + x as i32,
                                                (glyph.y * metrics.scale_factor
                                                    + (cell_line_i as u32 * metrics.cell_size_px.y
                                                        + metrics.em_px)
                                                        as f32)
                                                    .round()
                                                    as i32,
                                            );
                                        match kind {
                                            GlyphKind::Monochrome => self.glyphs.push(GlyphInfo {
                                                glyph_index,
                                                highlight_index: glyph.data,
                                                position,
                                            }),
                                            GlyphKind::Emoji => self.emoji.push(EmojiCell {
                                                position,
                                                glyph_index,
                                                padding: 0,
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

        self.glyph_count = self.glyphs.len() as u32;
        self.emoji_count = self.emoji.len() as u32;
        self.bg_count = self.bg.len() as u32;

        let glyphs = cast_slice(self.glyphs.as_slice());
        let emoji = cast_slice(self.emoji.as_slice());
        let bg = cast_slice(self.bg.as_slice());

        let alignment = shared.device.limits().min_storage_buffer_offset_alignment as u64;
        let glyphs_len = glyphs.len() as u64;
        let emoji_len = emoji.len() as u64;
        let bg_len = bg.len() as u64;
        let glyphs_padding = alignment - glyphs_len % alignment;
        let emoji_padding = alignment - emoji_len % alignment;
        let total_length = glyphs_len + glyphs_padding + emoji_len + emoji_padding + bg_len;

        if total_length > self.buffer_capacity {
            // 50% extra space to reduce reallocation
            self.buffer_capacity = total_length * 3 / 2;
            self.buffer = Some(shared.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Grid buffer"),
                size: self.buffer_capacity,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }

        let buffer = self.buffer.as_ref().unwrap();

        let mut offset = 0;
        shared.queue.write_buffer(buffer, 0, glyphs);
        self.monochrome_bind_group = NonZeroU64::new(glyphs_len).map(|size| {
            shared.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Glyph bind group"),
                layout: grid_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer,
                        offset,
                        size: Some(size),
                    }),
                }],
            })
        });
        offset += glyphs_len + glyphs_padding;

        shared.queue.write_buffer(buffer, offset, emoji);
        self.emoji_bind_group = NonZeroU64::new(emoji_len).map(|size| {
            shared.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Glyph bind group"),
                layout: grid_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer,
                        offset,
                        size: Some(size),
                    }),
                }],
            })
        });
        offset += emoji_len + emoji_padding;

        shared.queue.write_buffer(buffer, offset, bg);
        self.bg_bind_group = NonZeroU64::new(bg_len).map(|size| {
            shared.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Glyph bind group"),
                layout: grid_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer,
                        offset,
                        size: Some(size),
                    }),
                }],
            })
        });
    }

    pub fn update_grid_info(
        &mut self,
        fonts: &Fonts,
        grid: &ui::grid::Grid,
        position: Vec2<f64>,
        z: f32,
    ) {
        let metrics = fonts.with_style(FontStyle::Regular).metrics(fonts.size());
        self.grid_info = PushConstants {
            offset: (position * metrics.cell_size_px.into()).into(),
            grid_width: grid.size.x as u32,
            z,
        };
    }
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
    padding: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct PushConstants {
    pub offset: Vec2<f32>,
    pub grid_width: u32,
    pub z: f32,
}

impl PushConstants {
    pub const SIZE: usize = std::mem::size_of::<Self>();
}
