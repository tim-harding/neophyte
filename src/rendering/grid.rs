use crate::{
    event::hl_attr_define::Attributes,
    text::{
        cache::{CacheValue, FontCache, GlyphKind},
        fonts::{FontStyle, Fonts},
    },
    ui::{
        grid::{Cell as UiCell, Grid as UiGrid},
        packed_char::PackedCharContents,
        Highlights,
    },
    util::vec2::Vec2,
};
use bytemuck::{cast_slice, Pod, Zeroable};
use std::{
    cmp::Ordering,
    num::NonZeroU64,
    ops::{Add, Sub},
    str::Chars,
};
use swash::{
    shape::ShapeContext,
    text::{
        cluster::{CharCluster, Parser, Status, Token},
        Script,
    },
};

use super::Motion;

#[derive(Default)]
pub struct Grid {
    id: u64,
    monochrome: Vec<Cell>,
    emoji: Vec<Cell>,
    cell_fill: Vec<u32>,
    buffer: Option<wgpu::Buffer>,
    buffer_capacity: u64,
    cell_fill_bind_group: Option<wgpu::BindGroup>,
    monochrome_bind_group: Option<wgpu::BindGroup>,
    emoji_bind_group: Option<wgpu::BindGroup>,
    monochrome_count: u32,
    emoji_count: u32,
    offset: Vec2<i32>,
    size: Vec2<u32>,
    scrolling: ScrollingGrids,
}

impl Grid {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    pub fn scrolling(&self) -> &ScrollingGrids {
        &self.scrolling
    }

