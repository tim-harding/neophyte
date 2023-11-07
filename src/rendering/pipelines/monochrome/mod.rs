use crate::{
    rendering::{
        depth_texture::DepthTexture, glyph_bind_group::GlyphBindGroup,
        glyph_push_constants::GlyphPushConstants, grid::Grid, TARGET_FORMAT,
    },
    text::cache::Cached,
    util::vec2::Vec2,
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
            shader: device.create_shader_module(include_wgsl!("monochrome.wgsl")),
            bind_group: GlyphBindGroup::new(device),
            pipeline: None,
        }
    }

    pub fn clear(&mut self) {
        self.bind_group.clear();
        self.pipeline = None;
    }

    pub fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        cached_glyphs: &Cached,
        highlights_bind_group_layout: &wgpu::BindGroupLayout,
        grid_bind_group_layout: &wgpu::BindGroupLayout,
    ) {
        self.bind_group
            .update(device, queue, wgpu::TextureFormat::R8Unorm, cached_glyphs);

        let Some(glyph_bind_group_layout) = self.bind_group.layout() else {
            return;
        };

        // TODO: We're recreating the pipeline every update

        let glyph_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Monochrome pipeline layout"),
                bind_group_layouts: &[
                    highlights_bind_group_layout,
                    glyph_bind_group_layout,
                    grid_bind_group_layout,
                ],
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::VERTEX,
                    range: 0..GlyphPushConstants::SIZE,
                }],
            });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Monochrome pipeline"),
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

    #[allow(clippy::too_many_arguments)]
    pub fn render<'a, 'b>(
        &'a self,
        encoder: &'a mut wgpu::CommandEncoder,
        grids: impl Iterator<Item = (f32, &'b Grid)>,
        color_target: &wgpu::TextureView,
        depth_target: &wgpu::TextureView,
        target_size: Vec2<u32>,
        cell_size: Vec2<u32>,
        highlights_bind_group: &wgpu::BindGroup,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Monochrome render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
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

        if let (Some(pipeline), Some(glyph_bind_group)) =
            (&self.pipeline, self.bind_group.bind_group())
        {
            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, highlights_bind_group, &[]);
            render_pass.set_bind_group(1, glyph_bind_group, &[]);
            for (z, grid) in grids {
                let Some(monochrome_bind_group) = &grid.monochrome_bind_group() else {
                    continue;
                };
                render_pass.set_bind_group(2, monochrome_bind_group, &[]);
                grid.set_scissor(cell_size, target_size, &mut render_pass);
                GlyphPushConstants {
                    target_size,
                    offset: grid.offset(cell_size.y as f32),
                    z,
                    padding: 0.0,
                }
                .set(&mut render_pass);
                render_pass.draw(0..grid.monochrome_count() * 6, 0..1);
            }
        }
    }
}
