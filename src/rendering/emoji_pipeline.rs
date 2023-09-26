use super::{depth_texture::DepthTexture, state::TARGET_FORMAT};
use crate::util::vec2::Vec2;
use bytemuck::{cast_slice, Pod, Zeroable};
use std::mem::size_of;
use wgpu::include_wgsl;

pub struct EmojiPipeline {
    shader: wgpu::ShaderModule,
    pipeline: Option<wgpu::RenderPipeline>,
}

impl EmojiPipeline {
    pub fn new(device: &wgpu::Device) -> Self {
        EmojiPipeline {
            shader: device.create_shader_module(include_wgsl!("emoji.wgsl")),
            pipeline: None,
        }
    }

    pub fn clear(&mut self) {
        self.pipeline = None;
    }

    pub fn update(
        &mut self,
        device: &wgpu::Device,
        glyph_bind_group_layout: &wgpu::BindGroupLayout,
        grid_bind_group_layout: &wgpu::BindGroupLayout,
    ) {
        let glyph_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Emoji pipeline layout"),
                bind_group_layouts: &[glyph_bind_group_layout, grid_bind_group_layout],
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::VERTEX,
                    range: 0..PushConstants::SIZE,
                }],
            });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Emoji pipeline"),
            layout: Some(&glyph_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &self.shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &self.shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: TARGET_FORMAT,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
                format: DepthTexture::FORMAT,
                depth_write_enabled: false,
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

        self.pipeline = Some(pipeline);
    }

    pub fn pipeline(&self) -> Option<&wgpu::RenderPipeline> {
        self.pipeline.as_ref()
    }
}

pub fn set_push_constants(
    render_pass: &mut wgpu::RenderPass,
    target_size: Vec2<u32>,
    offset: Vec2<i32>,
    z: f32,
) {
    render_pass.set_push_constants(
        wgpu::ShaderStages::VERTEX,
        0,
        cast_slice(&[PushConstants {
            target_size,
            offset,
            z,
        }]),
    );
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
struct PushConstants {
    pub target_size: Vec2<u32>,
    pub offset: Vec2<i32>,
    pub z: f32,
}

impl PushConstants {
    pub const SIZE: u32 = size_of::<Self>() as u32;
}
