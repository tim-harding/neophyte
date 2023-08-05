use png::Encoder;
use std::{
    fs::{self, File},
    io::{self, BufWriter},
    path::Path,
};
use swash::{
    scale::{Render, ScaleContext, Source, StrikeWith},
    shape::{Direction, ShapeContext},
    text::Script,
    CacheKey, FontRef,
};

#[allow(unused)]
pub fn render() {
    let path = Path::new(r"/home/tim/temp.png");
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let font_path = Path::new(r"/usr/share/fonts/OTF/CascadiaCode-Regular.otf");
    let font = Font::from_file(&font_path, 0).unwrap();
    let mut shape_context = ShapeContext::new();
    let mut shaper = shape_context
        .builder(font.as_ref())
        .script(Script::Arabic)
        .direction(Direction::RightToLeft)
        .size(24.0)
        .build();
    shaper.add_str("-> - >");
    let mut scale_context = ScaleContext::new();
    let mut scaler = scale_context
        .builder(font.as_ref())
        .size(24.0)
        .hint(true)
        .build();
    const WIDTH: usize = 600;
    const HEIGHT: usize = 40;
    let mut data = [0u8; WIDTH * HEIGHT];
    let mut x_offset = 0;
    shaper.shape_with(|cluster| {
        for glyph in cluster.glyphs {
            let image = Render::new(&[
                Source::ColorOutline(0),
                Source::ColorBitmap(StrikeWith::BestFit),
                Source::Outline,
            ])
            .render(&mut scaler, glyph.id)
            .unwrap();
            for y in 0..image.placement.height as i32 {
                for x in 0..image.placement.width as i32 {
                    let dst_y = y - image.placement.top - glyph.y as i32 + 24;
                    let dst_x = x + x_offset + image.placement.left + glyph.x as i32;
                    if dst_y < 0 || dst_x < 0 {
                        continue;
                    }
                    let dst_i = dst_y as usize * WIDTH + dst_x as usize;
                    let src_i = y as usize * image.placement.width as usize + x as usize;
                    data[dst_i] = data[dst_i].saturating_add(image.data[src_i]);
                }
            }
            x_offset += glyph.advance.floor() as i32;
        }
    });

    let mut encoder = Encoder::new(w, WIDTH as u32, HEIGHT as u32);
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);
    let mut w = encoder.write_header().unwrap();
    w.write_image_data(&data).unwrap();
}

pub struct Font {
    data: Vec<u8>,
    offset: u32,
    key: CacheKey,
}

impl Font {
    pub fn from_file(path: &Path, index: usize) -> Result<Self, FontFromFileError> {
        let data = fs::read(&path)?;
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
