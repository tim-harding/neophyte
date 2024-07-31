use super::text::Text;
use crate::{
    event::{hl_attr_define::Attributes, rgb::Rgb, Content},
    text::{cache::FontCache, fonts::Fonts},
    ui::{grid::CellContents, messages::Messages},
    util::vec2::{CellVec, Vec2},
};
use swash::shape::ShapeContext;

pub struct MessageGrids {
    texts: Vec<Text>,
}

impl MessageGrids {
    pub const fn new() -> Self {
        Self { texts: vec![] }
    }

    #[allow(clippy::too_many_arguments)]
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
        if !messages.dirty_show {
            return;
        }

        self.texts.clear();
        let mut offset = 0;
        for message in messages.show.iter().rev() {
            let lines = lines(&message.content);
            let line_count = lines.len();
            let mut text = Text::new(CellVec::new(0, 0));
            text.update_contents(
                device,
                queue,
                None,
                lines
                    .into_iter()
                    .enumerate()
                    .map(|(i, c)| (i as i32, c.into_iter())),
                grid_bind_group_layout,
                highlights,
                default_fg,
                default_bg,
                fonts,
                font_cache,
                shape_context,
            );
            offset += line_count;
            text.update_window(Some(CellVec::new(
                (base_grid_size.x as u32).saturating_sub(text.size().0.x) as f32,
                base_grid_size.y as f32 - 1.0 - offset as f32,
            )));
            self.texts.push(text);
        }
    }

    pub fn texts(&self) -> impl Iterator<Item = &Text> {
        self.texts.iter()
    }
}

fn lines(content: &Content) -> Vec<Vec<CellContents>> {
    let mut lines = vec![];
    let mut cells = content
        .chunks
        .iter()
        .flat_map(|chunk| chunk.text_chunk.chars().map(|c| (c, chunk.attr_id)));
    loop {
        let mut line = vec![];
        for cell in cells.by_ref() {
            match cell.0 {
                '\n' => break,
                c => line.push(CellContents {
                    text: c.into(),
                    highlight: cell.1,
                }),
            }
        }
        if line.is_empty() {
            break;
        }
        lines.push(line);
    }
    lines
}
