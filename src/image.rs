use crate::util::vec2::Vec2;
use png::{ColorType, Encoder};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, BufWriter},
    path::Path,
};
use swash::{
    scale::{image::Image, Render, ScaleContext, Source, StrikeWith},
    shape::{Direction, ShapeContext},
    text::Script,
    zeno::Placement,
    CacheKey, FontRef, GlyphId,
};

// TODO: Subpixel rendering
// TODO: Font atlas
// TODO: Subpixel offsets
// TODO: Emoji

#[allow(unused)]
pub fn render() {
    let cascadia = Font::from_file("/usr/share/fonts/OTF/CascadiaCode-Regular.otf", 0).unwrap();
    let noto = Font::from_file("/usr/share/fonts/noto/NotoColorEmoji.ttf", 0).unwrap();
    let mut shape_context = ShapeContext::new();
    let mut shaper = shape_context
        .builder(cascadia.as_ref())
        .script(Script::Arabic)
        .direction(Direction::RightToLeft)
        .size(24.0)
        .build();
    shaper.add_str("Things and stuff");
    let mut scale_context = ScaleContext::new();
    let mut scaler = scale_context
        .builder(cascadia.as_ref())
        .size(24.0)
        .hint(true)
        .build();
    let mut cache = FontAtlas::from_font(cascadia.as_ref(), 24.0);
    const WIDTH: usize = 600;
    const HEIGHT: usize = 40;
    let mut data = [0u8; WIDTH * HEIGHT];
    let mut x_offset = 0;
    // shaper.shape_with(|cluster| {
    //     for glyph in cluster.glyphs {
    //         let image = cache.glyph(glyph.id).unwrap();
    //         for y in 0..image.placement.height as i32 {
    //             for x in 0..image.placement.width as i32 {
    //                 let dst_y = y - image.placement.top - glyph.y as i32 + 24;
    //                 let dst_x = x + x_offset + image.placement.left + glyph.x as i32;
    //                 if dst_y < 0 || dst_x < 0 {
    //                     continue;
    //                 }
    //                 let dst_i = dst_y as usize * WIDTH + dst_x as usize;
    //                 let src_i = y as usize * image.placement.width as usize + x as usize;
    //                 data[dst_i] = data[dst_i].saturating_add(image.data[src_i]);
    //             }
    //         }
    //         x_offset += glyph.advance.floor() as i32;
    //     }
    // });
    write_png(
        "/home/tim/temp.png",
        cache.size() as u32,
        cache.size() as u32,
        ColorType::Grayscale,
        cache.data(),
    );
}

// TODO: u16 overflow handling. What should be the maximum texture size?

// Algorithm borrowed from
// https://straypixels.net/texture-packing-for-fonts/
struct FontAtlas {
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
            glyphs.push((id, image));
        });
        glyphs.sort_unstable_by(|(_, l), (_, r)| {
            let size = |g: &Image| g.placement.width * g.placement.height;
            size(l).cmp(&size(r))
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

        for (src, dst) in image
            .data
            .chunks(image.placement.width as usize)
            .skip(origin.y as usize)
            .zip(self.data.chunks_mut(self.size as usize))
        {
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
                    real_size.x = self.size.x - self.origin.x;
                }
                if self.origin.y + self.size.y == u16::MAX {
                    real_size.y = self.size.y - self.origin.y;
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

                self.children = Some((
                    Box::new(Node::new(self.origin, self.size)),
                    Box::new(if vertical_split {
                        Node::new(
                            Vec2::new(self.origin.x, self.origin.y + size.y),
                            Vec2::new(self.size.x, self.size.y - size.y),
                        )
                    } else {
                        Node::new(
                            Vec2::new(self.origin.x + size.x, self.origin.y),
                            Vec2::new(self.size.x - size.x, self.size.y),
                        )
                    }),
                ));
                self.children.as_mut().unwrap().0.pack(size, texture_size)
            }
        }
    }
}

fn write_png(
    path: impl AsRef<Path>,
    width: u32,
    height: u32,
    color: ColorType,
    pixels: &[u8],
) -> Result<(), WritePngError> {
    let file = File::create(path)?;
    let ref mut w = BufWriter::new(file);
    let mut encoder = Encoder::new(w, width, height);
    encoder.set_color(color);
    encoder.set_depth(png::BitDepth::Eight);
    let mut w = encoder.write_header().unwrap();
    w.write_image_data(pixels)?;
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum WritePngError {
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("PNG: {0}")]
    Png(#[from] png::EncodingError),
}

pub struct Font {
    data: Vec<u8>,
    offset: u32,
    key: CacheKey,
}

impl Font {
    pub fn from_file(path: impl AsRef<Path>, index: usize) -> Result<Self, FontFromFileError> {
        let data = fs::read(path)?;
        let font = FontRef::from_index(&data, index).ok_or(FontFromFileError::Font)?;
        Ok(Self {
            offset: font.offset,
            key: font.key,
            data,
        })
    }

    pub fn as_ref(&self) -> FontRef {
        // Unlike the FontRef constructors, this does not construct a new key,
        // enabling performance optimizations and caching mechanisms
        FontRef {
            data: &self.data,
            offset: self.offset,
            key: self.key,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FontFromFileError {
    #[error("IO: {0}")]
    Io(#[from] io::Error),
    #[error("Invalid font data or font index out of range")]
    Font,
}
