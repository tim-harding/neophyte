mod node;

use crate::util::vec2::Vec2;
use node::Node;
use std::collections::HashMap;
use swash::{scale::image::Image, zeno::Placement, GlyphId};

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

    pub fn pack(&mut self, id: GlyphId, image: &Image) -> Vec2<u16> {
        let glyph_size = Vec2::new(image.placement.width as u16, image.placement.height as u16);
        let origin = loop {
            if let Some(node) = self.root.pack(glyph_size, self.size) {
                break node;
            } else {
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
            }
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

        origin
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
    pub origin: Vec2<u16>,
    pub placement: Placement,
}
