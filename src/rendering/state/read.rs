use super::{font, grid, highlights, ConstantState};
use bytemuck::cast_slice;

pub struct ReadState {
    pub grid: grid::Read,
    pub font: font::Read,
    pub highlights: highlights::Read,
}

pub struct ReadStateUpdates {
    pub grid: Option<grid::Read>,
    pub font: Option<font::Read>,
    pub highlights: Option<highlights::Read>,
}

impl ReadState {
    pub fn from_updates(updates: ReadStateUpdates) -> Option<Self> {
        let ReadStateUpdates {
            grid,
            font,
            highlights,
        } = updates;
        Some(Self {
            grid: grid?,
            font: font?,
            highlights: highlights?,
        })
    }

    pub fn apply_updates(&mut self, updates: ReadStateUpdates) {
        let ReadStateUpdates {
            grid,
            font,
            highlights,
        } = updates;
        if let Some(grid) = grid {
            self.grid = grid;
        }
        if let Some(font) = font {
            self.font = font;
        }
        if let Some(highlights) = highlights {
            self.highlights = highlights;
        }
    }

    pub fn render(&self, constant: &ConstantState) -> Result<(), wgpu::SurfaceError> {
        let output = constant.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = constant
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
                    load: wgpu::LoadOp::Clear(self.highlights.clear_color),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&constant.grid.cell_fill_render_pipeline);
        render_pass.set_bind_group(0, &self.highlights.bind_group, &[]);
        render_pass.set_bind_group(1, &self.grid.bind_group, &[]);
        render_pass.set_push_constants(
            wgpu::ShaderStages::VERTEX,
            0,
            cast_slice(&[self.grid.grid_info]),
        );
        render_pass.draw(0..self.grid.vertex_count, 0..1);

        render_pass.set_pipeline(&self.font.pipeline);
        render_pass.set_bind_group(0, &self.highlights.bind_group, &[]);
        render_pass.set_bind_group(1, &self.grid.bind_group, &[]);
        render_pass.set_bind_group(2, &self.font.bind_group, &[]);
        render_pass.set_push_constants(
            wgpu::ShaderStages::VERTEX,
            0,
            cast_slice(&[self.grid.grid_info]),
        );
        render_pass.draw(0..self.grid.vertex_count, 0..1);
        drop(render_pass);

        constant.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}
