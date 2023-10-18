use super::font::{Font, Metrics};
use crate::{
    ui::{FontSize, FontsSetting},
    util::vec2::Vec2,
};
use font_loader::system_fonts::{self, FontPropertyBuilder};
use std::sync::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// A shared handle to fonts with tracking for whether the render thread needs
/// to reset its glyph cache.
#[derive(Default, Debug)]
pub struct FontsHandle {
    needs_glyph_cache_reset: Mutex<bool>,
    inner: RwLock<Fonts>,
}

impl FontsHandle {
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets a write handle to the fonts and flags that the render thread will
    /// need to reset its glyph cache
    pub fn write(&self) -> RwLockWriteGuard<Fonts> {
        let lock = self.inner.write().unwrap();
        *self.needs_glyph_cache_reset.lock().unwrap() = true;
        lock
    }

    /// Gets a read handle to the fonts
    pub fn read(&self) -> RwLockReadGuard<Fonts> {
        self.inner.read().unwrap()
    }

    /// Get a read handle to the fonts and consume the glyph cache reset flag.
    pub fn read_and_take_cache_reset(&self) -> (RwLockReadGuard<Fonts>, bool) {
        let inner_lock = self.inner.read().unwrap();
        let mut needs_glyph_cache_reset_lock = self.needs_glyph_cache_reset.lock().unwrap();
        let needs_glyph_cache_reset = *needs_glyph_cache_reset_lock;
        *needs_glyph_cache_reset_lock = false;
        (inner_lock, needs_glyph_cache_reset)
    }
}

/// Loaded fonts
#[derive(Debug, Clone)]
pub struct Fonts {
    fonts: Vec<FontVariants>,
    fallback: Font,
}

impl Default for Fonts {
    fn default() -> Self {
        Self::new()
    }
}

impl Fonts {
    pub fn new() -> Self {
        Self {
            fonts: vec![],
            fallback: get(FontPropertyBuilder::new().monospace(), FontSize::default()).unwrap(),
        }
    }

    pub fn set_font_size(&mut self, size: FontSize) {
        for font in self.fonts.iter_mut() {
            font.resize(size);
        }
    }

    pub fn set_fonts(&mut self, setting: &FontsSetting) {
        let mut old = std::mem::take(&mut self.fonts);
        self.fonts = setting
            .fonts
            .iter()
            .map(|name| {
                if let Some(i) = old.iter().position(|old| &old.name == name) {
                    let mut existing = old.swap_remove(i);
                    existing.resize(setting.size);
                    existing
                } else {
                    FontVariants::with_name(name.clone(), setting.size)
                }
            })
            .collect();
    }

    pub fn iter(&self) -> impl Iterator<Item = &FontVariants> {
        self.fonts.iter()
    }

    /// Get the metrics for the first loaded font
    pub fn metrics(&self) -> Metrics {
        self.fonts
            .iter()
            .find_map(|variants| variants.metrics())
            .unwrap_or(self.fallback.metrics())
    }

    /// Get the cell size of the first loaded font
    pub fn cell_size(&self) -> Vec2<u32> {
        self.metrics().into_pixels().cell_size()
    }
}

#[derive(Clone, Debug)]
pub struct FontVariants {
    /// The font name
    pub name: String,
    /// If regular variant of the font, if available
    pub regular: Option<Font>,
    /// If bold variant of the font, if available
    pub bold: Option<Font>,
    /// If italic variant of the font, if available
    pub italic: Option<Font>,
    /// If bold italic variant of the font, if available
    pub bold_italic: Option<Font>,
}

impl FontVariants {
    /// Attempt to load the system font with the given name
    pub fn with_name(name: String, size: FontSize) -> Self {
        let builder = || FontPropertyBuilder::new().family(&name);
        Self {
            regular: get(builder(), size),
            bold: get(builder().bold(), size),
            italic: get(builder().italic(), size),
            bold_italic: get(builder().bold().italic(), size),
            name,
        }
    }

    /// Gets the variant with the given style
    pub fn style(&self, style: FontStyle) -> Option<&Font> {
        match style {
            FontStyle::Regular => self.regular.as_ref(),
            FontStyle::Bold => self.bold.as_ref(),
            FontStyle::Italic => self.italic.as_ref(),
            FontStyle::BoldItalic => self.bold_italic.as_ref(),
        }
    }

    /// Recalculate the font metrics for the loaded variants
    pub fn resize(&mut self, size: FontSize) {
        for font in self.iter_mut() {
            font.resize(size);
        }
    }

    /// An iterator over the loaded variants
    pub fn iter(&self) -> impl Iterator<Item = &Font> {
        [&self.regular, &self.bold, &self.italic, &self.bold_italic]
            .into_iter()
            .filter_map(|font| font.as_ref())
    }

    /// An iterator over the loaded variants
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Font> {
        [
            &mut self.regular,
            &mut self.bold,
            &mut self.italic,
            &mut self.bold_italic,
        ]
        .into_iter()
        .filter_map(|font| font.as_mut())
    }

    /// Metrics for the first loaded variant
    pub fn metrics(&self) -> Option<Metrics> {
        self.iter().map(|font| font.metrics()).next()
    }
}

fn get(builder: FontPropertyBuilder, size: FontSize) -> Option<Font> {
    system_fonts::get(&builder.build())
        .and_then(|(data, index)| Font::from_bytes(data, index as usize, size))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FontStyle {
    #[default]
    Regular,
    Bold,
    Italic,
    BoldItalic,
}

impl FontStyle {
    pub fn new(bold: bool, italic: bool) -> Self {
        use FontStyle::*;
        match (bold, italic) {
            (true, true) => BoldItalic,
            (true, false) => Bold,
            (false, true) => Italic,
            (false, false) => Regular,
        }
    }
}
