use super::{
    state::{GlyphInfo, GridInfo, HighlightInfo, StateConstant},
    state_read::StateRead,
};
use crate::{
    event::hl_attr_define::Rgb,
    text::{
        cache::FontCache,
        font::{metrics, Font},
    },
    ui::Ui,
    util::vec2::Vec2,
};
use bytemuck::cast_slice;
use wgpu::util::DeviceExt;

pub struct StateWrite {
    pub highlights: Vec<HighlightInfo>,
    pub font_cache: FontCache,
    pub font: Font,
}

impl StateWrite {
    pub fn new(font: Font, font_cache: FontCache) -> Self {
        Self {
            font_cache,
            font,
            highlights: vec![],
        }
    }

    // TODO: Should only rebuild the pipeline as the result of a resize
    pub fn update_text(
        &mut self,
        ui: Ui,
        env: &StateConstant,
        surface_size: Vec2<u32>,
    ) -> StateRead {
        let grid = ui.composite();
        let font = self.font.as_ref();
        let charmap = font.charmap();
        let metrics = metrics(font, 24.0);

        let fg_default = ui.default_colors.rgb_fg.unwrap_or(Rgb::new(255, 255, 255));
        let bg_default = ui.default_colors.rgb_bg.unwrap_or(Rgb::new(0, 0, 0));

        let srgb = |n| (n as f32 / 255.0).powf(2.2);
        let srgb = |c: Rgb| [srgb(c.r()), srgb(c.g()), srgb(c.b()), 1.0];
        for highlight in ui.highlights.iter() {
            let i = *highlight.0 as usize;
            if i + 1 > self.highlights.len() {
                self.highlights.resize(i + 1, HighlightInfo::default());
            }
            self.highlights[i] = HighlightInfo {
                fg: srgb(highlight.1.rgb_attr.foreground.unwrap_or(fg_default)),
                bg: srgb(highlight.1.rgb_attr.background.unwrap_or(bg_default)),
            };
        }

        let highlights_buffer = env
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Highlight buffer"),
                contents: cast_slice(self.highlights.as_slice()),
                usage: wgpu::BufferUsages::STORAGE,
            });

        let highlights_bind_group = env.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Highlights bind group"),
            layout: &env.highlights_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &highlights_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        let srgb = |n| (n as f64 / 255.0).powf(2.2);
        let clear_color = wgpu::Color {
            r: srgb(bg_default.r()),
            g: srgb(bg_default.g()),
            b: srgb(bg_default.b()),
            a: 1.0,
        };

        let mut glyph_info = vec![];
        for (cell_line, hl_line) in grid.cells.rows().zip(grid.highlights.rows()) {
            for (c, hl) in cell_line.zip(hl_line) {
                let id = charmap.map(c);
                let glyph_index = match self.font_cache.lut.get(&id) {
                    Some(glyph) => glyph,
                    None => {
                        glyph_info.push(GlyphInfo {
                            glyph_index: 0,
                            highlight_index: hl,
                        });
                        continue;
                    }
                };

                glyph_info.push(GlyphInfo {
                    glyph_index: *glyph_index as u32,
                    highlight_index: hl,
                });
            }
        }

        let grid_info = GridInfo {
            surface_size,
            cell_size: Vec2::new(metrics.advance as u32, metrics.cell_height()),
            grid_width: grid.size().x as u32,
            baseline: metrics.ascent as u32,
        };

        let vertex_count = glyph_info.len() as u32 * 6;

        let glyph_info_buffer = env
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("info buffer"),
                usage: wgpu::BufferUsages::STORAGE,
                contents: cast_slice(glyph_info.as_slice()),
            });

        let grid_bind_group = env.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("glyph info bind group"),
            layout: &env.grid_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &glyph_info_buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        StateRead {
            clear_color,
            highlights_bind_group,
            grid_bind_group,
            grid_info,
            vertex_count,
        }
    }
}