    pub fn scrolling_mut(&mut self) -> &mut ScrollingGrids {
        &mut self.scrolling
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_grid(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        grid_bind_group_layout: &wgpu::BindGroupLayout,
        highlights: &Highlights,
        fonts: &mut Fonts,
        font_cache: &mut FontCache,
        shape_context: &mut ShapeContext,
        grid: &UiGrid,
    ) {
        let metrics = fonts.metrics();
        let metrics_px = metrics.into_pixels();
        let cell_size = metrics_px.cell_size();
        self.size = grid.size.try_cast().unwrap();

        self.monochrome.clear();
        self.emoji.clear();
        self.cell_fill.clear();

        for cell in grid.buffer.iter() {
            self.cell_fill.push(cell.highlight);
        }

        for (cell_line_i, cell_line) in self.scrolling.rows(grid) {
            let mut cluster = CharCluster::new();
            let mut parser = Parser::new(
                Script::Latin,
                cell_line.enumerate().flat_map(|(cell_i, cell)| {
                    let iter: OnceOrChars = match cell.text.contents() {
                        PackedCharContents::Char(c) => c.into(),
                        PackedCharContents::U22(u22) => {
                            grid.overflow[u22.as_u32() as usize].chars().into()
                        }
                    };
                    iter.map(move |c| Token {
                        ch: c,
                        offset: cell_i as u32,
                        len: 0,
                        info: c.into(),
                        data: cell.highlight,
                    })
                }),
            );

            let mut next_font: Option<BestFont> = None;
            let mut is_parser_empty = false;
            while !is_parser_empty {
                match next_font {
                    Some(current_font_unwrapped) => {
                        let font_info = fonts.iter().nth(current_font_unwrapped.index).unwrap();
                        let font = font_info.style(current_font_unwrapped.style).unwrap();
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

                        shaper.shape_with(|glyph_cluster| {
                            let x = glyph_cluster.source.start * cell_size.x;
                            for glyph in glyph_cluster.glyphs {
                                let CacheValue { index, kind } = match font_cache.get(
                                    font.as_ref(),
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

                                let offset = match kind {
                                    GlyphKind::Monochrome => font_cache.monochrome.offset[index],
                                    GlyphKind::Emoji => font_cache.emoji.offset[index],
                                };
                                let position = offset * Vec2::new(1, -1)
                                    + Vec2::new(
                                        (glyph.x * metrics.scale_factor).round() as i32 + x as i32,
                                        (glyph.y * metrics.scale_factor
                                            + (cell_line_i * cell_size.y as i64
                                                + metrics_px.em as i64)
                                                as f32)
                                            .round() as i32,
                                    );
                                match kind {
                                    GlyphKind::Monochrome => self.monochrome.push(Cell {
                                        position,
                                        glyph_index,
                                        highlight_index: glyph.data,
                                    }),
                                    GlyphKind::Emoji => self.emoji.push(Cell {
                                        position,
                                        glyph_index,
                                        highlight_index: 0,
                                    }),
                                }
                            }
                        });
                    }

                    None => loop {
                        if !parser.next(&mut cluster) {
                            is_parser_empty = true;
                            break;
                        }
                        if let Some(best_font) = best_font(&mut cluster, fonts, highlights) {
                            next_font = Some(best_font);
                            break;
                        }
                    },
                }
            }
        }

        self.monochrome_count = self.monochrome.len() as u32;
        self.emoji_count = self.emoji.len() as u32;

        let glyphs = cast_slice(self.monochrome.as_slice());
        let emoji = cast_slice(self.emoji.as_slice());
        let bg = cast_slice(self.cell_fill.as_slice());

        let alignment = device.limits().min_storage_buffer_offset_alignment as u64;
        let glyphs_len = glyphs.len() as u64;
        let emoji_len = emoji.len() as u64;
        let bg_len = bg.len() as u64;
        let glyphs_padding = alignment - glyphs_len % alignment;
        let emoji_padding = alignment - emoji_len % alignment;
        let total_length = glyphs_len + glyphs_padding + emoji_len + emoji_padding + bg_len;

        if total_length > self.buffer_capacity {
            self.buffer_capacity = total_length * 2;
            self.buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Grid buffer"),
                size: self.buffer_capacity,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }

        let buffer = self.buffer.as_ref().unwrap();

        let mut offset = 0;
        queue.write_buffer(buffer, 0, glyphs);
        self.monochrome_bind_group = NonZeroU64::new(glyphs_len).map(|size| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
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

        queue.write_buffer(buffer, offset, emoji);
        self.emoji_bind_group = NonZeroU64::new(emoji_len).map(|size| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
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

        queue.write_buffer(buffer, offset, bg);
        self.cell_fill_bind_group = NonZeroU64::new(bg_len).map(|size| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
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

    pub fn update_window(&mut self, position: Vec2<f64>, cell_size: Vec2<f64>) {
        let offset = position * cell_size;
        self.offset = Vec2::new(offset.x as i32, offset.y as i32);
    }

    pub fn id(&self) -> u64 {
        self.id
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

    pub fn size(&self) -> Vec2<u32> {
        self.size
    }

    pub fn offset(&self) -> Vec2<i32> {
        self.offset
    }

    pub fn cell_fill_count(&self) -> u32 {
        self.size.area()
    }

    pub fn monochrome_count(&self) -> u32 {
        self.monochrome_count
    }

    pub fn emoji_count(&self) -> u32 {
        self.emoji_count
    }
}

fn best_font(
    cluster: &mut CharCluster,
    fonts: &Fonts,
    highlights: &Highlights,
) -> Option<BestFont> {
    let style = highlights
        .get(&(cluster.user_data() as u64))
        .map(|highlight| {
            let Attributes { bold, italic, .. } = highlight.rgb_attr;
            let bold = bold.unwrap_or_default();
            let italic = italic.unwrap_or_default();
            FontStyle::new(bold, italic)
        })
        .unwrap_or_default();
    let mut best_font = None;
    for (i, font_info) in fonts.iter().enumerate() {
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
pub struct Cell {
    pub position: Vec2<i32>,
    pub glyph_index: u32,
    pub highlight_index: u32,
}

#[derive(Clone)]
enum OnceOrChars<'a> {
    Char(std::iter::Once<char>),
    Chars(Chars<'a>),
}

impl<'a> From<char> for OnceOrChars<'a> {
    fn from(c: char) -> Self {
        Self::Char(std::iter::once(c))
    }
}

impl<'a> From<Chars<'a>> for OnceOrChars<'a> {
    fn from(chars: Chars<'a>) -> Self {
        Self::Chars(chars)
    }
}

impl<'a> Iterator for OnceOrChars<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            OnceOrChars::Char(iter) => iter.next(),
            OnceOrChars::Chars(iter) => iter.next(),
        }
    }
}

// TODO: Remove scrolling grids that have exited the viewport
#[derive(Default)]
pub struct ScrollingGrids {
    scrolling_count: usize,
    scrolling: Vec<GridPart>,
    t: f32,
}

impl ScrollingGrids {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn finish_scroll(&mut self) {
        self.scrolling_count = 0;
        for grid in self.scrolling.iter_mut() {
            grid.clear();
        }
    }

    pub fn target_offset(&self) -> i64 {
        self.scrolling
            .first()
            .map(|grid_part| grid_part.offset)
            .unwrap_or(0)
    }

    pub fn t(&self) -> f32 {
        self.t
    }

    pub fn advance(&mut self, delta_seconds: f32) -> Motion {
        if self.t.abs() < 0.25 {
            self.t = 0.0;
            Motion::Still
        } else {
            let sign = if self.t.is_sign_positive() { -1.0 } else { 1.0 };
            let magnitude = (delta_seconds * 1_0.).min(self.t.abs());
            self.t += sign * magnitude;
            Motion::Animating
        }
    }

    // TODO: Better to take ownership of previous grid?
    pub fn push(&mut self, grid: &UiGrid, offset: i64, current_grid_height: usize) {
        if self.scrolling_count == self.scrolling.len() {
            self.scrolling.push(GridPart::new(grid.clone()));
        } else {
            let part = &mut self.scrolling[self.scrolling_count];
            part.grid.copy_from(grid);
        }

        self.t += offset as f32;
        self.scrolling_count += 1;
        let mut cover = Range::until(current_grid_height as i64);
        for part in self.scrolling.iter_mut().take(self.scrolling_count) {
            part.offset -= offset;
            let grid_range = Range::until(part.grid.size.y as i64) + part.offset;
            let grid_range = grid_range.cover(cover) - part.offset;
            cover = cover.union(grid_range);
            part.start = grid_range.start.try_into().unwrap();
            part.end = grid_range.end.try_into().unwrap();
        }
    }

    pub fn rows<'a, 'b: 'a>(
        &'a self,
        current: &'b UiGrid,
    ) -> impl Iterator<Item = (i64, impl Iterator<Item = &'a UiCell> + '_ + Clone)> + '_ + Clone
    {
        current
            .rows()
            .enumerate()
            .map(|(i, row)| (i as i64, row))
            .chain(
                self.scrolling
                    .iter()
                    .take(self.scrolling_count)
                    .rev()
                    .flat_map(|part| {
                        part.grid
                            .rows()
                            .enumerate()
                            .skip(part.start)
                            .take(part.end - part.start)
                            .map(|(i, cells)| (i as i64 + part.offset, cells))
                    }),
            )
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct Range {
    start: i64,
    end: i64,
}

impl Range {
    pub fn new(start: i64, end: i64) -> Self {
        assert!(end >= start);
        Self { start, end }
    }

    pub fn until(end: i64) -> Self {
        Self::new(0, end)
    }

    pub const fn len(&self) -> usize {
        (self.end - self.start) as usize
    }

    pub fn union(self, other: Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    pub fn covered(self) -> Self {
        Self::new(self.start, self.start)
    }

    pub fn cover(self, cover: Self) -> Self {
        match self.start.cmp(&cover.start) {
            // |...
            //    -...
            Ordering::Less => match self.end.cmp(&cover.start) {
                // |    |
                //         ------
                // ^    ^
                Ordering::Less => self,

                // |    |
                //      ------
                // ^    ^
                Ordering::Equal => self,

                // |    |
                //    ------
                // ^  ^
                Ordering::Greater => Self::new(self.start, cover.start),
            },

            // |...
            // -...
            Ordering::Equal => match self.end.cmp(&cover.end) {
                // |    |
                // --------
                // ^
                // ^
                Ordering::Less => self.covered(),

                // |    |
                // ------
                // ^
                // ^
                Ordering::Equal => self.covered(),

                // |    |
                // ---
                //   ^  ^
                Ordering::Greater => Self::new(cover.end, self.end),
            },

            //    |...
            // -...
            Ordering::Greater => {
                match self.start.cmp(&cover.end) {
                    //   |...
                    // ------
                    Ordering::Less => match self.end.cmp(&cover.end) {
                        //   |    |
                        // ----------
                        //   ^
                        //   ^
                        Ordering::Less => self.covered(),

                        //   |    |
                        // --------
                        //   ^
                        //   ^
                        Ordering::Equal => self.covered(),

                        //   |    |
                        // -----
                        //     ^  ^
                        Ordering::Greater => Self::new(cover.end, self.end),
                    },

                    //      |...
                    // ------
                    Ordering::Equal => self,

                    //          |...
                    // ------
                    Ordering::Greater => self,
                }
            }
        }
    }
}

impl Add<i64> for Range {
    type Output = Self;

    fn add(self, rhs: i64) -> Self::Output {
        Self::new(self.start + rhs, self.end + rhs)
    }
}

impl Sub<i64> for Range {
    type Output = Self;

    fn sub(self, rhs: i64) -> Self::Output {
        Self::new(self.start - rhs, self.end - rhs)
    }
}

struct GridPart {
    grid: UiGrid,
    offset: i64,
    start: usize,
    end: usize,
}

impl GridPart {
    pub fn new(grid: UiGrid) -> Self {
        Self {
            grid,
            offset: 0,
            start: 0,
            end: 0,
        }
    }

    pub fn clear(&mut self) {
        self.grid.clear();
        self.offset = 0;
        self.start = 0;
        self.end = 0;
    }
}
