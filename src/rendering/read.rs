use super::{font, grid, State};
use bytemuck::cast_slice;

pub struct ReadState {
    pub grid: grid::Read,
    pub font: font::Read,
}

pub struct ReadStateUpdates {
    pub grid: Option<grid::Read>,
    pub font: Option<font::Read>,
}

impl ReadState {
    pub fn from_updates(updates: ReadStateUpdates) -> Option<Self> {
        let ReadStateUpdates { grid, font } = updates;
        Some(Self {
            grid: grid?,
            font: font?,
        })
    }

    pub fn apply_updates(&mut self, updates: ReadStateUpdates) {
        let ReadStateUpdates { grid, font } = updates;
        if let Some(grid) = grid {
            self.grid = grid;
        }
        if let Some(font) = font {
            self.font = font;
        }
    }

    pub fn render(&self, state: &State) -> Result<(), wgpu::SurfaceError> {
        let highlights_bind_group = match &state.highlights.bind_group {
            Some(highlights_bind_group) => highlights_bind_group,
            None => return Ok(()),
        };
        let output = state.shared.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder =
            state
                .shared
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render encoder"),
                });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None, // No multisampling
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(state.highlights.clear_color),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&state.grid_constant.cell_fill_render_pipeline);
        render_pass.set_bind_group(0, &highlights_bind_group, &[]);
        render_pass.set_bind_group(1, &self.grid.bg_bind_group, &[]);
        render_pass.set_push_constants(
            wgpu::ShaderStages::VERTEX,
            0,
            cast_slice(&[self.grid.grid_info]),
        );
        render_pass.draw(0..self.grid.bg_count as u32 * 6, 0..1);

        render_pass.set_pipeline(&self.font.pipeline);
        render_pass.set_bind_group(0, &highlights_bind_group, &[]);
        render_pass.set_bind_group(1, &self.grid.glyph_bind_group, &[]);
        render_pass.set_bind_group(2, &self.font.bind_group, &[]);
        render_pass.set_push_constants(
            wgpu::ShaderStages::VERTEX,
            0,
            cast_slice(&[self.grid.grid_info]),
        );
        render_pass.draw(0..self.grid.glyph_count as u32 * 6, 0..1);
        drop(render_pass);

        state.shared.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}
