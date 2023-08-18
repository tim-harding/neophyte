use std::{fs, io, path::Path, sync::Arc};
use swash::{proxy::CharmapProxy, CacheKey, Charmap, FontRef};

#[derive(Clone)]
pub struct Font {
    data: Arc<Vec<u8>>,
    charmap: CharmapProxy,
    offset: u32,
    key: CacheKey,
}

impl Font {
    pub fn from_file(path: impl AsRef<Path>, index: usize) -> Result<Self, FontFromFileError> {
        let data = fs::read(path)?;
        let font = FontRef::from_index(&data, index).ok_or(FontFromFileError::Font)?;
        Ok(Self {
            offset: font.offset,
            charmap: font.charmap().proxy(),
            key: font.key,
            data: Arc::new(data),
        })
    }

    pub fn from_bytes(data: Vec<u8>, index: usize) -> Option<Self> {
        let font = FontRef::from_index(&data, index)?;
        Some(Self {
            offset: font.offset,
            charmap: font.charmap().proxy(),
            key: font.key,
            data: Arc::new(data),
        })
    }

    pub fn charmap(&self) -> Charmap {
        self.charmap.materialize(&self.as_ref())
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
