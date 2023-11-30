//! Alpha-blends one texture over another. This is used to combine the results
//! of the monochrome and lines pipelines with the background cells. These need
//! to rendered separately so that the alphas of the monochrome and lines can be
//! used for rendering the cursor.

use crate::rendering::{nearest_sampler, texture::Texture};
use wgpu::include_wgsl;

pub struct Pipeline {
    pipeline_layout: wgpu::PipelineLayout,
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    sampler: wgpu::Sampler,
    shader: wgpu::ShaderModule,
}

impl Pipeline {
    pub fn new(device: &wgpu::Device, texture_view: &wgpu::TextureView) -> Self {
        let sampler = nearest_sampler(device);
        let shader = device.create_shader_module(include_wgsl!("blend.wgsl"));

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Blend bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blend pipeline"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        Self {
            pipeline: pipeline(device, &pipeline_layout, &shader),
            bind_group: bind_group(device, &bind_group_layout, texture_view, &sampler),
            pipeline_layout,
            bind_group_layout,
            sampler,
            shader,
        }
    }

    pub fn update(&mut self, device: &wgpu::Device, texture_view: &wgpu::TextureView) {
        self.pipeline = pipeline(device, &self.pipeline_layout, &self.shader);
        self.bind_group = bind_group(device, &self.bind_group_layout, texture_view, &self.sampler);
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, color_target: &wgpu::TextureView) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Blend render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }
}

fn pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Blend pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: Texture::LINEAR_FORMAT,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}

pub fn bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    texture_view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Blend bind group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}
