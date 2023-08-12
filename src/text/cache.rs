use crate::util::vec2::Vec2;
use bytemuck::{Pod, Zeroable};
use std::collections::{hash_map::Entry, HashMap};
use swash::{
    scale::{Render, ScaleContext, Source, StrikeWith},
    FontRef, GlyphId,
};

pub struct FontCache {
    scale_context: ScaleContext, // TODO: Externalize
    /// The glyph image data
    pub data: Vec<Vec<u8>>,
    /// Info about glyphs. Use lut to get the index for a glyph
    pub info: Vec<GlyphInfo>,
    /// Maps a glyph to an index into the info or images array.
    pub lut: HashMap<GlyphId, usize>,
}

impl FontCache {
    pub fn new() -> Self {
        Self {
            // For a glyph ID of zero, use one-pixel black texture with a
            // zero-sized placement so nothing renders
            data: vec![vec![0]],
            info: vec![GlyphInfo {
                size: Vec2::new(1, 1),
                placement_offset: Vec2::default(),
            }],
            lut: HashMap::default(),
            scale_context: ScaleContext::new(),
        }
    }

    pub fn get(&mut self, font: FontRef, size: f32, glyph_id: GlyphId) -> Option<usize> {
        match self.lut.entry(glyph_id) {
            Entry::Occupied(entry) => Some(*entry.get()),
            Entry::Vacant(entry) => {
                let mut scaler = self
                    .scale_context
                    .builder(font)
                    .size(size)
                    .hint(true)
                    .build();

                Render::new(&[
                    Source::ColorOutline(0),
                    Source::ColorBitmap(StrikeWith::BestFit),
                    Source::Outline,
                ])
                .render(&mut scaler, glyph_id)
                .and_then(|image| {
                    if image.data.len() > 0 {
                        let index = self.data.len();
                        self.info.push(GlyphInfo {
                            size: Vec2::new(image.placement.width, image.placement.height),
                            placement_offset: Vec2::new(image.placement.left, image.placement.top),
                        });
                        self.data.push(image.data);
                        entry.insert(index);
                        Some(index)
                    } else {
                        None
                    }
                })
            }
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct GlyphInfo {
    pub size: Vec2<u32>,
    pub placement_offset: Vec2<i32>,
}
