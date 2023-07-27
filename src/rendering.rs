use png::Encoder;
use rusttype::{point, Font, Scale};
use std::{
    fs::{self, File},
    io::BufWriter,
    path::Path,
};

#[allow(unused)]
pub fn render() {
    let path = Path::new(r"/home/tim/temp.png");
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = Encoder::new(w, 512, 256);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut w = encoder.write_header().unwrap();

    let mut data = [255u8; 256 * 512 * 3];
    for x in 0..512 {
        for y in 0..256 {
            let p = (y * 512 + x) * 3;
            data[p] = x as u8;
            data[p + 1] = y as u8;
        }
    }

    let font_path = Path::new(r"/usr/share/fonts/TTF/CaskaydiaCoveNerdFont-Regular.ttf");
    let font_data = fs::read(&font_path).unwrap();
    let font = Font::try_from_vec(font_data).unwrap();
    let height = 48f32;

    let scale = Scale::uniform(height);

    // The ascent is the highest point of any glyph. We shift down so that first line doesn't clip.
    let v_metrics = font.v_metrics(scale);
    let offset = point(0.0, v_metrics.ascent);

    let glyphs: Vec<_> = font.layout("rusttype", scale, offset).collect();
    for glyph in glyphs {
        if let Some(bb) = glyph.pixel_bounding_box() {
            glyph.draw(|x, y, v| {
                let x = (x as i32 + bb.min.x) as usize;
                let y = (y as i32 + bb.min.y) as usize;
                if x > 512 || y > 256 {
                    return;
                }
                let v = 1.0 - v;
                let p = (y as usize * 512 + x as usize) * 3;
                data[p] = ((data[p] as f32) * v) as u8;
                data[p + 1] = ((data[p + 1] as f32) * v) as u8;
                data[p + 2] = ((data[p + 2] as f32) * v) as u8;
            });
        }
    }

    w.write_image_data(&data).unwrap();
}
