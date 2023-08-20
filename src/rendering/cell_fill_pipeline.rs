use bytemuck::cast_slice;
use wgpu::include_wgsl;

use super::grid::GridInfo;

pub struct CellFillPipeline {
    pub pipeline: wgpu::RenderPipeline,
}

impl CellFillPipeline {
    pub fn new(
        device: &wgpu::Device,
        highlights_bind_group_layout: &wgpu::BindGroupLayout,
        grid_bind_group_layout: &wgpu::BindGroupLayout,
        texture_format: wgpu::TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(include_wgsl!("cell_fill.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Cell fill pipeline layout"),
            bind_group_layouts: &[&highlights_bind_group_layout, &grid_bind_group_layout],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX,
                range: 0..GridInfo::SIZE as u32,
            }],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Cell fill render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            // How to interpret vertices when converting to triangles
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self { pipeline }
    }

    pub fn render<'b, 'c, 'a: 'b + 'c>(
        &'a self,
        render_pass: &'b mut wgpu::RenderPass<'c>,
        highlights_bind_group: &'a wgpu::BindGroup,
        bg_bind_group: &'a wgpu::BindGroup,
        grid_info: GridInfo,
        bg_count: u32,
    ) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, highlights_bind_group, &[]);
        render_pass.set_bind_group(1, bg_bind_group, &[]);
        render_pass.set_push_constants(wgpu::ShaderStages::VERTEX, 0, cast_slice(&[grid_info]));
        render_pass.draw(0..bg_count * 6, 0..1);
    }
}
