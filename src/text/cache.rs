#![allow(unused)]

use crate::util::vec2::Vec2;
use std::collections::HashMap;
use swash::{
    scale::{image::Image, Render, ScaleContext, Source, StrikeWith},
    zeno::Placement,
    FontRef, GlyphId,
};

// TODO: Cache glyphs lazily

#[derive(Clone, Default)]
pub struct FontCache {
    /// The glyph image data
    pub data: Vec<Vec<u8>>,
    /// Info about glyphs. Use lut to get the index for a glyph
    pub info: Vec<GlyphInfo>,
    /// Maps a glyph to an index into the info or images array.
    pub lut: HashMap<GlyphId, usize>,
}

impl FontCache {
    pub fn from_font(font: FontRef, size: f32) -> Self {
        let mut out = Self::default();
        let mut scale_context = ScaleContext::new();
        let mut scaler = scale_context.builder(font).size(size).hint(true).build();
        font.charmap().enumerate(|_c, id| {
            let image = Render::new(&[
                Source::ColorOutline(0),
                Source::ColorBitmap(StrikeWith::BestFit),
                Source::Outline,
            ])
            .render(&mut scaler, id)
            .unwrap();
            if image.data.len() > 0 {
                let index = out.data.len();
                out.lut.insert(id, index);
                out.info.push(GlyphInfo {
                    size: Vec2::new(image.placement.width, image.placement.height),
                    placement_offset: Vec2::new(image.placement.left, image.placement.top),
                });
                out.data.push(image.data);
            }
        });
        out
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlyphInfo {
    pub size: Vec2<u32>,
    pub placement_offset: Vec2<i32>,
}

pub struct GlyphEntry {
    pub placement: Placement,
    pub index: u32,
}
