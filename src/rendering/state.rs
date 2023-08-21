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
use wgpu::include_wgsl;
use winit::{dpi::PhysicalSize, window::Window};

pub struct RenderState {
    pub shape_context: ShapeContext,
    pub font_cache: FontCache,
    pub shared: Shared,
    pub grids: Vec<(u64, grid::Grid)>,
    pub glyph_pipeline: GlyphPipeline,
    pub emoji_pipeline: GlyphPipeline,
    pub cell_fill_pipeline: CellFillPipeline,
    pub highlights_bind_group_layout: HighlightsBindGroupLayout,
    pub highlights: HighlightsBindGroup,
    pub grid_bind_group_layout: GridBindGroupLayout,
}

impl RenderState {
    pub async fn new(window: Arc<Window>) -> Self {
        let shared = Shared::new(window).await;
        let highlights_bind_group_layout = HighlightsBindGroupLayout::new(&shared.device);
        let grid_bind_group_layout = GridBindGroupLayout::new(&shared.device);
        Self {
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

    pub fn update(&mut self, ui: &Ui, fonts: &mut Fonts) {
        self.highlights
            .update(&ui, &self.highlights_bind_group_layout, &self.shared);
        self.glyph_pipeline.update(
            &self.shared,
            &self.font_cache.monochrome,
            &self.highlights_bind_group_layout,
            &self.grid_bind_group_layout,
            wgpu::TextureFormat::R8Unorm,
        );
        self.emoji_pipeline.update(
            &self.shared,
            &self.font_cache.emoji,
            &self.highlights_bind_group_layout,
            &self.grid_bind_group_layout,
            wgpu::TextureFormat::Rgba8UnormSrgb,
        );

        for ui_grid in ui.grids.iter() {
            if ui_grid.dirty {
                let grid = grid::Grid::new(
                    &self.shared,
                    &ui_grid,
                    &ui.highlights,
                    fonts,
                    &mut self.font_cache,
                    &mut self.shape_context,
                    &self.grid_bind_group_layout,
                );
                match self
                    .grids
                    .binary_search_by(|probe| probe.0.cmp(&ui_grid.id))
                {
                    Ok(index) => {
                        self.grids[index] = (ui_grid.id, grid);
                    }
                    Err(index) => {
                        self.grids.insert(index, (ui_grid.id, grid));
                    }
                }
            }
        }

        let mut i = 0;
        while let Some((id, _)) = self.grids.get(i) {
            if ui.grid_index(*id).is_ok() {
                i += 1;
            } else {
                self.grids.remove(i);
            }
        }

        // TODO: Assign z-index to windows
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

            for (_, grid) in self.grids.iter() {
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
