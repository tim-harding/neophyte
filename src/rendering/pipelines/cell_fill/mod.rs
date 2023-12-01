//! Renders the highlight backgrounds for grid cells that don't use the default
//! fill color. The default fill is already written by the default_fill
//! pipeline.

use crate::{
    rendering::{
        text::{set_scissor, Text},
        texture::Texture,
    },
    util::vec2::{PixelVec, Vec2},
};
use bytemuck::{checked::cast_slice, Pod, Zeroable};
use wgpu::include_wgsl;

pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        grid_bind_group_layout: &wgpu::BindGroupLayout,
        texture_format: wgpu::TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(include_wgsl!("cell_fill.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Cell fill pipeline layout"),
            bind_group_layouts: &[grid_bind_group_layout],
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
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
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
        grids: impl Iterator<Item = (f32, PixelVec<i32>, &'b Text)>,
        color_target: &wgpu::TextureView,
        depth_target: &wgpu::TextureView,
        target_size: PixelVec<u32>,
        cell_size: Vec2<u32>,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Cell fill render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_target,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        for (z, scroll_offset, grid) in grids {
            let Some(bg_bind_group) = &grid.cell_fill_bind_group() else {
                continue;
            };
            render_pass.set_bind_group(0, bg_bind_group, &[]);

            if let Some(offset) = grid.offset() {
                let size = grid.size().into_pixels(cell_size);
                let offset = offset.round_to_pixels(cell_size);
                set_scissor(size, offset, target_size, &mut render_pass);
                PushConstants {
                    target_size,
                    cell_size,
                    offset: offset + scroll_offset,
                    z,
                    padding: 0,
                }
                .set(&mut render_pass);
                render_pass.draw(0..grid.cell_fill_count() * 6, 0..1);
            }
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct PushConstants {
    pub target_size: PixelVec<u32>,
    pub cell_size: Vec2<u32>,
    pub offset: PixelVec<i32>,
    pub z: f32,
    pub padding: u32,
}

impl PushConstants {
    pub const SIZE: u32 = std::mem::size_of::<Self>() as u32;

    pub fn set(self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_push_constants(wgpu::ShaderStages::VERTEX, 0, cast_slice(&[self]));
    }
}
