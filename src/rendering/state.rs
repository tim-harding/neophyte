use super::{
    font::GlyphPipeline,
    grid::{self, GridBindGroupLayout},
    highlights::{HighlightsBindGroup, HighlightsBindGroupLayout},
    shared::Shared,
};
use crate::{
    text::{cache::FontCache, fonts::Fonts},
    ui::Ui,
};
use bytemuck::cast_slice;
use std::sync::Arc;
use winit::{dpi::PhysicalSize, window::Window};

pub struct State {
    pub shared: Shared,
    pub grid: grid::Write,
    pub glyph_pipeline: GlyphPipeline,
    pub highlights_bind_group_layout: HighlightsBindGroupLayout,
    pub highlights: HighlightsBindGroup,
    pub grid_bind_group_layout: grid::GridBindGroupLayout,
}

impl State {
    pub async fn new(window: Arc<Window>) -> Self {
        let shared = Shared::new(window).await;
        let highlights_bind_group_layout = HighlightsBindGroupLayout::new(&shared.device);
        let grid_bind_group_layout = GridBindGroupLayout::new(&shared.device);
        let grid_write = grid::Write::new(
            &shared.device,
            shared.surface_format,
            &highlights_bind_group_layout,
            &grid_bind_group_layout,
        );
        Self {
            glyph_pipeline: GlyphPipeline::new(&shared.device),
            shared,
            grid_bind_group_layout,
            highlights_bind_group_layout,
            grid: grid_write,
            highlights: HighlightsBindGroup::default(),
        }
    }

    pub fn update(&mut self, ui: Ui, fonts: &mut Fonts, font_cache: &mut FontCache) {
        self.highlights
            .update(&ui, &self.highlights_bind_group_layout, &self.shared);
        self.glyph_pipeline.update(
            &self.shared,
            font_cache,
            &self.highlights_bind_group_layout,
            &self.grid_bind_group_layout,
        );
        self.grid.updates(
            &self.shared,
            &ui,
            fonts,
            font_cache,
            &self.grid_bind_group_layout,
        );
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
        let glyph_bind_group = match &self.grid.glyph_bind_group {
            Some(glyph_bind_group) => glyph_bind_group,
            None => return Ok(()),
        };
        let bg_bind_group = match &self.grid.bg_bind_group {
            Some(bg_bind_group) => bg_bind_group,
            None => return Ok(()),
        };
        let grid_info = match &self.grid.grid_info {
            Some(grid_info) => *grid_info,
            None => return Ok(()),
        };
        let glyph_count = match &self.grid.glyph_count {
            Some(glyph_count) => *glyph_count,
            None => return Ok(()),
        };
        let bg_count = match &self.grid.bg_count {
            Some(bg_count) => *bg_count,
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

            render_pass.set_pipeline(&self.grid.cell_fill_render_pipeline);
            render_pass.set_bind_group(0, &highlights_bind_group, &[]);
            render_pass.set_bind_group(1, &bg_bind_group, &[]);
            render_pass.set_push_constants(wgpu::ShaderStages::VERTEX, 0, cast_slice(&[grid_info]));
            render_pass.draw(0..bg_count as u32 * 6, 0..1);

            self.glyph_pipeline.render(
                &mut render_pass,
                highlights_bind_group,
                glyph_bind_group,
                glyph_count,
                grid_info,
            );
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
