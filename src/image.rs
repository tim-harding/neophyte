use png::Encoder;
use std::{
    fs::{self, File},
    io::{self, BufWriter},
    path::Path,
};
use swash::{CacheKey, FontRef};

#[allow(unused)]
pub fn render() {
    let path = Path::new(r"/home/tim/temp.png");
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = Encoder::new(w, 512, 256);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut w = encoder.write_header().unwrap();

    let mut data = [255u8; 256 * 512 * 3];
    for x in 0..512 {
        for y in 0..256 {
            let p = (y * 512 + x) * 3;
            data[p] = x as u8;
            data[p + 1] = y as u8;
        }
    }

    let font_path = Path::new(r"/usr/share/fonts/TTF/CaskaydiaCoveNerdFont-Regular.ttf");
    let font = Font::from_file(&font_path, 0).unwrap();

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
