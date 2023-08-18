use super::font::Font;
use font_loader::system_fonts::{self, FontPropertyBuilder};

pub struct Fonts {
    size: u32,
    fonts: Vec<FontInfo>,
    fallback: FontInfo,
}

impl Fonts {
    pub fn new() -> Self {
        Self {
            size: 16,
            fonts: vec![],
            fallback: FontInfo::fallback(),
        }
    }

    pub fn reload(&mut self, font_names: Vec<String>, size: u32) {
        self.size = size;
        let mut old = std::mem::take(&mut self.fonts);
        self.fonts = font_names
            .into_iter()
            .map(|name| {
                if let Some(i) = old.iter().position(|old| old.name == name) {
                    old.swap_remove(i)
                } else {
                    FontInfo::with_name(name)
                }
            })
            .collect();
    }

    pub fn first_regular(&self) -> Option<&Font> {
        self.guifonts()
            .find_map(|font_info| font_info.regular.as_ref())
            .or_else(|| self.fallback.regular.as_ref())
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn guifonts(&self) -> impl Iterator<Item = &FontInfo> {
        self.fonts.iter().chain(std::iter::once(&self.fallback))
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

    pub fn fallback() -> Self {
        let builder = || FontPropertyBuilder::new().monospace();
        Self {
            name: String::default(),
            regular: get(builder()),
            bold: get(builder().bold()),
            italic: get(builder().italic()),
            bold_italic: get(builder().bold().italic()),
        }
    }
}

fn get(builder: FontPropertyBuilder) -> Option<Font> {
    system_fonts::get(&builder.build())
        .and_then(|(data, index)| Font::from_bytes(data, index as usize))
}
