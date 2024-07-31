use super::text::Text;
use crate::{
    event::{hl_attr_define::Attributes, rgb::Rgb, Content},
    text::{cache::FontCache, fonts::Fonts},
    ui::{
        cmdline::{Cmdline, Mode},
        grid::CellContents,
    },
    util::vec2::{CellVec, Vec2},
};
use swash::shape::ShapeContext;

pub struct CmdlineGrid {
    pub text: Text,
}

impl CmdlineGrid {
    pub fn new() -> Self {
        Self {
            text: Text::new(CellVec::new(0, 0)),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        cmdline: &Cmdline,
        position: Option<CellVec<f32>>,
        grid_bind_group_layout: &wgpu::BindGroupLayout,
        highlights: &[Option<Attributes>],
        default_fg: Rgb,
        default_bg: Rgb,
        fonts: &Fonts,
        font_cache: &mut FontCache,
        shape_context: &mut ShapeContext,
        width: u32,
    ) {
        if let Some(mode) = &cmdline.mode {
            match mode {
                Mode::Normal { levels } => {
                    // TODO: Handle multiple levels
                    // TODO: Guarantee at least one level at the type level
                    let prompt = levels.last().unwrap();
                    let mut content_lines = prompt.content_lines.iter();
                    let first_line = content_lines.next().unwrap();
                    self.text.update_contents(
                        device,
                        queue,
                        Some(CellVec(Vec2::new(width, 1))),
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
                        .map(|(i, line)| (i as i32, line)),
                        grid_bind_group_layout,
                        highlights,
                        default_fg,
                        default_bg,
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
        } else {
            self.text.update_contents(
                device,
                queue,
                None,
                std::iter::empty::<(i32, std::iter::Empty<CellContents>)>(),
                grid_bind_group_layout,
                highlights,
                default_fg,
                default_bg,
                fonts,
                font_cache,
                shape_context,
            )
        }

        self.text.update_window(position);
    }
}

fn iter_line(content: &Content) -> impl Iterator<Item = CellContents> + Clone {
    content.chunks.iter().flat_map(|chunk| {
        chunk.text_chunk.chars().map(|c| CellContents {
            highlight: chunk.attr_id,
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
