use crate::util::vec2::Vec2;
use std::{fs, io, path::Path, sync::Arc};
use swash::{proxy::CharmapProxy, CacheKey, Charmap, FontRef};

#[derive(Clone)]
pub struct Font {
    data: Arc<Vec<u8>>,
    charmap: CharmapProxy,
    offset: u32,
    key: CacheKey,
    metrics: swash::Metrics,
}

impl Font {
    pub fn from_file(path: impl AsRef<Path>, index: usize) -> Result<Self, FontFromFileError> {
        let data = fs::read(path)?;
        Self::from_bytes(data, index).ok_or(FontFromFileError::Font)
    }

    pub fn from_bytes(data: Vec<u8>, index: usize) -> Option<Self> {
        let font = FontRef::from_index(&data, index)?;
        Some(Self {
            offset: font.offset,
            metrics: font.metrics(&[]),
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

    pub fn metrics(&self, width: u32) -> Metrics {
        let metrics = self.metrics;
        let scale_factor = width as f32 / metrics.average_width;
        let em = metrics.units_per_em as f32 * scale_factor;
        let em_px = em.ceil() as u32;
        let descent = metrics.descent * scale_factor;
        let descent_px = descent.ceil() as u32;
        let cell_size_px = Vec2::new(width, em_px + descent_px);
        Metrics {
            scale_factor,
            em,
            em_px,
            descent,
            descent_px,
            cell_size_px,
        }
    }
}

pub struct Metrics {
    pub scale_factor: f32,
    pub em: f32,
    pub em_px: u32,
    pub descent: f32,
    pub descent_px: u32,
    pub cell_size_px: Vec2<u32>,
}

#[derive(Debug, thiserror::Error)]
pub enum FontFromFileError {
    #[error("IO: {0}")]
    Io(#[from] io::Error),
    #[error("Invalid font data or font index out of range")]
    Font,
}
