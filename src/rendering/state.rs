use super::{
    cell_fill_pipeline::CellFillPipeline,
    cursor::{bg::CursorBg, fg::CursorFg},
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
    util::vec2::Vec2,
};
use bytemuck::{cast_slice, Pod, Zeroable};
use std::sync::Arc;
use swash::shape::ShapeContext;
use wgpu::include_wgsl;
use winit::{dpi::PhysicalSize, window::Window};

pub struct RenderState {
    pub cursor_bg: CursorBg,
    // pub cursor_fg: CursorFg,
    pub shape_context: ShapeContext,
    pub font_cache: FontCache,
    pub shared: Shared,
    pub grids: Vec<grid::Grid>,
    pub monochrome_pipeline: GlyphPipeline,
    pub emoji_pipeline: GlyphPipeline,
    pub cell_fill_pipeline: CellFillPipeline,
    pub highlights: HighlightsBindGroup,
    pub grid_bind_group_layout: GridBindGroupLayout,
    pub draw_order_index_cache: Vec<usize>,
    pub shared_push_constants: SharedPushConstants,
}

// TODO: Use each pipeline to completion

impl RenderState {
    pub async fn new(window: Arc<Window>) -> Self {
        let shared = Shared::new(window).await;
        let highlights = HighlightsBindGroup::new(&shared.device);
        let grid_bind_group_layout = GridBindGroupLayout::new(&shared.device);
        Self {
            cursor_bg: CursorBg::new(&shared.device, shared.surface_config.format),
            // cursor_fg: CursorFg::new(&shared.device, shared.surface_config.format),
            shape_context: ShapeContext::new(),
            font_cache: FontCache::new(),
            monochrome_pipeline: GlyphPipeline::new(
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
            draw_order_index_cache: vec![],
            shared_push_constants: SharedPushConstants::default(),
        }
    }

    pub fn update(&mut self, ui: &Ui, fonts: &mut Fonts) {
        let cell_size = fonts
            .with_style(FontStyle::Regular)
            .metrics(fonts.size())
            .cell_size_px;
        self.cursor_bg
            .update(ui, self.shared.surface_size(), cell_size.into());
        // self.cursor_fg.update(
        //     ui,
        //     self.shared.surface_size(),
        //     fonts,
        //     &mut self.font_cache,
        //     &mut self.shape_context,
        // );

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

            grid.update_grid_info(fonts, ui_grid, ui.position(ui_grid.id), z);
        }

        self.highlights.update(ui, &self.shared);
        self.monochrome_pipeline.update(
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
        self.shared_push_constants = SharedPushConstants {
            surface_size: self.shared.surface_size(),
            cell_size,
        };
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.shared.resize(size);
    }

    pub fn render(&mut self, draw_order: &[u64]) -> Result<(), wgpu::SurfaceError> {
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

            render_pass.set_pipeline(&self.cell_fill_pipeline.pipeline);
            render_pass.set_bind_group(0, highlights_bind_group, &[]);
            render_pass.set_push_constants(
                wgpu::ShaderStages::VERTEX,
                0,
                cast_slice(&[self.shared_push_constants]),
            );
            self.draw_order_index_cache.clear();
            for &id in draw_order.iter().rev() {
                let i = self
                    .grids
                    .binary_search_by(|probe| probe.id.cmp(&{ id }))
                    .unwrap();
                self.draw_order_index_cache.push(i);
                let grid = &self.grids[i];

                if let Some(bg_bind_group) = &grid.bg_bind_group {
                    render_pass.set_bind_group(1, &bg_bind_group, &[]);
                    render_pass.set_push_constants(
                        wgpu::ShaderStages::VERTEX,
                        SharedPushConstants::SIZE as u32,
                        cast_slice(&[grid.grid_info]),
                    );
                    render_pass.draw(0..grid.bg_count * 6, 0..1);
                }
            }

            if let Some(contingent) = &self.monochrome_pipeline.contingent {
                render_pass.set_pipeline(&contingent.pipeline);
                render_pass.set_bind_group(0, highlights_bind_group, &[]);
                render_pass.set_bind_group(1, &contingent.bind_group, &[]);
                render_pass.set_push_constants(
                    wgpu::ShaderStages::VERTEX,
                    0,
                    cast_slice(&[self.shared_push_constants]),
                );
                for i in self.draw_order_index_cache.iter() {
                    let grid = &self.grids[*i];
                    if let Some(monochrome_bind_group) = &grid.monochrome_bind_group {
                        render_pass.set_bind_group(2, &monochrome_bind_group, &[]);
                        render_pass.set_push_constants(
                            wgpu::ShaderStages::VERTEX,
                            SharedPushConstants::SIZE as u32,
                            cast_slice(&[grid.grid_info]),
                        );
                        render_pass.draw(0..grid.glyph_count * 6, 0..1);
                    }
                }
            }

            self.cursor_bg.render(&mut render_pass);
            // self.cursor_fg.render(&mut render_pass);

            if let Some(contingent) = &self.emoji_pipeline.contingent {
                render_pass.set_pipeline(&contingent.pipeline);
                render_pass.set_bind_group(0, highlights_bind_group, &[]);
                render_pass.set_bind_group(1, &contingent.bind_group, &[]);
                render_pass.set_push_constants(
                    wgpu::ShaderStages::VERTEX,
                    0,
                    cast_slice(&[self.shared_push_constants]),
                );
                for i in self.draw_order_index_cache.iter() {
                    let grid = &self.grids[*i];
                    if let Some(emoji_bind_group) = &grid.emoji_bind_group {
                        render_pass.set_bind_group(2, &emoji_bind_group, &[]);
                        render_pass.set_push_constants(
                            wgpu::ShaderStages::VERTEX,
                            SharedPushConstants::SIZE as u32,
                            cast_slice(&[grid.grid_info]),
                        );
                        render_pass.draw(0..grid.glyph_count * 6, 0..1);
                    }
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

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct SharedPushConstants {
    pub surface_size: Vec2<u32>,
    pub cell_size: Vec2<u32>,
}

impl SharedPushConstants {
    pub const SIZE: usize = std::mem::size_of::<Self>();
}
