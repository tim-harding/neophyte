use crate::{
    rendering::{depth_texture::DepthTexture, grid::Grid},
    util::vec2::Vec2,
};
use bytemuck::{checked::cast_slice, Pod, Zeroable};
use wgpu::include_wgsl;

pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        highlights_bind_group_layout: &wgpu::BindGroupLayout,
        grid_bind_group_layout: &wgpu::BindGroupLayout,
        texture_format: wgpu::TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(include_wgsl!("cell_fill.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Cell fill pipeline layout"),
            bind_group_layouts: &[highlights_bind_group_layout, grid_bind_group_layout],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX,
                range: 0..PushConstants::SIZE,
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
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DepthTexture::FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self { pipeline }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render<'a, 'b>(
        &'a self,
        encoder: &'a mut wgpu::CommandEncoder,
        grids: impl Iterator<Item = (f32, &'b Grid)>,
        color_target: &wgpu::TextureView,
        depth_target: &wgpu::TextureView,
        target_size: Vec2<u32>,
        highlights_bind_group: &wgpu::BindGroup,
        cell_size: Vec2<u32>,
        clear_color: wgpu::Color,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Cell fill render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color),
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_target,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, highlights_bind_group, &[]);
        for (z, grid) in grids {
            let Some(bg_bind_group) = &grid.cell_fill_bind_group() else {
                continue;
            };
            render_pass.set_bind_group(1, bg_bind_group, &[]);
            PushConstants {
                target_size,
                cell_size,
                offset: grid.offset(),
                grid_width: grid.size().x,
                z,
            }
            .set(&mut render_pass);
            render_pass.draw(0..grid.cell_fill_count() * 6, 0..1);
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct PushConstants {
    pub target_size: Vec2<u32>,
    pub cell_size: Vec2<u32>,
    pub offset: Vec2<i32>,
    pub grid_width: u32,
    pub z: f32,
}

impl PushConstants {
    pub const SIZE: u32 = std::mem::size_of::<Self>() as u32;

    pub fn set(self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_push_constants(wgpu::ShaderStages::VERTEX, 0, cast_slice(&[self]));
    }
}
