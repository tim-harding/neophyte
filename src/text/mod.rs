pub mod cache;
pub mod font;
pub mod fonts;

use crate::text::font::Font;
use png::{ColorType, Encoder};
use std::{fs::File, io::BufWriter, path::Path};
use swash::{
    scale::{Render, ScaleContext, Source, StrikeWith},
    shape::{Direction, ShapeContext},
    text::Script,
    zeno::Vector,
};

// TODO: Subpixel rendering
// TODO: Font atlas
// TODO: Subpixel offsets
// TODO: Emoji

#[allow(unused)]
pub fn render_shaped_text() {
    let cascadia = Font::from_file("/usr/share/fonts/OTF/CascadiaCode-Regular.otf", 0).unwrap();
    let noto = Font::from_file("/usr/share/fonts/noto/NotoColorEmoji.ttf", 0).unwrap();
    let mut shape_context = ShapeContext::new();
    let mut shaper = shape_context
        .builder(cascadia.as_ref())
        .script(Script::Arabic)
        .direction(Direction::RightToLeft)
        .size(24.0)
        .build();
    shaper.add_str("Things and stuff ->");
    let mut scale_context = ScaleContext::new();
    let mut scaler = scale_context
        .builder(cascadia.as_ref())
        .size(24.0)
        .hint(true)
        .build();
    const WIDTH: usize = 600;
    const HEIGHT: usize = 40;
    let mut data = [0u8; WIDTH * HEIGHT];
    let mut x_offset = 0.0f32;
    shaper.shape_with(|cluster| {
        for glyph in cluster.glyphs {
            let image = Render::new(&[
                Source::ColorOutline(0),
                Source::ColorBitmap(StrikeWith::BestFit),
                Source::Outline,
            ])
            .offset(Vector::new(
                glyph.x.fract() + x_offset.fract(),
                glyph.y.fract(),
            ))
            .render(&mut scaler, glyph.id)
            .unwrap();
            for y in 0..image.placement.height as i32 {
                for x in 0..image.placement.width as i32 {
                    let dst_y = y - image.placement.top - glyph.y as i32 + 24;
                    let dst_x = x + x_offset as i32 + image.placement.left + glyph.x as i32;
                    if dst_y < 0 || dst_x < 0 {
                        continue;
                    }
                    let dst_i = dst_y as usize * WIDTH + dst_x as usize;
                    let src_i = y as usize * image.placement.width as usize + x as usize;
                    data[dst_i] = data[dst_i].saturating_add(image.data[src_i]);
                }
            }
            x_offset += glyph.advance;
        }
    });
    write_png(
        "/home/tim/temp.png",
        WIDTH as u32,
        HEIGHT as u32,
        ColorType::Grayscale,
        &data,
    );
}

fn write_png(
    path: impl AsRef<Path>,
    width: u32,
    height: u32,
    color: ColorType,
    pixels: &[u8],
) -> Result<(), WritePngError> {
    let file = File::create(path)?;
    let ref mut w = BufWriter::new(file);
    let mut encoder = Encoder::new(w, width, height);
    encoder.set_color(color);
    encoder.set_depth(png::BitDepth::Eight);
    let mut w = encoder.write_header().unwrap();
    w.write_image_data(pixels)?;
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum WritePngError {
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("PNG: {0}")]
    Png(#[from] png::EncodingError),
}
