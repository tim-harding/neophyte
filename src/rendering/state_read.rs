use super::state::{GridInfo, StateConstant};
use bytemuck::cast_slice;

pub struct StateRead {
    pub clear_color: wgpu::Color,
    pub highlights_bind_group: wgpu::BindGroup,
    pub grid_bind_group: wgpu::BindGroup,
    pub grid_info: GridInfo,
    pub vertex_count: u32,
}

impl StateRead {
    pub fn render(&self, constant: &StateConstant) -> Result<(), wgpu::SurfaceError> {
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
                    load: wgpu::LoadOp::Clear(self.clear_color),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&constant.cell_fill_render_pipeline);
        render_pass.set_bind_group(0, &self.highlights_bind_group, &[]);
        render_pass.set_bind_group(1, &self.grid_bind_group, &[]);
        render_pass.set_push_constants(
            wgpu::ShaderStages::VERTEX,
            0,
            cast_slice(&[self.grid_info]),
        );
        render_pass.draw(0..self.vertex_count, 0..1);

        render_pass.set_pipeline(&constant.glyph_render_pipeline);
        render_pass.set_bind_group(0, &self.highlights_bind_group, &[]);
        render_pass.set_bind_group(1, &self.grid_bind_group, &[]);
        render_pass.set_bind_group(2, &constant.font_bind_group, &[]);
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
