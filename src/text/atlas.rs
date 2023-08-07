#![allow(unused)]

// TODO: u16 overflow handling. What should be the maximum texture size?

use std::collections::HashMap;

use swash::{
    scale::{image::Image, Render, ScaleContext, Source, StrikeWith},
    zeno::Placement,
    FontRef, GlyphId,
};

use crate::util::vec2::Vec2;

// Algorithm borrowed from
// https://straypixels.net/texture-packing-for-fonts/
pub struct FontAtlas {
    /// x and y dimensions of the texture
    size: u16,
    /// Root of the glyph tree
    root: Node,
    /// Glyph atlas image data
    data: Vec<u8>,
    /// A lookup table from glyphs to their rendering info.
    lut: HashMap<GlyphId, PackedGlyph>,
}

impl FontAtlas {
    pub fn new() -> Self {
        const DEFAULT_SIZE: u16 = 256;
        Self {
            size: DEFAULT_SIZE,
            root: Node::new(Vec2::new(0, 0), Vec2::new(u16::MAX, u16::MAX)),
            data: vec![0u8; DEFAULT_SIZE as usize * DEFAULT_SIZE as usize],
            lut: HashMap::default(),
        }
    }

    pub fn from_font(font: FontRef, size: f32) -> Self {
        let mut glyphs = vec![];
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
                glyphs.push((id, image));
            }
        });
        glyphs.sort_unstable_by(|(_, l), (_, r)| {
            let size = |g: &Image| g.placement.width * g.placement.height;
            size(r).cmp(&size(l))
        });
        let mut this = Self::new();
        for (id, image) in glyphs {
            this.pack(id, &image);
        }
        this
    }

    pub fn pack(&mut self, id: GlyphId, image: &Image) -> Pack {
        let mut resized = false;
        let glyph_size = Vec2::new(image.placement.width as u16, image.placement.height as u16);
        let origin = if let Some(node) = self.root.pack(glyph_size, self.size) {
            node
        } else {
            resized = true;
            let old_size = self.size;
            self.size *= 2;
            let old = std::mem::take(&mut self.data);
            self.data = vec![0u8; self.size as usize * self.size as usize];
            for (src, dst) in old
                .chunks(old_size as usize)
                .zip(self.data.chunks_mut(self.size as usize))
            {
                for (src, dst) in src.into_iter().zip(dst.into_iter()) {
                    *dst = *src;
                }
            }
            self.root.pack(glyph_size, self.size).unwrap()
        };

        for (src, dst) in image.data.chunks(image.placement.width as usize).zip(
            self.data
                .chunks_mut(self.size as usize)
                .skip(origin.y as usize),
        ) {
            for (src, dst) in src.into_iter().zip(dst.into_iter().skip(origin.x as usize)) {
                *dst = *src;
            }
        }

        self.lut.insert(
            id,
            PackedGlyph {
                origin,
                placement: image.placement,
            },
        );

        Pack { resized, origin }
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn size(&self) -> u16 {
        self.size
    }

    pub fn get(&self, id: GlyphId) -> Option<&PackedGlyph> {
        self.lut.get(&id)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PackedGlyph {
    origin: Vec2<u16>,
    placement: Placement,
}

pub struct Pack {
    resized: bool,
    origin: Vec2<u16>,
}

struct Node {
    origin: Vec2<u16>,
    size: Vec2<u16>,
    is_filled: bool,
    children: Option<(Box<Node>, Box<Node>)>,
}

impl Node {
    pub fn new(origin: Vec2<u16>, size: Vec2<u16>) -> Self {
        Self {
            origin,
            size,
            is_filled: false,
            children: None,
        }
    }

    pub fn pack(&mut self, size: Vec2<u16>, texture_size: u16) -> Option<Vec2<u16>> {
        if self.is_filled {
            return None;
        } else if let Some(children) = self.children.as_mut() {
            children
                .0
                .pack(size, texture_size)
                .or_else(|| children.1.pack(size, texture_size))
        } else {
            let real_size = {
                let mut real_size = self.size;
                if self.origin.x + self.size.x == u16::MAX {
                    real_size.x = texture_size - self.origin.x;
                }
                if self.origin.y + self.size.y == u16::MAX {
                    real_size.y = texture_size - self.origin.y;
                }
                real_size
            };

            if self.size == size {
                self.is_filled = true;
                Some(self.origin)
            } else if real_size.x < size.x || real_size.y < size.y {
                None
            } else {
                let remainder = real_size - size;
                let vertical_split = if remainder == Vec2::new(0, 0) {
                    // If we are going to the edge of the texture, split
                    // according to the glyph dimensions instead
                    self.size.x < self.size.y
                } else {
                    remainder.x < remainder.y
                };

                self.children = Some(if vertical_split {
                    (
                        Box::new(Node::new(self.origin, Vec2::new(self.size.x, size.y))),
                        Box::new(Node::new(
                            Vec2::new(self.origin.x, self.origin.y + size.y),
                            Vec2::new(self.size.x, self.size.y - size.y),
                        )),
                    )
                } else {
                    (
                        Box::new(Node::new(self.origin, Vec2::new(size.x, self.size.y))),
                        Box::new(Node::new(
                            Vec2::new(self.origin.x + size.x, self.origin.y),
                            Vec2::new(self.size.x - size.x, self.size.y),
                        )),
                    )
                });
                self.children.as_mut().unwrap().0.pack(size, texture_size)
            }
        }
    }
}
