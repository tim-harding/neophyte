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

pub fn advance(font: FontRef, size: f32) -> f32 {
    let metrics = font.metrics(&[]).linear_scale(size);
    metrics.average_width / metrics.units_per_em as f32
}

#[derive(Debug, thiserror::Error)]
pub enum FontFromFileError {
    #[error("IO: {0}")]
    Io(#[from] io::Error),
    #[error("Invalid font data or font index out of range")]
    Font,
}
