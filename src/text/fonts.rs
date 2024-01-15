use super::font::{Font, Metrics};
use crate::{
    ui::options::FontSize,
    util::{vec2::Vec2, MaybeInto, Parse},
};
use font_loader::system_fonts::{self, FontPropertyBuilder};
use swash::Setting;

/// Loaded fonts
#[derive(Debug, Clone)]
pub struct Fonts {
    fonts: Vec<FontFamily>,
}

impl Default for Fonts {
    fn default() -> Self {
        Self::new()
    }
}

impl Fonts {
    pub fn new() -> Self {
        Self {
            fonts: vec![
                FontFamily {
                    setting: FontSetting::with_name("Roboto Mono".to_string()),
                    regular: Font::from_bytes(
                        include_bytes!("fonts/RobotoMono-Regular.ttf").as_slice(),
                        0,
                        FontSize::default(),
                    ),
                    bold: Font::from_bytes(
                        include_bytes!("fonts/RobotoMono-Bold.ttf").as_slice(),
                        0,
                        FontSize::default(),
                    ),
                    bold_italic: Font::from_bytes(
                        include_bytes!("fonts/RobotoMono-BoldItalic.ttf").as_slice(),
                        0,
                        FontSize::default(),
                    ),
                    italic: Font::from_bytes(
                        include_bytes!("fonts/RobotoMono-Italic.ttf").as_slice(),
                        0,
                        FontSize::default(),
                    ),
                },
                FontFamily {
                    setting: FontSetting::with_name("Last Resort".to_string()),
                    regular: Font::from_bytes(
                        include_bytes!("fonts/LastResort-Regular.ttf").as_slice(),
                        0,
                        FontSize::default(),
                    ),
                    bold: None,
                    italic: None,
                    bold_italic: None,
                },
            ],
        }
    }

    pub fn set_font_size(&mut self, size: FontSize) {
        for font in self.fonts.iter_mut() {
            font.resize(size);
        }
    }

    pub fn set_fonts(&mut self, fonts: Vec<FontSetting>, size: FontSize) {
        let mut old = std::mem::take(&mut self.fonts);
        self.fonts = fonts
            .into_iter()
            .map(move |font| {
                if let Some(i) = old.iter().position(|old| old.setting == font) {
                    let mut existing = old.swap_remove(i);
                    existing.resize(size);
                    existing
                } else {
                    FontFamily::with_settings(font, size)
                }
            })
            .collect();
        if self.fonts.is_empty() {
            self.fonts = Self::new().fonts;
        }
    }

    pub fn families(&self) -> impl Iterator<Item = &FontFamily> {
        self.fonts.iter()
    }

    pub fn fonts(&self) -> impl Iterator<Item = (&Font, FontStyle)> {
        self.families().flat_map(|font| font.iter())
    }

    /// Get the metrics for the first loaded font
    pub fn metrics(&self) -> Metrics {
        self.families()
            .find_map(|variants| variants.metrics())
            .unwrap()
    }

    /// Get the cell size of the first loaded font
    pub fn cell_size(&self) -> Vec2<u32> {
        self.metrics().into_pixels().cell_size()
    }
}

#[derive(Clone, Debug)]
pub struct FontFamily {
    /// The font name
    pub setting: FontSetting,
    /// If regular variant of the font, if available
    pub regular: Option<Font>,
    /// If bold variant of the font, if available
    pub bold: Option<Font>,
    /// If italic variant of the font, if available
    pub italic: Option<Font>,
    /// If bold italic variant of the font, if available
    pub bold_italic: Option<Font>,
}

impl FontFamily {
    /// Attempt to load the system font with the given name
    pub fn with_settings(setting: FontSetting, size: FontSize) -> Self {
        let builder = || FontPropertyBuilder::new().family(&setting.name);
        Self {
            regular: get(builder(), size),
            bold: get(builder().bold(), size),
            italic: get(builder().italic(), size),
            bold_italic: get(builder().bold().italic(), size),
            setting,
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
        for (font, _) in self.iter_mut() {
            font.resize(size);
        }
    }

    /// An iterator over the loaded variants
    pub fn iter(&self) -> impl Iterator<Item = (&Font, FontStyle)> {
        [&self.regular, &self.bold, &self.italic, &self.bold_italic]
            .into_iter()
            .zip(STYLES)
            .filter_map(|(font, style)| font.as_ref().map(|font| (font, style)))
    }

    /// An iterator over the loaded variants
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&mut Font, FontStyle)> {
        [
            &mut self.regular,
            &mut self.bold,
            &mut self.italic,
            &mut self.bold_italic,
        ]
        .into_iter()
        .zip(STYLES)
        .filter_map(|(font, style)| font.as_mut().map(|font| (font, style)))
    }

    /// Metrics for the first loaded variant
    pub fn metrics(&self) -> Option<Metrics> {
        self.iter().map(|font| font.0.metrics()).next()
    }
}

const STYLES: [FontStyle; 4] = [
    FontStyle::Regular,
    FontStyle::Bold,
    FontStyle::Italic,
    FontStyle::BoldItalic,
];

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontSetting {
    pub name: String,
    pub features: Vec<Setting<u16>>,
    pub variations: Vec<Setting<f32>>,
}

impl FontSetting {
    pub fn with_name(name: String) -> Self {
        Self {
            name,
            features: vec![],
            variations: vec![],
        }
    }
}

impl Parse for FontSetting {
    fn parse(value: rmpv::Value) -> Option<Self> {
        match value {
            rmpv::Value::String(s) => Some(Self::with_name(s.into_str()?)),
            rmpv::Value::Map(map) => {
                let mut name = None;
                let mut features = vec![];
                let mut variations = vec![];
                for (k, v) in map {
                    match k.as_str()? {
                        "name" => name = Some(v.maybe_into()?),
                        "features" => features = v.maybe_into()?,
                        "variations" => variations = v.maybe_into()?,
                        _ => {}
                    }
                }
                Some(Self {
                    name: name?,
                    features,
                    variations,
                })
            }
            _ => None,
        }
    }
}

impl<T: Parse + Copy + SettingDefault> Parse for Setting<T> {
    fn parse(value: rmpv::Value) -> Option<Self> {
        match value {
            rmpv::Value::Map(map) => {
                let mut name: Option<String> = None;
                let mut value = None;
                for (k, v) in map {
                    match k.as_str()? {
                        "name" => name = Some(v.maybe_into()?),
                        "value" => value = Some(v.maybe_into()?),
                        _ => {}
                    }
                }
                if let (Some(name), Some(value)) = (name, value) {
                    Some((name.as_str(), value).into())
                } else {
                    None
                }
            }
            rmpv::Value::String(s) => Some((s.as_str()?, T::setting_default()).into()),
            _ => None,
        }
    }
}

trait SettingDefault {
    fn setting_default() -> Self;
}

impl SettingDefault for u16 {
    fn setting_default() -> Self {
        1
    }
}

impl SettingDefault for f32 {
    fn setting_default() -> Self {
        0.
    }
}
