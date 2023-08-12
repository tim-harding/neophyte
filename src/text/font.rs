use std::{fs, io, path::Path, sync::Arc};
use swash::{CacheKey, FontRef};

#[derive(Clone)]
pub struct Font {
    data: Arc<Vec<u8>>,
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
            data: Arc::new(data),
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

// TODO: Store and materialize metrics
pub fn metrics(font: FontRef, size: f32) -> Metrics {
    let metrics = font.metrics(&[]).scale(size);
    Metrics {
        advance: metrics.average_width,
        ascent: metrics.ascent,
        descent: metrics.descent,
        leading: metrics.leading,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Metrics {
    pub advance: f32,
    pub ascent: f32,
    pub descent: f32,
    pub leading: f32,
}

impl Metrics {
    pub fn cell_height(&self) -> u32 {
        (self.ascent + self.descent + self.leading).ceil() as u32
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FontFromFileError {
    #[error("IO: {0}")]
    Io(#[from] io::Error),
    #[error("Invalid font data or font index out of range")]
    Font,
}
