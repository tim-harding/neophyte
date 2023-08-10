#![allow(unused)]

use crate::util::vec2::Vec2;
use std::collections::HashMap;
use swash::{
    scale::{image::Image, Render, ScaleContext, Source, StrikeWith},
    zeno::Placement,
    FontRef, GlyphId,
};

pub type GlyphLut = HashMap<GlyphId, GlyphEntry>;

pub fn rasterize_font(font: FontRef, size: f32) -> (Vec<Image>, GlyphLut) {
    let mut textures = vec![];
    let mut lut = HashMap::new();
    let mut scale_context = ScaleContext::new();
    let mut scaler = scale_context.builder(font).size(size).hint(true).build();
    let mut i = 0;
    font.charmap().enumerate(|_c, id| {
        let image = Render::new(&[
            Source::ColorOutline(0),
            Source::ColorBitmap(StrikeWith::BestFit),
            Source::Outline,
        ])
        .render(&mut scaler, id)
        .unwrap();
        if image.data.len() > 0 {
            lut.insert(
                id,
                GlyphEntry {
                    placement: image.placement,
                    index: i,
                },
            );
            textures.push(image);
            i += 1;
        }
    });
    (textures, lut)
}

pub struct GlyphEntry {
    pub placement: Placement,
    pub index: u32,
}
