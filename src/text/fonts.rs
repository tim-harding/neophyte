use super::font::Font;
use font_loader::system_fonts::{self, FontPropertyBuilder};

pub struct Fonts {
    size: u32,
    fonts: Vec<FontInfo<NamePropertyBuilder>>,
    fallback: FontInfo<FallbackPropertyBuilder>,
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
                if let Some(i) = old.iter().position(|old| old.name() == name) {
                    old.swap_remove(i)
                } else {
                    FontInfo::with_name(name)
                }
            })
            .collect();
    }

    pub fn first_regular(&mut self) -> Option<Font> {
        self.fonts
            .iter_mut()
            .find_map(|font_info| font_info.regular())
            .or_else(|| self.fallback.regular())
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn fonts(&self) -> impl Iterator<Item = &FontInfo<NamePropertyBuilder>> {
        self.fonts.iter()
    }

    pub fn fonts_mut(&mut self) -> impl Iterator<Item = &mut FontInfo<NamePropertyBuilder>> {
        self.fonts.iter_mut()
    }

    pub fn fallback(&self) -> &FontInfo<FallbackPropertyBuilder> {
        &self.fallback
    }

    pub fn fallback_mut(&mut self) -> &mut FontInfo<FallbackPropertyBuilder> {
        &mut self.fallback
    }
}

pub struct FontInfo<T: PropertyBuilder> {
    builder: T,
    regular: FontState,
    bold: FontState,
    italic: FontState,
    bold_italic: FontState,
}

impl FontInfo<NamePropertyBuilder> {
    pub fn with_name(name: String) -> Self {
        Self::new(NamePropertyBuilder { name })
    }
}

impl FontInfo<FallbackPropertyBuilder> {
    pub fn fallback() -> Self {
        Self::new(FallbackPropertyBuilder)
    }
}

impl<T: PropertyBuilder> FontInfo<T> {
    pub fn new(builder: T) -> Self {
        Self {
            builder,
            regular: FontState::Unloaded,
            bold: FontState::Unloaded,
            italic: FontState::Unloaded,
            bold_italic: FontState::Unloaded,
        }
    }

    pub fn name(&self) -> &str {
        self.builder.name()
    }

    pub fn regular(&mut self) -> Option<Font> {
        get(self.builder.create(), &mut self.bold_italic)
    }

    pub fn bold(&mut self) -> Option<Font> {
        get(self.builder.create().bold(), &mut self.bold_italic)
    }

    pub fn italic(&mut self) -> Option<Font> {
        get(self.builder.create().italic(), &mut self.bold_italic)
    }

    pub fn bold_italic(&mut self) -> Option<Font> {
        get(self.builder.create().bold().italic(), &mut self.bold_italic)
    }
}

fn get(builder: FontPropertyBuilder, font_state: &mut FontState) -> Option<Font> {
    match font_state {
        FontState::Loaded(font) => return Some(font.clone()),
        FontState::Missing => None,
        FontState::Unloaded => match system_fonts::get(&builder.build()) {
            Some((data, index)) => {
                *font_state = match Font::from_bytes(data, index as usize) {
                    Some(font) => FontState::Loaded(font),
                    None => FontState::Missing,
                };
                match &font_state {
                    FontState::Loaded(font) => Some(font.clone()),
                    FontState::Unloaded | FontState::Missing => None,
                }
            }
            None => None,
        },
    }
}

#[derive(Clone)]
enum FontState {
    /// The font has been loaded
    Loaded(Font),
    /// The font has not been loaded
    Unloaded,
    /// Font loading was attempted and failed
    Missing,
}

pub trait PropertyBuilder {
    fn create(&self) -> FontPropertyBuilder;
    fn name(&self) -> &str;
}

pub struct NamePropertyBuilder {
    name: String,
}

impl PropertyBuilder for NamePropertyBuilder {
    fn create(&self) -> FontPropertyBuilder {
        FontPropertyBuilder::new().family(self.name.as_str())
    }

    fn name(&self) -> &str {
        &self.name
    }
}

pub struct FallbackPropertyBuilder;

impl PropertyBuilder for FallbackPropertyBuilder {
    fn create(&self) -> FontPropertyBuilder {
        FontPropertyBuilder::new().monospace()
    }

    fn name(&self) -> &str {
        ""
    }
}
