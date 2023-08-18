use crate::util::vec2::Vec2;
use std::collections::{hash_map::Entry, HashMap};
use swash::{
    scale::{Render, ScaleContext, Source, StrikeWith},
    FontRef, GlyphId,
};

use super::fonts::FontStyle;

#[derive(Default)]
pub struct FontCache {
    pub data: Vec<Vec<u8>>,
    pub size: Vec<Vec2<u32>>,
    pub offset: Vec<Vec2<i32>>,
    scale_context: ScaleContext,
    lut: HashMap<CacheKey, Option<usize>>,
}

impl FontCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.lut.clear();
    }

    pub fn get(
        &mut self,
        font: FontRef,
        size: f32,
        glyph_id: GlyphId,
        style: FontStyle,
    ) -> Option<usize> {
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
                        if size.area() > 0 {
                            let index = self.data.len();
                            entry.insert(Some(index));
                            self.data.push(image.data);
                            self.size.push(size);
                            self.offset.push(Vec2::new(placement.left, placement.top));
                            Some(index)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CacheKey {
    glyph_id: GlyphId,
    style: FontStyle,
}
