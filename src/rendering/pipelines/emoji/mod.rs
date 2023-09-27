use crate::rendering::{
    depth_texture::DepthTexture, glyph_push_constants::GlyphPushConstants, state::TARGET_FORMAT,
};
use wgpu::include_wgsl;

pub struct Pipeline {
    shader: wgpu::ShaderModule,
    pipeline: Option<wgpu::RenderPipeline>,
}

impl Pipeline {
    pub fn new(device: &wgpu::Device) -> Self {
        Pipeline {
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
                    range: 0..GlyphPushConstants::SIZE,
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
