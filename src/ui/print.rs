use crate::event::{Content, HlAttrDefine};
use std::{collections::HashMap, io::Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub fn print_content(content: &Content, hl_attrs: &HashMap<u64, HlAttrDefine>) {
    let f = StandardStream::stdout(ColorChoice::Always);
    display_content(content, hl_attrs, f);
}

pub fn eprint_content(content: &Content, hl_attrs: &HashMap<u64, HlAttrDefine>) {
    let f = StandardStream::stderr(ColorChoice::Always);
    display_content(content, hl_attrs, f);
}

fn display_content(
    content: &Content,
    hl_attrs: &HashMap<u64, HlAttrDefine>,
    mut f: StandardStream,
) {
    for chunk in content.chunks.iter() {
        if let Some(hl_attr) = hl_attrs.get(&chunk.attr_id) {
            f.set_color(&hl_attr_to_colorspec(hl_attr)).unwrap();
        } else {
            f.reset().unwrap();
        }
        write!(f, "{}", chunk.text_chunk).unwrap();
    }
}

pub fn hl_attr_to_colorspec(hl: &HlAttrDefine) -> ColorSpec {
    let mut spec = ColorSpec::new();
    let hl = &hl.rgb_attr;
    let reverse = hl.reverse.unwrap_or(false);

    if let Some(foreground) = hl.foreground {
        let color = Some(u64_to_color(foreground));
        if reverse {
            spec.set_bg(color);
        } else {
            spec.set_fg(color);
        }
    }

    if let Some(background) = hl.background {
        let color = Some(u64_to_color(background));
        if reverse {
            spec.set_fg(color);
        } else {
            spec.set_bg(color);
        }
    }

    spec.set_italic(hl.italic.unwrap_or(false));
    spec.set_bold(hl.bold.unwrap_or(false));
    spec.set_strikethrough(hl.strikethrough.unwrap_or(false));
    spec.set_underline(hl.underline.unwrap_or(false));
    spec
}

pub fn u64_to_color(n: u64) -> Color {
    let r = (n >> 16) as u8;
    let g = (n >> 8) as u8;
    let b = n as u8;
    Color::Rgb(r, g, b)
}
