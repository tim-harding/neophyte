use crate::{ui::FontSize, util::vec2::Vec2};
use std::{fs, io, path::Path, sync::Arc};
use swash::{proxy::CharmapProxy, CacheKey, Charmap, FontRef};

/// Wrapper over a Swash font
#[derive(Debug, Clone)]
pub struct Font {
    data: Arc<Vec<u8>>,
    charmap: CharmapProxy,
    offset: u32,
    key: CacheKey,
    metrics: Metrics,
}

impl Font {
    /// Create a font from the given TTF or OTF font file
    pub fn from_file(
        path: impl AsRef<Path>,
        index: usize,
        size: FontSize,
    ) -> Result<Self, FontFromFileError> {
        let data = fs::read(path)?;
        Self::from_bytes(data, index, size).ok_or(FontFromFileError::Font)
    }

    /// Create a font from the given TTF or OTF font data
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

    /// Cached Swash charmap
    pub fn charmap(&self) -> Charmap {
        self.charmap.materialize(&self.as_ref())
    }

    /// Gets the underlying Swash font
    pub fn as_ref(&self) -> FontRef {
        // Unlike the FontRef constructors, this does not construct a new key,
        // enabling performance optimizations and caching mechanisms
        FontRef {
            data: &self.data,
            offset: self.offset,
            key: self.key,
        }
    }

    /// Update the font metrics with the given font size
    pub fn resize(&mut self, size: FontSize) {
        self.metrics = Metrics::new(self.as_ref(), size);
    }

    /// Cached Swash font metrics
    pub fn metrics(&self) -> Metrics {
        self.metrics
    }
}

/// Swash metrics in pixels with full precision
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    /// Multiplier from units per em to pixels
    pub scale_factor: f32,
    /// The width of a cell
    pub width: f32,
    /// The height of a cell
    pub em: f32,
    /// Distance from the baseline to the bottom of the alignment box.
    pub descent: f32,
    /// Distance from the baseline to the top of the alignment box.
    pub ascent: f32,
    /// Recommended additional spacing between lines.
    pub leading: f32,
    /// Distance from the baseline to the top of a typical English capital.
    pub cap_height: f32,
    /// Distance from the baseline to the top of the lowercase "x" or
    /// similar character.
    pub x_height: f32,
    /// Recommended distance from the baseline to the top of an underline
    /// stroke.
    pub underline_offset: f32,
    /// Recommended distance from the baseline to the top of a strikeout
    /// stroke.
    pub strikeout_offset: f32,
    /// Recommended thickness of an underline or strikeout stroke.
    pub stroke_size: f32,
}

impl Metrics {
    /// Convert to metrics in pixels
    pub fn into_pixels(self) -> MetricsPixels {
        self.into()
    }

    fn new(font: FontRef, size: FontSize) -> Self {
        let metrics = font.metrics(&[]);
        let (scale_factor, em) = match size {
            FontSize::Width(width) => {
                let scale_factor = width / metrics.max_width;
                let em = metrics.units_per_em as f32 * scale_factor;
                (scale_factor, em)
            }

            FontSize::Height(height) => {
                let scale_factor = height / metrics.units_per_em as f32;
                (scale_factor, height)
            }
        };

        let metrics = metrics.scale(em);
        Self {
            scale_factor,
            em,
            width: metrics.max_width,
            descent: metrics.descent,
            ascent: metrics.ascent,
            leading: metrics.leading,
            cap_height: metrics.cap_height,
            x_height: metrics.x_height,
            underline_offset: metrics.underline_offset,
            strikeout_offset: metrics.strikeout_offset,
            stroke_size: metrics.stroke_size,
        }
    }
}

/// Swash metrics in pixel units
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetricsPixels {
    /// Multiplier from units per em to pixels
    pub scale_factor: u32,
    /// The width of a cell
    pub width: u32,
    /// The height of a cell
    pub em: u32,
    /// Distance from the baseline to the bottom of the alignment box.
    pub descent: u32,
    /// Distance from the baseline to the top of the alignment box.
    pub ascent: u32,
    /// Recommended additional spacing between lines.
    pub leading: u32,
    /// Distance from the baseline to the top of a typical English capital.
    pub cap_height: u32,
    /// Distance from the baseline to the top of the lowercase "x" or
    /// similar character.
    pub x_height: u32,
    /// Recommended distance from the baseline to the top of an underline
    /// stroke.
    pub underline_offset: u32,
    /// Recommended distance from the baseline to the top of a strikeout
    /// stroke.
    pub strikeout_offset: u32,
    /// Recommended thickness of an underline or strikeout stroke.
    pub stroke_size: u32,
}

impl MetricsPixels {
    /// The dimensions of a grid cell with this font
    pub fn cell_size(&self) -> Vec2<u32> {
        Vec2::new(self.width, self.em + self.descent)
    }
}

impl From<Metrics> for MetricsPixels {
    fn from(metrics: Metrics) -> Self {
        Self {
            scale_factor: metrics.scale_factor.round() as u32,
            width: metrics.width.round() as u32,
            em: metrics.em.round() as u32,
            descent: metrics.descent.round() as u32,
            ascent: metrics.ascent.round() as u32,
            leading: metrics.leading.round() as u32,
            cap_height: metrics.cap_height.round() as u32,
            x_height: metrics.x_height.round() as u32,
            underline_offset: metrics.underline_offset.round() as u32,
            strikeout_offset: metrics.strikeout_offset.round() as u32,
            stroke_size: metrics.stroke_size.round() as u32,
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
