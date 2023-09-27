use crate::util::vec2::Vec2;
use bytemuck::{Pod, Zeroable};
use wgpu::include_wgsl;

pub struct BlitRenderPipeline {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
    shader: wgpu::ShaderModule,
    sampler: wgpu::Sampler,
    pub push_constants: PushConstants,
    pub bind_group: wgpu::BindGroup,
    pub pipeline: wgpu::RenderPipeline,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Pod, Zeroable)]
pub struct PushConstants {
    src_size: Vec2<u32>,
    dst_size: Vec2<u32>,
}

impl PushConstants {
    pub const SIZE: u32 = std::mem::size_of::<Self>() as u32;
}

impl BlitRenderPipeline {
    pub fn new(
        device: &wgpu::Device,
        dst_format: wgpu::TextureFormat,
        src_tex: &wgpu::TextureView,
    ) -> Self {
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

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX,
                range: 0..PushConstants::SIZE,
            }],
        });

        let shader = device.create_shader_module(include_wgsl!("gamma_blit.wgsl"));

        Self {
            push_constants: PushConstants::default(),
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
        src_size: Vec2<u32>,
        dst_size: Vec2<u32>,
    ) {
        self.bind_group = bind_group(device, &self.bind_group_layout, &self.sampler, src_tex);
        self.pipeline = pipeline(device, &self.pipeline_layout, &self.shader, dst_format);
        self.push_constants = PushConstants { src_size, dst_size };
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
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: dst_format,
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
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
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
