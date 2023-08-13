use super::{
    font, highlights, read::ReadStateUpdates, surface_config::SurfaceConfig, ConstantState,
    GlyphInfo, GridInfo,
};
use crate::{
    text::font::{metrics, Font},
    ui::Ui,
    util::vec2::Vec2,
};
use bytemuck::cast_slice;
use wgpu::util::DeviceExt;

pub struct WriteState {
    pub font: Font,
    pub font_write: font::Write,
    pub highlights: highlights::Write,
}

impl WriteState {
    pub fn new(font: Font, font_write: font::Write, highlights: highlights::Write) -> Self {
        Self {
            font,
            font_write,
            highlights,
        }
    }

    // TODO: Should only rebuild the pipeline as the result of a resize
    pub fn updates(
        &mut self,
        ui: Ui,
        constant: &ConstantState,
        surface_config: &SurfaceConfig,
    ) -> ReadStateUpdates {
        let grid = ui.composite();
        let font = self.font.as_ref();
        let charmap = font.charmap();
        let metrics = metrics(font, 24.0);

        let highlights = self.highlights.updates(&ui, &constant);

        let mut glyph_info = vec![];
        for (cell_line, hl_line) in grid.cells.rows().zip(grid.highlights.rows()) {
            for (c, hl) in cell_line.zip(hl_line) {
                let id = charmap.map(c);
                let glyph_index = match self.font_write.font_cache.get(font, 24.0, id) {
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
                    glyph_index: glyph_index as u32,
                    highlight_index: hl,
                });
            }
        }

        let grid_info = GridInfo {
            surface_size: surface_config.size(),
            cell_size: Vec2::new(metrics.advance as u32, metrics.cell_height()),
            grid_width: grid.size().x as u32,
            baseline: metrics.ascent as u32,
        };

        let vertex_count = glyph_info.len() as u32 * 6;

        let glyph_info_buffer =
            constant
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("info buffer"),
                    usage: wgpu::BufferUsages::STORAGE,
                    contents: cast_slice(glyph_info.as_slice()),
                });

        let grid_bind_group = constant
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("glyph info bind group"),
                layout: &constant.grid_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &glyph_info_buffer,
                        offset: 0,
                        size: None,
                    }),
                }],
            });

        let font = self.font_write.updates(constant, &surface_config);

        ReadStateUpdates {
            grid_bind_group,
            grid_info,
            vertex_count,
            font,
            highlights,
        }
    }
}
