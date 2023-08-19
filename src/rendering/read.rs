use super::State;
use bytemuck::cast_slice;

pub fn render(state: &State) -> Result<(), wgpu::SurfaceError> {
    let output = state.shared.surface.get_current_texture()?;
    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = state
        .shared
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render encoder"),
        });

    let highlights_bind_group = match &state.highlights.bind_group {
        Some(highlights_bind_group) => highlights_bind_group,
        None => return Ok(()),
    };
    let glyph_bind_group = match &state.grid.glyph_bind_group {
        Some(glyph_bind_group) => glyph_bind_group,
        None => return Ok(()),
    };
    let bg_bind_group = match &state.grid.bg_bind_group {
        Some(bg_bind_group) => bg_bind_group,
        None => return Ok(()),
    };
    let grid_info = match &state.grid.grid_info {
        Some(grid_info) => *grid_info,
        None => return Ok(()),
    };
    let glyph_count = match &state.grid.glyph_count {
        Some(glyph_count) => *glyph_count,
        None => return Ok(()),
    };
    let bg_count = match &state.grid.bg_count {
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
                    load: wgpu::LoadOp::Clear(state.highlights.clear_color),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&state.grid.cell_fill_render_pipeline);
        render_pass.set_bind_group(0, &highlights_bind_group, &[]);
        render_pass.set_bind_group(1, &bg_bind_group, &[]);
        render_pass.set_push_constants(wgpu::ShaderStages::VERTEX, 0, cast_slice(&[grid_info]));
        render_pass.draw(0..bg_count as u32 * 6, 0..1);

        state.glyph_pipeline.render(
            &mut render_pass,
            highlights_bind_group,
            glyph_bind_group,
            glyph_count,
            grid_info,
        );
    }

    state.shared.queue.submit(std::iter::once(encoder.finish()));
    output.present();
    Ok(())
}
