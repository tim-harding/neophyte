use super::{font, highlights, ConstantState, GridInfo};
use bytemuck::cast_slice;

pub struct ReadState {
    pub grid_bind_group: wgpu::BindGroup,
    pub grid_info: GridInfo,
    pub vertex_count: u32,
    pub font: font::Read,
    pub highlights: highlights::Read,
}

pub struct ReadStateUpdates {
    pub grid_bind_group: wgpu::BindGroup,
    pub grid_info: GridInfo,
    pub vertex_count: u32,
    pub font: Option<font::Read>,
    pub highlights: Option<highlights::Read>,
}

impl ReadState {
    pub fn from_updates(updates: ReadStateUpdates) -> Option<Self> {
        let ReadStateUpdates {
            grid_bind_group,
            grid_info,
            vertex_count,
            font,
            highlights,
        } = updates;
        Some(Self {
            grid_bind_group,
            grid_info,
            vertex_count,
            font: font?,
            highlights: highlights?,
        })
    }

    pub fn apply_updates(&mut self, updates: ReadStateUpdates) {
        let ReadStateUpdates {
            grid_bind_group,
            grid_info,
            vertex_count,
            font,
            highlights,
        } = updates;
        self.grid_bind_group = grid_bind_group;
        self.grid_info = grid_info;
        self.vertex_count = vertex_count;
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

        render_pass.set_pipeline(&constant.cell_fill_render_pipeline);
        render_pass.set_bind_group(0, &self.highlights.bind_group, &[]);
        render_pass.set_bind_group(1, &self.grid_bind_group, &[]);
        render_pass.set_push_constants(
            wgpu::ShaderStages::VERTEX,
            0,
            cast_slice(&[self.grid_info]),
        );
        render_pass.draw(0..self.vertex_count, 0..1);

        render_pass.set_pipeline(&self.font.pipeline);
        render_pass.set_bind_group(0, &self.highlights.bind_group, &[]);
        render_pass.set_bind_group(1, &self.grid_bind_group, &[]);
        render_pass.set_bind_group(2, &self.font.bind_group, &[]);
        render_pass.set_push_constants(
            wgpu::ShaderStages::VERTEX,
            0,
            cast_slice(&[self.grid_info]),
        );
        render_pass.draw(0..self.vertex_count, 0..1);
        drop(render_pass);

        constant.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}
