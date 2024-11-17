use super::text::Text;
use crate::{
    text::{cache::FontCache, fonts::Fonts},
    ui::{grid::CellContents, messages::Messages},
    util::vec2::{CellVec, Vec2},
};
use neophyte_ui_event::{hl_attr_define::Attributes, rgb::Rgb, Content};
use swash::shape::ShapeContext;

pub struct MessageGrids {
    texts: Vec<Text>,
    previous_base_grid_size: Vec2<u16>,
}

impl MessageGrids {
    pub const fn new() -> Self {
        Self {
            texts: vec![],
            previous_base_grid_size: Vec2::new(0, 0),
        }
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
        let did_size_change = self.previous_base_grid_size != base_grid_size;
        self.previous_base_grid_size = base_grid_size;
        if !(messages.dirty || did_size_change) {
            return;
        }

        self.texts.clear();
        let mut offset = 0;
        let is_history = !messages.history.is_empty();
        let to_display = if is_history {
            &messages.history
        } else {
            &messages.show
        };
        for message in to_display.iter().rev() {
            let lines = lines(&message.content);
            offset += lines.len();
            let mut text = Text::new(CellVec::new(0, 0));

            let size = if is_history {
                Some(CellVec::new(base_grid_size.x as u32, lines.len() as u32))
            } else {
                None
            };
            text.update_contents(
                device,
                queue,
                size,
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

            let position = if is_history {
                CellVec::new(0.0, base_grid_size.y as f32 - offset as f32)
            } else {
                CellVec::new(
                    (base_grid_size.x as u32).saturating_sub(text.size().0.x) as f32,
                    base_grid_size.y as f32 - 1.0 - offset as f32,
                )
            };
            text.update_window(Some(position));

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
