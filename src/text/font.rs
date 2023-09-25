use crate::{ui::FontSize, util::vec2::Vec2};
use std::{fs, io, path::Path, sync::Arc};
use swash::{proxy::CharmapProxy, CacheKey, Charmap, FontRef};

#[derive(Clone)]
pub struct Font {
    data: Arc<Vec<u8>>,
    charmap: CharmapProxy,
    offset: u32,
    key: CacheKey,
    metrics: Metrics,
}

impl Font {
    pub fn from_file(
        path: impl AsRef<Path>,
        index: usize,
        size: FontSize,
    ) -> Result<Self, FontFromFileError> {
        let data = fs::read(path)?;
        Self::from_bytes(data, index, size).ok_or(FontFromFileError::Font)
    }

    pub fn from_bytes(data: Vec<u8>, index: usize, size: FontSize) -> Option<Self> {
        let font = FontRef::from_index(&data, index)?;
        Some(Self {
            offset: font.offset,
            metrics: Metrics::new(font, size),
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

    pub fn resize(&mut self, size: FontSize) {
        self.metrics = Metrics::new(self.as_ref(), size);
    }

    pub fn metrics(&self) -> Metrics {
        self.metrics
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    pub scale_factor: f32,
    pub width: f32,
    pub em: f32,
    pub descent: f32,
}

impl Metrics {
    pub fn into_pixels(self) -> MetricsPixels {
        self.into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetricsPixels {
    pub width: u32,
    pub em: u32,
    pub descent: u32,
}

impl MetricsPixels {
    pub fn cell_size(&self) -> Vec2<u32> {
        Vec2::new(self.width, self.em + self.descent)
    }
}

impl From<Metrics> for MetricsPixels {
    fn from(metrics: Metrics) -> Self {
        Self {
            width: metrics.width.ceil() as u32,
            em: metrics.em.ceil() as u32,
            descent: metrics.descent.ceil() as u32,
        }
    }
}

impl Metrics {
    fn new(font: FontRef, size: FontSize) -> Self {
        let metrics = font.metrics(&[]);
        match size {
            FontSize::Width(width) => {
                let scale_factor = width as f32 / metrics.max_width;
                Self {
                    scale_factor,
                    width: width as f32,
                    em: metrics.units_per_em as f32 * scale_factor,
                    descent: metrics.descent * scale_factor,
                }
            }

            FontSize::Height(height) => {
                let metrics = metrics.scale(height as f32);
                Self {
                    scale_factor: height as f32 / metrics.units_per_em as f32,
                    width: metrics.max_width,
                    em: height as f32,
                    descent: metrics.descent,
                }
            }
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
