use super::text::Text;
use crate::{
    event::{hl_attr_define::Attributes, rgb::Rgb},
    text::{cache::FontCache, fonts::Fonts},
    ui::messages::Messages,
    util::vec2::Vec2,
};
use swash::shape::ShapeContext;

pub struct MessageGrids {
    texts: Vec<Text>,
}

impl MessageGrids {
    pub const fn new() -> Self {
        Self { texts: vec![] }
    }

    pub fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        messages: &Messages,
        base_grid_size: Vec2<u16>,
        grid_bind_group_layout: &wgpu::BindGroupLayout,
        highlights: &[Option<Attributes>],
        default_fg: Rgb,
        default_bg: Rgb,
        fonts: &Fonts,
        font_cache: &mut FontCache,
        shape_context: &mut ShapeContext,
    ) {
    }
}
