//! Paints monochrome and emoji glyphs with the appropriate highlight colors.

use crate::{
    rendering::{
        glyph_bind_group::GlyphBindGroup,
        glyph_push_constants::GlyphPushConstants,
        text::{set_scissor, Text},
        texture::Texture,
    },
    text::cache::Cached,
};
use neophyte_linalg::{PixelVec, Vec2};
use wgpu::include_wgsl;

pub enum Kind {
    Monochrome,
    Emoji,
}

pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group: GlyphBindGroup,
    atlas_size: u32,
    kind: Kind,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        grid_bind_group_layout: &wgpu::BindGroupLayout,
        kind: Kind,
    ) -> Self {
        let shader = match kind {
            Kind::Monochrome => include_wgsl!("monochrome.wgsl"),
            Kind::Emoji => include_wgsl!("emoji.wgsl"),
        };
        let bind_group = GlyphBindGroup::new(device);
        let shader = device.create_shader_module(shader);

        let glyph_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Glyph pipeline layout"),
                bind_group_layouts: &[bind_group.layout(), grid_bind_group_layout],
                push_constant_ranges: &[wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::VERTEX,
                    range: 0..GlyphPushConstants::SIZE,
                }],
            });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Glyph pipeline"),
            layout: Some(&glyph_pipeline_layout),
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
                    format: Texture::LINEAR_FORMAT,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
            cache: None,
        });

        Pipeline {
            bind_group,
            pipeline,
            kind,
            atlas_size: 0,
        }
    }

    pub fn clear(&mut self) {
        self.bind_group.clear();
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, cached_glyphs: &Cached) {
        let texture_format = match self.kind {
            Kind::Monochrome => wgpu::TextureFormat::R8Unorm,
            Kind::Emoji => wgpu::TextureFormat::Rgba8UnormSrgb,
        };
        self.bind_group
            .update(device, queue, texture_format, cached_glyphs);
        self.atlas_size = cached_glyphs.atlas.size();
    }

    pub fn render<'a, 'b>(
        &'a self,
        encoder: &'a mut wgpu::CommandEncoder,
        grids: impl Iterator<Item = (f32, PixelVec<i32>, &'b Text)>,
        color_target: &wgpu::TextureView,
        depth_target: &wgpu::TextureView,
        target_size: PixelVec<u32>,
        cell_size: Vec2<u32>,
    ) {
        let color_load_op = match self.kind {
            Kind::Monochrome => wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            Kind::Emoji => wgpu::LoadOp::Load,
        };
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Glyph render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: color_load_op,
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

        if let Some(glyph_bind_group) = self.bind_group.bind_group() {
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, glyph_bind_group, &[]);
            for (z, scroll_offset, grid) in grids {
                let (bind_group, count) = match self.kind {
                    Kind::Monochrome => (grid.monochrome_bind_group(), grid.monochrome_count()),
                    Kind::Emoji => (grid.emoji_bind_group(), grid.emoji_count()),
                };
                let Some(bind_group) = bind_group else {
                    continue;
                };
                render_pass.set_bind_group(1, bind_group, &[]);
                let size = grid.size().into_pixels(cell_size);
                if let Some(offset) = grid.offset() {
                    let offset = offset.round_to_pixels(cell_size);
                    set_scissor(size, offset, target_size, &mut render_pass);
                    GlyphPushConstants {
                        target_size: target_size.try_cast().unwrap(),
                        offset: offset + scroll_offset,
                        z,
                        atlas_size: self.atlas_size.try_into().unwrap(),
                    }
                    .set(&mut render_pass);
                    render_pass.draw(0..count * 6, 0..1);
                }
            }
        }
    }
}
