//! For each grid, draw a quad with the correct position and depth with the
//! default background color. This sets the correct depth in the Z-buffer for
//! later passes. Once the Z-buffer is set correctly, we can just render the
//! glyphs and highlight backgrounds for each grid at the correct Z-depth with
//! LessEqual depth compare and they will be correctly masked, even with
//! front-to-back rendering and overlapping windows.

use crate::rendering::{
    text::{Text, set_scissor},
    texture::Texture,
};
use bytemuck::{Pod, Zeroable, checked::cast_slice};
use neophyte_linalg::{PixelVec, Vec2};
use wgpu::include_wgsl;

pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
}

impl Pipeline {
    pub fn new(device: &wgpu::Device, texture_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(include_wgsl!("default_fill.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Default fill pipeline layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX,
                range: 0..PushConstants::SIZE,
            }],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Default fill render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
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
            cache: None,
        });

        Self { pipeline }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render<'a, 'b>(
        &'a self,
        encoder: &'a mut wgpu::CommandEncoder,
        grids: impl Iterator<Item = (f32, &'b Text)>,
        color_target: &wgpu::TextureView,
        depth_target: &wgpu::TextureView,
        target_size: PixelVec<u32>,
        cell_size: Vec2<u32>,
        clear_color: [f32; 4],
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Default fill render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: (clear_color[0] as f64).powf(2.2),
                        g: (clear_color[1] as f64).powf(2.2),
                        b: (clear_color[2] as f64).powf(2.2),
                        a: (clear_color[3] as f64).powf(2.2),
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_target,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // TODO: Could do this in a single draw call
        render_pass.set_pipeline(&self.pipeline);
        for (z, grid) in grids {
            let size = grid.size().into_pixels(cell_size);
            if let Some(offset) = grid.offset() {
                let offset = offset.round_to_pixels(cell_size);
                set_scissor(size, offset, target_size, &mut render_pass);
                PushConstants {
                    z,
                    r: clear_color[0],
                    g: clear_color[1],
                    b: clear_color[2],
                    a: clear_color[3],
                }
                .set(&mut render_pass);
                render_pass.draw(0..6, 0..1);
            }
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct PushConstants {
    z: f32,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl PushConstants {
    pub const SIZE: u32 = std::mem::size_of::<Self>() as u32;

    pub fn set(self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_push_constants(wgpu::ShaderStages::VERTEX, 0, cast_slice(&[self]));
    }
}
