use crate::{
    text::{
        cache::{CacheValue, FontCache, GlyphKind},
        fonts::{FontStyle, Fonts},
    },
    ui::grid::CellContents,
    util::IntoSrgb,
};
use bytemuck::{cast_slice, Pod, Zeroable};
use neophyte_linalg::{CellVec, PixelVec, Vec2};
use neophyte_ui_event::{hl_attr_define::Attributes, rgb::Rgb};
use std::num::NonZeroU64;
use swash::{
    shape::ShapeContext,
    text::{
        cluster::{CharCluster, Parser, Status, Token},
        Script,
    },
};

pub struct Text {
    monochrome: Vec<MonochromeCell>,
    emoji: Vec<EmojiCell>,
    cell_fill: Vec<BgCell>,
    decoration: Vec<Decoration>,
    buffer: Option<wgpu::Buffer>,
    buffer_capacity: u64,
    cell_fill_bind_group: Option<wgpu::BindGroup>,
    monochrome_bind_group: Option<wgpu::BindGroup>,
    emoji_bind_group: Option<wgpu::BindGroup>,
    lines_bind_group: Option<wgpu::BindGroup>,
    window_position: Option<CellVec<f32>>,
    size: CellVec<u32>,
}

impl Text {
    pub fn new(size: CellVec<u32>) -> Self {
        Self {
            monochrome: vec![],
            emoji: vec![],
            cell_fill: vec![],
            decoration: vec![],
            buffer: None,
            buffer_capacity: 0,
            cell_fill_bind_group: None,
            monochrome_bind_group: None,
            emoji_bind_group: None,
            lines_bind_group: None,
            // TODO: Should be initialized to grid position. This may be
            // causing the initial Telescope scroll.
            window_position: None,
            size,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_contents<'a>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: Option<CellVec<u32>>,
        lines: impl Iterator<Item = (i32, impl Iterator<Item = CellContents<'a>> + Clone)> + Clone,
        grid_bind_group_layout: &wgpu::BindGroupLayout,
        highlights: &[Option<Attributes>],
        default_fg: Rgb,
        default_bg: Rgb,
        fonts: &Fonts,
        font_cache: &mut FontCache,
        shape_context: &mut ShapeContext,
    ) {
        let metrics = fonts.metrics();
        let metrics_px = metrics.into_pixels();
        let cell_size = metrics_px.cell_size();

        let default_fg = default_fg.into_srgb(1.);
        let default_bg = default_bg.into_srgb(1.);

        self.monochrome.clear();
        self.emoji.clear();
        self.cell_fill.clear();
        self.decoration.clear();

        let mut cluster = CharCluster::new();
        self.size = CellVec(Vec2::new(0, 0));
        let mut line_length = 0;
        for (cell_line_i, cell_line) in lines {
            self.size.0.x = self.size.0.x.max(line_length);
            self.size.0.y += 1;
            line_length = 0;
            let mut parser = Parser::new(
                Script::Latin,
                cell_line.enumerate().flat_map(|(cell_i, cell)| {
                    cell.text.map(move |c| Token {
                        ch: c,
                        offset: cell_i as u32,
                        len: 1,
                        info: c.into(),
                        data: cell.highlight,
                    })
                }),
            );

            let mut next_font: Option<BestFont> = None;
            let mut is_parser_empty = false;
            while !is_parser_empty {
                if let Some(current_font_unwrapped) = next_font {
                    let font_info = fonts.families().nth(current_font_unwrapped.index).unwrap();
                    let font = font_info.style(current_font_unwrapped.style).unwrap();
                    let mut shaper = shape_context
                        .builder(font.as_ref())
                        .features(font_info.setting.features.iter().cloned().map(|s| s.0))
                        .variations(font_info.setting.variations.iter().cloned().map(|s| s.0))
                        .script(Script::Arabic)
                        .build();
                    shaper.add_cluster(&cluster);

                    loop {
                        if !parser.next(&mut cluster) {
                            is_parser_empty = true;
                            break;
                        }

                        let best_font = best_font(&mut cluster, fonts, highlights);
                        match best_font {
                            Some(best_font) => {
                                if current_font_unwrapped == best_font {
                                    shaper.add_cluster(&cluster);
                                } else {
                                    next_font = Some(best_font);
                                    break;
                                }
                            }

                            None => {
                                next_font = None;
                                break;
                            }
                        }
                    }

                    shaper.shape_with(|cluster| {
                        line_length += 1;
                        let (fg, is_underlined) = if let Some(hl) = highlights
                            .get(cluster.data as usize)
                            .and_then(|hl| (*hl).as_ref())
                        {
                            let blend = hl.blend();
                            let fg = hl.foreground;
                            let bg = hl.background;

                            if let Some(bg) = if hl.reverse() {
                                Some(fg.map(|fg| fg.into_srgb(blend)).unwrap_or(default_fg))
                            } else {
                                bg.map(|bg| bg.into_srgb(blend))
                            } {
                                // Although some programming fonts are said to
                                // contain ligatures, in practice these are more
                                // commonly implemented as multi-character alternates.
                                // In contrast to genuine OpenType ligatures,
                                // multi-character alternates still get a glyph cluster
                                // per input char where some of those clusters may
                                // contain an empty glyph. That means we can produce the
                                // cell fill characters during shaping without worrying
                                // too much about whether a glyph cluster spans multiple
                                // cells. This is something to improve on in the future
                                // in case some fonts contain actual ligatures.
                                let bg_cell = BgCell {
                                    x: cluster.source.start.try_into().unwrap(),
                                    y: cell_line_i,
                                    r: bg[0],
                                    g: bg[1],
                                    b: bg[2],
                                    a: bg[3],
                                };
                                self.cell_fill.push(bg_cell);
                            }

                            (
                                if hl.reverse() {
                                    bg.map(|bg| bg.into_srgb(blend)).unwrap_or(default_bg)
                                } else {
                                    fg.map(|fg| fg.into_srgb(blend)).unwrap_or(default_fg)
                                },
                                hl.underline(),
                            )
                        } else {
                            (default_fg, false)
                        };

                        let x = cluster.source.start * cell_size.x;
                        let mut advanced = 0.0f32;
                        for glyph in cluster.glyphs {
                            let CacheValue { index, kind } = match font_cache.get(
                                font.as_ref(),
                                font_info.setting.variations.as_slice(),
                                metrics.em,
                                glyph.id,
                                current_font_unwrapped.style,
                                current_font_unwrapped.index,
                            ) {
                                Some(glyph) => glyph,
                                None => {
                                    continue;
                                }
                            };
                            let glyph_index = index as u32;

                            let position = Vec2::new(glyph.x, glyph.y) * metrics.scale_factor;
                            let position = Vec2::new(
                                position.x.round() as i32 + x as i32 + advanced.round() as i32,
                                position.y.round() as i32 + (cell_line_i * cell_size.y as i32),
                            );

                            if is_underlined {
                                let line_position = position
                                    + Vec2::new(
                                        0,
                                        (metrics_px.ascent + metrics_px.underline_offset) as i32,
                                    );
                                let line_size =
                                    Vec2::new(metrics_px.width, metrics_px.stroke_size.max(1));
                                self.decoration.push(Decoration {
                                    x: line_position.x,
                                    y: line_position.y,
                                    w: line_size.x,
                                    h: line_size.y,
                                    r: fg[0],
                                    g: fg[1],
                                    b: fg[2],
                                })
                            }

                            let position = position + Vec2::new(0, metrics_px.em as i32);
                            match kind {
                                GlyphKind::Monochrome => self.monochrome.push(MonochromeCell {
                                    x: position.x,
                                    y: position.y,
                                    r: fg[0],
                                    g: fg[1],
                                    b: fg[2],
                                    glyph_index: glyph_index.try_into().unwrap(),
                                }),
                                GlyphKind::Emoji => self.emoji.push(EmojiCell {
                                    x: position.x,
                                    y: position.y,
                                    glyph_index: glyph_index.try_into().unwrap(),
                                }),
                            }
                            advanced += glyph.advance * metrics.scale_factor;
                        }
                    });
                } else {
                    loop {
                        let range = cluster.range();
                        line_length += range.end - range.start;
                        if let Some(hl) = highlights[cluster.user_data() as usize] {
                            let bg = if hl.reverse() {
                                hl.foreground
                            } else {
                                hl.background
                            };

                            if let Some(bg) = bg {
                                let bg = bg.into_srgb(hl.blend());
                                for i in range.start..range.end {
                                    let bg_cell = BgCell {
                                        x: (i * cell_size.x).try_into().unwrap(),
                                        y: cell_line_i,
                                        r: bg[0],
                                        g: bg[1],
                                        b: bg[2],
                                        a: bg[3],
                                    };
                                    self.cell_fill.push(bg_cell);
                                }
                            }
                        }

                        if !parser.next(&mut cluster) {
                            is_parser_empty = true;
                            break;
                        }

                        if let Some(best_font) = best_font(&mut cluster, fonts, highlights) {
                            next_font = Some(best_font);
                            break;
                        }
                    }
                }
            }
        }

        self.size.0.x = self.size.0.x.max(line_length);
        if let Some(size) = size {
            self.size = size;
        }

        let glyphs = cast_slice(self.monochrome.as_slice());
        let emoji = cast_slice(self.emoji.as_slice());
        let bg = cast_slice(self.cell_fill.as_slice());
        let lines = cast_slice(self.decoration.as_slice());

        let alignment = device.limits().min_storage_buffer_offset_alignment as u64;

        let glyphs_len = glyphs.len() as u64;
        let emoji_len = emoji.len() as u64;
        let bg_len = bg.len() as u64;
        let lines_len = lines.len() as u64;

        let glyphs_padding = alignment - glyphs_len % alignment;
        let emoji_padding = alignment - emoji_len % alignment;
        let bg_padding = alignment - bg_len % alignment;

        let total_length = glyphs_len
            + glyphs_padding
            + emoji_len
            + emoji_padding
            + bg_len
            + bg_padding
            + lines_len;

        if total_length > self.buffer_capacity {
            self.buffer_capacity = total_length * 2;
            self.buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Grid buffer"),
                size: self.buffer_capacity,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }

        let Some(buffer) = self.buffer.as_ref() else {
            return;
        };

        let mut offset = 0;
        queue.write_buffer(buffer, 0, glyphs);
        self.monochrome_bind_group = NonZeroU64::new(glyphs_len).map(|size| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Monochrome bind group"),
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

        queue.write_buffer(buffer, offset, emoji);
        self.emoji_bind_group = NonZeroU64::new(emoji_len).map(|size| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Emoji bind group"),
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

        queue.write_buffer(buffer, offset, bg);
        self.cell_fill_bind_group = NonZeroU64::new(bg_len).map(|size| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Cell fill bind group"),
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
        offset += bg_len + bg_padding;

        queue.write_buffer(buffer, offset, lines);
        self.lines_bind_group = NonZeroU64::new(lines_len).map(|size| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Lines bind group"),
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

    pub fn update_window(&mut self, window_position: Option<CellVec<f32>>) {
        self.window_position = window_position;
    }

    pub fn cell_fill_bind_group(&self) -> Option<&wgpu::BindGroup> {
        self.cell_fill_bind_group.as_ref()
    }

    pub fn monochrome_bind_group(&self) -> Option<&wgpu::BindGroup> {
        self.monochrome_bind_group.as_ref()
    }

    pub fn emoji_bind_group(&self) -> Option<&wgpu::BindGroup> {
        self.emoji_bind_group.as_ref()
    }

    pub fn lines_bind_group(&self) -> Option<&wgpu::BindGroup> {
        self.lines_bind_group.as_ref()
    }

    pub fn size(&self) -> CellVec<u32> {
        self.size
    }

    pub fn offset(&self) -> Option<CellVec<f32>> {
        self.window_position
    }

    pub fn cell_fill_count(&self) -> u32 {
        self.cell_fill.len().try_into().unwrap()
    }

    pub fn monochrome_count(&self) -> u32 {
        self.monochrome.len().try_into().unwrap()
    }

    pub fn emoji_count(&self) -> u32 {
        self.emoji.len().try_into().unwrap()
    }

    pub fn lines_count(&self) -> u32 {
        self.decoration.len().try_into().unwrap()
    }
}

pub fn set_scissor(
    size: PixelVec<u32>,
    offset: PixelVec<i32>,
    target_size: PixelVec<u32>,
    render_pass: &mut wgpu::RenderPass,
) {
    let target_size: Vec2<i32> = target_size.0.try_cast().unwrap();
    let minmax = |size| {
        Vec2::combine(
            Vec2::combine(size, target_size, i32::min),
            Vec2::default(),
            i32::max,
        )
    };
    let offset = offset.0;
    let size = size.0;
    let size = size.try_cast().unwrap();
    let size = offset + size;
    let size = minmax(size);
    let size = size - offset;
    let size = minmax(size).try_cast().unwrap();
    let offset = minmax(offset).try_cast().unwrap();
    render_pass.set_scissor_rect(offset.x, offset.y, size.x, size.y);
}

fn best_font(
    cluster: &mut CharCluster,
    fonts: &Fonts,
    highlights: &[Option<Attributes>],
) -> Option<BestFont> {
    let style = highlights
        .get(cluster.user_data() as usize)
        .and_then(|hl| (*hl).as_ref())
        .map(|highlight| FontStyle::new(highlight.bold(), highlight.italic()))
        .unwrap_or_default();
    let mut best_font = None;
    for (i, font_info) in fonts.families().enumerate() {
        if let Some(font) = &font_info.style(style) {
            match cluster.map(|c| font.charmap().map(c)) {
                Status::Discard => {}
                Status::Keep => {
                    best_font = Some(BestFont::new(i, style));
                    continue;
                }
                Status::Complete => {
                    best_font = Some(BestFont::new(i, style));
                    break;
                }
            }
        }

        if style != FontStyle::Regular {
            if let Some(font) = &font_info.regular {
                match cluster.map(|c| font.charmap().map(c)) {
                    Status::Discard => {}
                    Status::Keep => best_font = Some(BestFont::new(i, FontStyle::Regular)),
                    Status::Complete => {
                        best_font = Some(BestFont::new(i, FontStyle::Regular));
                        break;
                    }
                }
            }
        }
    }
    best_font
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BestFont {
    index: usize,
    style: FontStyle,
}

impl BestFont {
    pub fn new(index: usize, style: FontStyle) -> Self {
        Self { index, style }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct MonochromeCell {
    pub x: i32,
    pub y: i32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub glyph_index: i32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct EmojiCell {
    pub x: i32,
    pub y: i32,
    pub glyph_index: i32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct BgCell {
    pub x: i32,
    pub y: i32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct Decoration {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

pub struct BindGroupLayout {
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl BindGroupLayout {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            bind_group_layout: device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            }),
        }
    }
}
