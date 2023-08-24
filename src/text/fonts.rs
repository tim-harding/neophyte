use super::font::Font;
use font_loader::system_fonts::{self, FontPropertyBuilder};

pub struct Fonts {
    size: u32,
    fonts: Vec<FontInfo>,
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
            size: 16,
            fonts: vec![],
            fallback: get(FontPropertyBuilder::new().monospace()).unwrap(),
        }
    }

    pub fn reload(&mut self, font_names: &[String], size: u32) {
        self.size = size;
        let mut old = std::mem::take(&mut self.fonts);
        self.fonts = font_names
            .iter()
            .map(|name| {
                if let Some(i) = old.iter().position(|old| &old.name == name) {
                    old.swap_remove(i)
                } else {
                    FontInfo::with_name(name.clone())
                }
            })
            .collect();
    }

    pub fn with_style(&self, style: FontStyle) -> &Font {
        self.iter()
            .find_map(|font_info| font_info.style(style))
            .unwrap_or(&self.fallback)
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn iter(&self) -> impl Iterator<Item = &FontInfo> {
        self.fonts.iter()
    }
}

pub struct FontInfo {
    pub name: String,
    pub regular: Option<Font>,
    pub bold: Option<Font>,
    pub italic: Option<Font>,
    pub bold_italic: Option<Font>,
}

impl FontInfo {
    pub fn with_name(name: String) -> Self {
        let builder = || FontPropertyBuilder::new().family(&name);
        Self {
            regular: get(builder()),
            bold: get(builder().bold()),
            italic: get(builder().italic()),
            bold_italic: get(builder().bold().italic()),
            name,
        }
    }

    pub fn style_or_regular(&self, style: FontStyle) -> Option<&Font> {
        self.style(style).or(self.regular.as_ref())
    }

    pub fn style(&self, style: FontStyle) -> Option<&Font> {
        match style {
            FontStyle::Regular => self.regular.as_ref(),
            FontStyle::Bold => self.bold.as_ref(),
            FontStyle::Italic => self.italic.as_ref(),
            FontStyle::BoldItalic => self.bold_italic.as_ref(),
        }
    }
}

fn get(builder: FontPropertyBuilder) -> Option<Font> {
    system_fonts::get(&builder.build())
        .and_then(|(data, index)| Font::from_bytes(data, index as usize))
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
