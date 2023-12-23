mod node;

use crate::util::vec2::Vec2;
use node::Node;
use swash::{
    scale::image::{Content, Image},
    zeno::Placement,
};

const DEFAULT_SIZE: u32 = 256;
const DEFAULT_PIXEL_COUNT: usize = DEFAULT_SIZE as usize * DEFAULT_SIZE as usize;
const DEFAULT_ROOT: Node = Node::new(Vec2::new(0, 0), Vec2::new(u32::MAX, u32::MAX));

// Algorithm borrowed from
// https://straypixels.net/texture-packing-for-fonts/
pub struct FontAtlas {
    /// x and y dimensions of the texture
    size: u32,
    /// Root of the glyph tree
    root: Node,
    /// Glyph atlas image data
    data: Vec<u8>,
    /// The number of color channels
    channels: u32,
}

impl FontAtlas {
    pub fn new(channels: u32) -> Self {
        Self {
            channels,
            size: DEFAULT_SIZE,
            root: DEFAULT_ROOT,
            data: vec![0u8; DEFAULT_PIXEL_COUNT * channels as usize],
        }
    }

    pub fn pack(&mut self, image: &Image) -> Vec2<u32> {
        match (image.content, self.channels) {
            (Content::Mask, 1) | (Content::Color | Content::SubpixelMask, 4) => {}
            _ => panic!("Wrong image content for atlas"),
        }

        let channels = self.channels as usize;
        let glyph_size = Vec2::new(image.placement.width as u32, image.placement.height as u32);
        let origin = loop {
            if let Some(node) = self.root.pack(glyph_size, self.size) {
                break node;
            } else {
                let old_size = self.size;
                self.size *= 2;
                let old = std::mem::replace(
                    &mut self.data,
                    vec![0u8; self.size as usize * self.size as usize * channels],
                );
                for (src, dst) in old
                    .chunks(old_size as usize * channels)
                    .zip(self.data.chunks_mut(self.size as usize * channels))
                {
                    for (src, dst) in src.into_iter().zip(dst.into_iter()) {
                        *dst = *src;
                    }
                }
            }
        };

        for (src, dst) in image
            .data
            .chunks(image.placement.width as usize * channels)
            .zip(
                self.data
                    .chunks_mut(self.size as usize * channels)
                    .skip(origin.y as usize),
            )
        {
            for (src, dst) in src
                .into_iter()
                .zip(dst.into_iter().skip(origin.x as usize * channels))
            {
                *dst = *src;
            }
        }

        origin
    }

    pub fn clear(&mut self) {
        self.size = DEFAULT_SIZE;
        self.root = DEFAULT_ROOT;
        self.data.clear();
        self.data
            .resize(DEFAULT_PIXEL_COUNT * self.channels as usize, 0);
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn size(&self) -> u32 {
        self.size
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PackedGlyph {
    pub origin: Vec2<u32>,
    pub placement: Placement,
}
