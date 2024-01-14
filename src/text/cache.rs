use super::{atlas::FontAtlas, fonts::FontStyle};
use crate::util::vec2::Vec2;
use bytemuck::{Pod, Zeroable};
use std::collections::{hash_map::Entry, HashMap};
use swash::{
    scale::{image::Content, Render, ScaleContext, Source, StrikeWith},
    FontRef, GlyphId, Setting,
};

/// A cache of font glyphs
pub struct FontCache {
    pub monochrome: Cached,
    pub emoji: Cached,
    /// Given a glyph, a font, and a font style, get the corresponding cache
    /// entry. A value of None indicates that we already tried to convert the
    /// given cache key and failed so we should not try again.
    lut: HashMap<CacheKey, Option<CacheValue>>,
    scale_context: ScaleContext,
}

impl Default for FontCache {
    fn default() -> Self {
        Self::new()
    }
}

impl FontCache {
    pub fn new() -> Self {
        Self {
            monochrome: Cached::new(1),
            emoji: Cached::new(4),
            lut: HashMap::new(),
            scale_context: ScaleContext::default(),
        }
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
                    .variations(variations.iter().cloned())
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
                            let (cached, kind) = match image.content {
                                Content::Mask => (&mut self.monochrome, GlyphKind::Monochrome),
                                Content::SubpixelMask | Content::Color => {
                                    (&mut self.emoji, GlyphKind::Emoji)
                                }
                            };

                            let index = cached.info.len();
                            let out = Some(CacheValue { index, kind });
                            entry.insert(out);
                            cached.revision += 1;
                            let origin = cached.atlas.pack(&image);
                            cached.info.push(GlyphInfo {
                                size: size.try_cast().unwrap(),
                                offset: Vec2::new(placement.left, placement.top) * Vec2::new(1, -1),
                                origin: origin.try_cast().unwrap(),
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

pub struct Cached {
    pub atlas: FontAtlas,
    pub info: Vec<GlyphInfo>,
    pub revision: u32,
}

impl Cached {
    pub fn new(channels: u32) -> Self {
        Self {
            atlas: FontAtlas::new(channels),
            info: vec![],
            revision: 0,
        }
    }

    pub fn clear(&mut self) {
        self.atlas.clear();
        self.info.clear();
        self.revision += 1;
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable)]
pub struct GlyphInfo {
    /// The size of the glyph in pixels
    pub size: Vec2<i32>,
    /// The amount to offset the glyph in pixels
    pub offset: Vec2<i32>,
    /// The upper-left corner of the texture in the glyph atlas
    pub origin: Vec2<i32>,
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
