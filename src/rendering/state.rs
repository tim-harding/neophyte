use super::{
    cell_fill_pipeline::CellFillPipeline,
    glyph_pipeline::GlyphPipeline,
    grid,
    grid_bind_group_layout::GridBindGroupLayout,
    highlights::{HighlightsBindGroup, HighlightsBindGroupLayout},
    shared::Shared,
};
use crate::{
    text::{cache::FontCache, fonts::Fonts},
    ui::Ui,
};
use std::sync::Arc;
use swash::shape::ShapeContext;
use winit::{dpi::PhysicalSize, window::Window};

pub struct State {
    pub shape_context: ShapeContext,
    pub font_cache: FontCache,
    pub shared: Shared,
    pub grids: Vec<grid::Grid>,
    pub glyph_pipeline: GlyphPipeline,
    pub cell_fill_pipeline: CellFillPipeline,
    pub highlights_bind_group_layout: HighlightsBindGroupLayout,
    pub highlights: HighlightsBindGroup,
    pub grid_bind_group_layout: GridBindGroupLayout,
}

impl State {
    pub async fn new(window: Arc<Window>) -> Self {
        let shared = Shared::new(window).await;
        let highlights_bind_group_layout = HighlightsBindGroupLayout::new(&shared.device);
        let grid_bind_group_layout = GridBindGroupLayout::new(&shared.device);
        Self {
            shape_context: ShapeContext::new(),
            font_cache: FontCache::new(),
            glyph_pipeline: GlyphPipeline::new(&shared.device),
            cell_fill_pipeline: CellFillPipeline::new(
                &shared.device,
                &highlights_bind_group_layout.bind_group_layout,
                &grid_bind_group_layout.bind_group_layout,
                shared.surface_format,
            ),
            shared,
            grid_bind_group_layout,
            highlights_bind_group_layout,
            grids: vec![],
            highlights: HighlightsBindGroup::default(),
        }
    }

    pub fn update(&mut self, mut ui: Ui, fonts: &mut Fonts) {
        self.highlights
            .update(&ui, &self.highlights_bind_group_layout, &self.shared);
        self.glyph_pipeline.update(
            &self.shared,
            &mut self.font_cache,
            &self.highlights_bind_group_layout,
            &self.grid_bind_group_layout,
        );
        // TODO: Caching
        self.grids.clear();
        let highlights = ui.highlights;
        let ui_grids = std::mem::take(&mut ui.grids);
        self.grids = std::iter::once(1)
            .chain(ui.draw_order.into_iter())
            .map(|id| {
                let grid_index = ui_grids
                    .binary_search_by(|probe| probe.id.cmp(&id))
                    .unwrap();
                let ui_grid = ui_grids.get(grid_index).unwrap();
                grid::Grid::new(
                    &self.shared,
                    ui_grid,
                    &highlights,
                    fonts,
                    &mut self.font_cache,
                    &mut self.shape_context,
                    &self.grid_bind_group_layout,
                )
            })
            .collect();
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.shared.resize(size);
    }

    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
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
                depth_stencil_attachment: None,
            });

            for grid in self.grids.iter() {
                if let Some(bg_bind_group) = &grid.bg_bind_group {
                    self.cell_fill_pipeline.render(
                        &mut render_pass,
                        &highlights_bind_group,
                        &bg_bind_group,
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
            }
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
