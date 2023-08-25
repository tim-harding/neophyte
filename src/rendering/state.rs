use super::{
    cell_fill_pipeline::CellFillPipeline,
    cursor::Cursor,
    glyph_pipeline::GlyphPipeline,
    grid::{self, Grid},
    grid_bind_group_layout::GridBindGroupLayout,
    highlights::HighlightsBindGroup,
    shared::Shared,
};
use crate::{
    text::{
        cache::FontCache,
        fonts::{FontStyle, Fonts},
    },
    ui::Ui,
};
use std::sync::Arc;
use swash::shape::ShapeContext;
use wgpu::include_wgsl;
use winit::{dpi::PhysicalSize, window::Window};

pub struct RenderState {
    pub cursor: Cursor,
    pub shape_context: ShapeContext,
    pub font_cache: FontCache,
    pub shared: Shared,
    pub grids: Vec<grid::Grid>,
    pub glyph_pipeline: GlyphPipeline,
    pub emoji_pipeline: GlyphPipeline,
    pub cell_fill_pipeline: CellFillPipeline,
    pub highlights: HighlightsBindGroup,
    pub grid_bind_group_layout: GridBindGroupLayout,
}

// TODO: Use each pipeline to completion

impl RenderState {
    pub async fn new(window: Arc<Window>) -> Self {
        let shared = Shared::new(window).await;
        let highlights = HighlightsBindGroup::new(&shared.device);
        let grid_bind_group_layout = GridBindGroupLayout::new(&shared.device);
        Self {
            cursor: Cursor::new(&shared.device, shared.surface_config.format),
            shape_context: ShapeContext::new(),
            font_cache: FontCache::new(),
            glyph_pipeline: GlyphPipeline::new(
                &shared.device,
                shared
                    .device
                    .create_shader_module(include_wgsl!("glyph.wgsl")),
            ),
            emoji_pipeline: GlyphPipeline::new(
                &shared.device,
                shared
                    .device
                    .create_shader_module(include_wgsl!("emoji.wgsl")),
            ),
            cell_fill_pipeline: CellFillPipeline::new(
                &shared.device,
                &highlights.bind_group_layout,
                &grid_bind_group_layout.bind_group_layout,
                shared.surface_format,
            ),
            shared,
            grid_bind_group_layout,
            grids: vec![],
            highlights,
        }
    }

    pub fn update(&mut self, ui: &Ui, fonts: &mut Fonts) {
        let cell_size = fonts
            .with_style(FontStyle::Regular)
            .metrics(fonts.size())
            .cell_size_px;
        self.cursor
            .update(ui, self.shared.surface_size(), cell_size.into());

        let mut i = 0;
        while let Some(grid) = self.grids.get(i) {
            if ui.grid_index(grid.id).is_ok() {
                i += 1;
            } else {
                self.grids.remove(i);
            }
        }

        for ui_grid in ui.grids.iter() {
            let index = match self
                .grids
                .binary_search_by(|probe| probe.id.cmp(&ui_grid.id))
            {
                Ok(index) => index,
                Err(index) => {
                    self.grids.insert(index, Grid::new(ui_grid.id));
                    index
                }
            };
            let grid = &mut self.grids[index];

            if ui_grid.dirty {
                grid.update_content(
                    &self.shared,
                    ui_grid,
                    &ui.highlights,
                    fonts,
                    &mut self.font_cache,
                    &mut self.shape_context,
                    &self.grid_bind_group_layout.bind_group_layout,
                );
            }

            let z = 1.0
                - ui.draw_order
                    .iter()
                    .position(|&id| id == ui_grid.id)
                    .map(|i| i + 1)
                    .unwrap_or(0) as f32
                    / ui.draw_order.len() as f32;

            grid.update_grid_info(fonts, &self.shared, ui_grid, ui.position(ui_grid.id), z);
        }

        self.highlights.update(ui, &self.shared);
        self.glyph_pipeline.update(
            &self.shared,
            &self.font_cache.monochrome,
            &self.highlights.bind_group_layout,
            &self.grid_bind_group_layout,
            wgpu::TextureFormat::R8Unorm,
        );
        self.emoji_pipeline.update(
            &self.shared,
            &self.font_cache.emoji,
            &self.highlights.bind_group_layout,
            &self.grid_bind_group_layout,
            wgpu::TextureFormat::Rgba8UnormSrgb,
        );
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.shared.resize(size);
    }

    pub fn render(&self, draw_order: &[u64]) -> Result<(), wgpu::SurfaceError> {
        let output = self.shared.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder =
            self.shared
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render encoder"),
                });

        let highlights_bind_group = match &self.highlights.bind_group {
            Some(highlights_bind_group) => highlights_bind_group,
            None => return Ok(()),
        };

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None, // No multisampling
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.highlights.clear_color),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.shared.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            for &id in draw_order.iter().rev() {
                let i = self
                    .grids
                    .binary_search_by(|probe| probe.id.cmp(&{ id }))
                    .unwrap();
                let grid = &self.grids[i];

                if let Some(bg_bind_group) = &grid.bg_bind_group {
                    self.cell_fill_pipeline.render(
                        &mut render_pass,
                        highlights_bind_group,
                        bg_bind_group,
                        grid.grid_info,
                        grid.bg_count,
                    );
                }

                if let Some(glyph_bind_group) = &grid.glyph_bind_group {
                    self.glyph_pipeline.render(
                        &mut render_pass,
                        highlights_bind_group,
                        glyph_bind_group,
                        grid.glyph_count,
                        grid.grid_info,
                    );
                }

                if let Some(emoji_bind_group) = &grid.emoji_bind_group {
                    self.emoji_pipeline.render(
                        &mut render_pass,
                        highlights_bind_group,
                        emoji_bind_group,
                        grid.emoji_count,
                        grid.grid_info,
                    );
                }
            }

            self.cursor.render(&mut render_pass);
        }

        self.shared.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    pub fn rebuild_swap_chain(&mut self) {
        let size = self.shared.surface_size();
        let size = PhysicalSize::new(size.x, size.y);
        self.shared.resize(size)
    }
}
