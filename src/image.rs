use png::Encoder;
use std::{
    fs::{self, File},
    io::BufWriter,
    path::Path,
};
use swash::FontRef;

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
    let font = FontRef::from_index(&font_data, 0).unwrap();
    println!("{}", font.attributes());
    for string in font.localized_strings() {
        println!("[{:?}] {}", string.id(), string.to_string());
    }

    w.write_image_data(&data).unwrap();
}
