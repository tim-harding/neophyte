use super::text::Text;
use crate::{
    event::{rgb::Rgb, Content, HlAttrDefine},
    text::{cache::FontCache, fonts::Fonts},
    ui::{
        cmdline::{Cmdline, Mode},
        grid::CellContents,
    },
    util::vec2::Vec2,
};
use swash::shape::ShapeContext;

pub struct CmdlineGrid {
    pub grid: Text,
}

impl CmdlineGrid {
    pub fn new() -> Self {
        Self {
            grid: Text::new(Vec2::new(0, 0)),
        }
    }

    pub fn update<'a>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        cmdline: &Cmdline,
        grid_bind_group_layout: &wgpu::BindGroupLayout,
        highlights: &[HlAttrDefine],
        default_fg: Rgb,
        fonts: &Fonts,
        font_cache: &mut FontCache,
        shape_context: &mut ShapeContext,
    ) {
        if let Some(mode) = &cmdline.mode {
            match mode {
                Mode::Normal { levels } => {
                    // TODO: Handle multiple levels
                    let prompt = levels.last().unwrap();
                    let mut content_lines = prompt.content_lines.iter();
                    let first_line = content_lines.next().unwrap();
                    self.grid.update_contents(
                        device,
                        queue,
                        None,
                        std::iter::once(IterVariants::Head(
                            std::iter::once(CellContents {
                                highlight: 0,
                                text: prompt.first_char.unwrap_or(' ').into(),
                            })
                            .chain(prompt.prompt.chars().map(|c| CellContents {
                                highlight: 0,
                                text: c.into(),
                            }))
                            .chain(iter_line(first_line)),
                        ))
                        .chain(content_lines.map(|line| IterVariants::Tail(iter_line(line))))
                        .enumerate()
                        .map(|(i, line)| (i as i64, line)),
                        grid_bind_group_layout,
                        highlights,
                        default_fg,
                        fonts,
                        font_cache,
                        shape_context,
                    )
                }
                Mode::Block {
                    previous_lines: _,
                    current_line: _,
                } => todo!(),
            }
        }
    }
}

fn iter_line(content: &Content) -> impl Iterator<Item = CellContents> + Clone {
    content.chunks.iter().flat_map(|chunk| {
        chunk.text_chunk.chars().map(|c| CellContents {
            highlight: chunk.attr_id.try_into().unwrap(),
            text: c.into(),
        })
    })
}

#[derive(Clone)]
enum IterVariants<'a, H, T>
where
    H: Iterator<Item = CellContents<'a>> + Clone,
    T: Iterator<Item = CellContents<'a>> + Clone,
{
    Head(H),
    Tail(T),
}

impl<'a, H, T> Iterator for IterVariants<'a, H, T>
where
    H: Iterator<Item = CellContents<'a>> + Clone,
    T: Iterator<Item = CellContents<'a>> + Clone,
{
    type Item = CellContents<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IterVariants::Head(h) => h.next(),
            IterVariants::Tail(t) => t.next(),
        }
    }
}
