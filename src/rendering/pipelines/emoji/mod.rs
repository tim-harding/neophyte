use crate::{
    rendering::{
        depth_texture::DepthTexture, glyph_bind_group::GlyphBindGroup,
        glyph_push_constants::GlyphPushConstants, state::TARGET_FORMAT,
    },
    text::cache::Cached,
};
use wgpu::include_wgsl;

pub struct Pipeline {
    shader: wgpu::ShaderModule,
    pipeline: Option<wgpu::RenderPipeline>,
    bind_group: GlyphBindGroup,
}

impl Pipeline {
    pub fn new(device: &wgpu::Device) -> Self {
        Pipeline {
            shader: device.create_shader_module(include_wgsl!("emoji.wgsl")),
            bind_group: GlyphBindGroup::new(device),
            pipeline: None,
        }
    }

    pub fn clear(&mut self) {
        self.pipeline = None;
        self.bind_group.clear();
    }

    pub fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        cached_glyphs: &Cached,
        grid_bind_group_layout: &wgpu::BindGroupLayout,
    ) {
        self.bind_group.update(
            device,
            queue,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            cached_glyphs,
        );
        let Some(glyph_bind_group_layout) = self.bind_group.layout() else {
            return;
        };

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

    pub fn bind_group(&self) -> Option<&wgpu::BindGroup> {
        self.bind_group.bind_group()
    }
}
