//! Copies the floating-point texture we rendered to over to the output surface
//! while applying gamma-correction and premultiplying by the alpha for window
//! transparency as needed.

use crate::{rendering::nearest_sampler, util::vec2::PixelVec};
use bytemuck::{cast_slice, Pod, Zeroable};
use wgpu::include_wgsl;

pub struct Pipeline {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
    shader: wgpu::ShaderModule,
    sampler: wgpu::Sampler,
    transparent: bool,
    pub push_constants_vertex: PushConstantsVertex,
    pub bind_group: wgpu::BindGroup,
    pub pipeline: wgpu::RenderPipeline,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        dst_format: wgpu::TextureFormat,
        src_tex: &wgpu::TextureView,
    ) -> Self {
        let sampler = nearest_sampler(device);
        let shader = device.create_shader_module(include_wgsl!("gamma_blit.wgsl"));

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
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
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[
                wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::VERTEX,
                    range: 0..PushConstantsVertex::SIZE,
                },
                wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::FRAGMENT,
                    range: PushConstantsVertex::SIZE
                        ..(PushConstantsVertex::SIZE + PushConstantsFragment::SIZE),
                },
            ],
        });

        Self {
            push_constants_vertex: PushConstantsVertex::default(),
            transparent: false,
            pipeline: pipeline(device, &pipeline_layout, &shader, dst_format),
            bind_group: bind_group(device, &bind_group_layout, &sampler, src_tex),
            bind_group_layout,
            pipeline_layout,
            shader,
            sampler,
        }
    }

    pub fn update(
        &mut self,
        device: &wgpu::Device,
        dst_format: wgpu::TextureFormat,
        src_tex: &wgpu::TextureView,
        src_size: PixelVec<u32>,
        dst_size: PixelVec<u32>,
        transparent: bool,
    ) {
        self.bind_group = bind_group(device, &self.bind_group_layout, &self.sampler, src_tex);
        self.pipeline = pipeline(device, &self.pipeline_layout, &self.shader, dst_format);
        self.push_constants_vertex = PushConstantsVertex {
            src_size: src_size.try_cast().unwrap(),
            dst_size: dst_size.try_cast().unwrap(),
        };
        self.transparent = transparent;
    }

    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        color_target: &wgpu::TextureView,
        mut clear_color: wgpu::Color,
    ) {
        if self.transparent {
            clear_color.r *= clear_color.a;
            clear_color.g *= clear_color.a;
            clear_color.b *= clear_color.a;
        }
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Blit render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_push_constants(
            wgpu::ShaderStages::VERTEX,
            0,
            cast_slice(&[self.push_constants_vertex]),
        );
        render_pass.set_push_constants(
            wgpu::ShaderStages::FRAGMENT,
            PushConstantsVertex::SIZE,
            cast_slice(&[PushConstantsFragment {
                transparent: self.transparent as u8 as f32,
            }]),
        );
        render_pass.draw(0..6, 0..1);
    }
}

fn pipeline(
    device: &wgpu::Device,
    pipeline_layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    dst_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(pipeline_layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: dst_format,
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
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    })
}

fn bind_group(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    src_tex: &wgpu::TextureView,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(src_tex),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Pod, Zeroable)]
pub struct PushConstantsVertex {
    src_size: PixelVec<i32>,
    dst_size: PixelVec<i32>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Pod, Zeroable)]
pub struct PushConstantsFragment {
    transparent: f32,
}

impl PushConstantsVertex {
    pub const SIZE: u32 = std::mem::size_of::<Self>() as u32;
}

impl PushConstantsFragment {
    pub const SIZE: u32 = std::mem::size_of::<Self>() as u32;
}
