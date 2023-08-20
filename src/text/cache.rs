use super::fonts::FontStyle;
use crate::util::vec2::Vec2;
use std::collections::{hash_map::Entry, HashMap};
use swash::{
    scale::{image::Content, Render, ScaleContext, Source, StrikeWith},
    FontRef, GlyphId,
};

#[derive(Default)]
pub struct FontCache {
    pub monochrome: Cached,
    pub emoji: Cached,
    scale_context: ScaleContext,
    lut: HashMap<CacheKey, Option<CacheValue>>,
}

impl FontCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.monochrome.clear();
        self.emoji.clear();
        self.lut.clear();
    }

    pub fn get(
        &mut self,
        font: FontRef,
        size: f32,
        glyph_id: GlyphId,
        style: FontStyle,
    ) -> Option<CacheValue> {
        let key = CacheKey { glyph_id, style };
        match self.lut.entry(key) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let mut scaler = self
                    .scale_context
                    .builder(font)
                    .size(size)
                    .hint(true)
                    .build();
                match Render::new(&[
                    Source::ColorOutline(0),
                    Source::ColorBitmap(StrikeWith::BestFit),
                    Source::Outline,
                ])
                .render(&mut scaler, glyph_id)
                {
                    Some(image) => {
                        let placement = image.placement;
                        let size = Vec2::new(placement.width, placement.height);
                        let (dst, kind) = if image.content == Content::Color {
                            (&mut self.emoji, GlyphKind::Emoji)
                        } else {
                            (&mut self.monochrome, GlyphKind::Monochrome)
                        };
                        if size.area() > 0 {
                            let index = dst.data.len();
                            let out = Some(CacheValue { index, kind });
                            entry.insert(out);
                            dst.data.push(image.data);
                            dst.size.push(size);
                            dst.offset.push(Vec2::new(placement.left, placement.top));
                            out
                        } else {
                            entry.insert(None);
                            None
                        }
                    }
                    None => {
                        entry.insert(None);
                        None
                    }
                }
            }
        }
    }
}

#[derive(Default)]
pub struct Cached {
    pub data: Vec<Vec<u8>>,
    pub size: Vec<Vec2<u32>>,
    pub offset: Vec<Vec2<i32>>,
}

impl Cached {
    pub fn clear(&mut self) {
        self.data.clear();
        self.size.clear();
        self.offset.clear();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CacheKey {
    glyph_id: GlyphId,
    style: FontStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CacheValue {
    pub index: usize,
    pub kind: GlyphKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlyphKind {
    Monochrome,
    Emoji,
}
