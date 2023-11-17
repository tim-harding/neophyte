use super::fonts::FontStyle;
use crate::util::vec2::Vec2;
use bytemuck::{Pod, Zeroable};
use std::collections::{hash_map::Entry, HashMap};
use swash::{
    scale::{image::Content, Render, ScaleContext, Source, StrikeWith},
    FontRef, GlyphId, Setting,
};

/// A cache of font glyphs
#[derive(Default)]
pub struct FontCache {
    /// Cached monochrome glyphs.
    pub monochrome: Cached,
    /// Cached color glyphs
    pub emoji: Cached,
    /// Given a glyph, a font, and a font style, get the corresponding cache
    /// entry. A value of None indicates that we already tried to convert the
    /// given cache key and failed so we should not try again.
    lut: HashMap<CacheKey, Option<CacheValue>>,
    scale_context: ScaleContext,
}

impl FontCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Remove all cached entries
    pub fn clear(&mut self) {
        self.monochrome.clear();
        self.emoji.clear();
        self.lut.clear();
    }

    /// Get an existing cache entry or attempt to create it if it does not
    /// exist.
    pub fn get(
        &mut self,
        font: FontRef,
        variations: &[Setting<f32>],
        size: f32,
        glyph_id: GlyphId,
        style: FontStyle,
        font_index: usize,
    ) -> Option<CacheValue> {
        let key = CacheKey {
            glyph_id,
            style,
            font_index,
        };
        match self.lut.entry(key) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let mut scaler = self
                    .scale_context
                    .builder(font)
                    .size(size)
                    .hint(true)
                    .variations(variations.into_iter().cloned())
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
                            dst.info.push(GlyphInfo {
                                size,
                                offset: Vec2::new(placement.left, placement.top) * Vec2::new(1, -1),
                            });
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

/// Cached glyphs. The image data and glyph information are stored separately so
/// that no data conversion is needed to upload the textures glyph information
/// to GPU buffers. The index from a cache value can be used to index both of
/// these arrays.
#[derive(Debug, Default)]
pub struct Cached {
    /// The image data for the glyphs
    pub data: Vec<Vec<u8>>,
    /// Information about the size and placement of glyphs
    pub info: Vec<GlyphInfo>,
}

impl Cached {
    /// Clear all cached glyphs.
    pub fn clear(&mut self) {
        self.data.clear();
        self.info.clear();
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable)]
pub struct GlyphInfo {
    /// The size of the glyph in pixels
    pub size: Vec2<u32>,
    /// The amount to offset the glyph in pixels
    pub offset: Vec2<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CacheKey {
    glyph_id: GlyphId,
    style: FontStyle,
    font_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CacheValue {
    pub index: usize,
    pub kind: GlyphKind,
}

/// Indicates whether the cache value should be used to index the monochrome or
/// emoji cache entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlyphKind {
    Monochrome,
    Emoji,
}
