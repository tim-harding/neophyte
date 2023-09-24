use crate::{
    rendering::grid::{self, MonochromeCell},
    text::{
        cache::{CacheValue, FontCache, GlyphKind},
        fonts::{FontStyle, Fonts},
    },
    ui::Ui,
    util::vec2::Vec2,
};
use bytemuck::cast_slice;
use std::mem::size_of;
use swash::{
    shape::ShapeContext,
    text::{
        cluster::{CharCluster, Parser, Status, Token},
        Script,
    },
};

const MAX_DIACRITICS: u64 = 4;

#[derive(Debug)]
pub struct CursorFg {
    buffer: wgpu::Buffer,
    pub glyph_count: u8,
    pub bind_group: wgpu::BindGroup,
    pub grid_info: grid::PushConstants,
}

impl CursorFg {
    pub fn new(device: &wgpu::Device, grid_bind_group_layout: &wgpu::BindGroupLayout) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: size_of::<MonochromeCell>() as u64 * MAX_DIACRITICS,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: grid_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });
        Self {
            glyph_count: 0,
            buffer,
            bind_group,
            grid_info: Default::default(),
        }
    }

    pub fn update(
        &mut self,
        ui: &Ui,
        fonts: &mut Fonts,
        font_cache: &mut FontCache,
        shape_context: &mut ShapeContext,
        queue: &wgpu::Queue,
    ) {
        let highlight_index = ui
            .highlight_groups
            .get("Cursor")
            .cloned()
            .unwrap_or_default() as u32;
        let Some(grid) = ui.grid(ui.cursor.grid) else {
            self.glyph_count = 0;
            return;
        };
        let cell = grid.get(ui.cursor.pos);
        let style = if let Some(hl) = ui.highlights.get(&cell.highlight) {
            FontStyle::new(
                hl.rgb_attr.bold.unwrap_or_default(),
                hl.rgb_attr.italic.unwrap_or_default(),
            )
        } else {
            FontStyle::default()
        };

        let mut parser = Parser::new(
            Script::Latin,
            cell.text.chars().map(|c| Token {
                ch: c,
                offset: 0,
                len: 0,
                info: c.into(),
                data: cell.highlight as u32,
            }),
        );

        let mut cluster = CharCluster::new();
        if !parser.next(&mut cluster) {
            todo!()
        }

        let mut best_font = None;
        for font_info in fonts.iter() {
            if let Some(font) = font_info.style_or_regular(style) {
                match cluster.map(|c| font.charmap().map(c)) {
                    Status::Discard => {}
                    Status::Keep => best_font = Some(font),
                    Status::Complete => {
                        best_font = Some(font);
                        break;
                    }
                }
            }
        }

        let mut cell_count = 0;
        let mut cells = [MonochromeCell::default(); 4];

        let Some(font) = best_font else {
            self.glyph_count = 0;
            return;
        };

        let mut shaper = shape_context
            .builder(font.as_ref())
            .size(fonts.size() as f32)
            .script(Script::Arabic)
            .build();
        shaper.add_cluster(&cluster);
        let metrics = font.metrics(fonts.size());
        shaper.shape_with(|glyph_cluster| {
            for glyph in glyph_cluster.glyphs {
                let CacheValue { index, kind } =
                    match font_cache.get(font.as_ref(), metrics.em, glyph.id, style) {
                        Some(glyph) => glyph,
                        None => {
                            continue;
                        }
                    };
                let glyph_index = index as u32;

                match kind {
                    GlyphKind::Monochrome => {
                        let offset = font_cache.monochrome.offset[index];
                        let position = offset * Vec2::new(1, -1)
                            + Vec2::new(
                                (glyph.x * metrics.scale_factor).round() as i32,
                                (glyph.y * metrics.scale_factor + metrics.em_px as f32).round()
                                    as i32,
                            );
                        cells[cell_count] = MonochromeCell {
                            glyph_index,
                            highlight_index: 0,
                            position,
                        };
                        cell_count += 1;
                        if cell_count == 4 {
                            break;
                        }
                    }
                    GlyphKind::Emoji => continue,
                };
            }
        });

        self.glyph_count = cell_count as u8;
        queue.write_buffer(&self.buffer, 0, cast_slice(&cells));
        let grid_offset: Vec2<f32> = grid.offset().0.into();
        let cursor_pos: Vec2<f32> = ui.cursor.pos.into();
        let cell_size_px: Vec2<f32> = metrics.cell_size_px.into();
        self.grid_info = grid::PushConstants {
            offset: (grid_offset + cursor_pos) * cell_size_px,
            grid_width: 1,
            z: 0.0,
        };
    }
}
