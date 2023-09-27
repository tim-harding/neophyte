use crate::{
    event::mode_info_set::CursorShape, rendering::state::TARGET_FORMAT, ui::Ui, util::vec2::Vec2,
};
use bytemuck::{cast_slice, Pod, Zeroable};
use wgpu::include_wgsl;

pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    push_constants: PushConstants,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

impl Pipeline {
    pub fn new(device: &wgpu::Device, monochrome_target: &wgpu::TextureView) -> Self {
        let shader = device.create_shader_module(include_wgsl!("cursor.wgsl"));

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

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Cursor bind group layout"),
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

        let bind_group = bind_group(device, &bind_group_layout, monochrome_target, &sampler);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Cursor pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[
                wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::VERTEX,
                    range: 0..PushConstantsVertex::SIZE as u32,
                },
                wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::FRAGMENT,
                    range: (PushConstantsVertex::SIZE as u32)
                        ..(PushConstantsVertex::SIZE as u32 + PushConstantsFragment::SIZE as u32),
                },
            ],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Cursor render pipeline"),
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
                    format: TARGET_FORMAT,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
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
        });

        Self {
            pipeline,
            push_constants: Default::default(),
            bind_group_layout,
            bind_group,
            sampler,
        }
    }

    pub fn update(
        &mut self,
        device: &wgpu::Device,
        ui: &Ui,
        surface_size: Vec2<u32>,
        cell_size: Vec2<f32>,
        monochrome_target: &wgpu::TextureView,
    ) {
        let mode = &ui.modes[ui.current_mode as usize];
        let fill = mode.cell_percentage.unwrap_or(10) as f32 / 100.0;
        let (fg, bg) = ui
            .highlight_groups
            .get("Cursor")
            .and_then(|hl_id| ui.highlights.get(hl_id))
            .and_then(|hl| {
                let fg = hl.rgb_attr.foreground?;
                let bg = hl.rgb_attr.background?;
                Some((fg, bg))
            })
            .unwrap_or((
                ui.default_colors.rgb_fg.unwrap_or_default(),
                ui.default_colors.rgb_bg.unwrap_or_default(),
            ));

        self.push_constants = PushConstants {
            vertex: PushConstantsVertex {
                position: (ui.position(ui.cursor.grid) + ui.cursor.pos.into()).into(),
                target_size: surface_size,
                fill: match mode.cursor_shape.unwrap_or(CursorShape::Block) {
                    CursorShape::Block => Vec2::new(1.0, 1.0),
                    CursorShape::Horizontal => Vec2::new(1.0, fill),
                    CursorShape::Vertical => Vec2::new(fill, 1.0),
                },
                cell_size,
            },
            fragment: PushConstantsFragment {
                fg: bg.into_linear(),
                bg: fg.into_linear(),
            },
        };

        self.bind_group = bind_group(
            device,
            &self.bind_group_layout,
            monochrome_target,
            &self.sampler,
        );
    }

    pub fn render<'b, 'c, 'a: 'b + 'c>(&'a self, render_pass: &'b mut wgpu::RenderPass<'c>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_push_constants(
            wgpu::ShaderStages::VERTEX,
            0,
            cast_slice(&[self.push_constants.vertex]),
        );
        render_pass.set_push_constants(
            wgpu::ShaderStages::FRAGMENT,
            PushConstantsVertex::SIZE as u32,
            cast_slice(&[self.push_constants.fragment]),
        );
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }
}

fn bind_group(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    src_tex: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Cursor bind group"),
        layout: &bind_group_layout,
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
#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
struct PushConstants {
    vertex: PushConstantsVertex,
    fragment: PushConstantsFragment,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
struct PushConstantsVertex {
    position: Vec2<f32>,
    target_size: Vec2<u32>,
    fill: Vec2<f32>,
    cell_size: Vec2<f32>,
}

impl PushConstantsFragment {
    pub const SIZE: usize = std::mem::size_of::<Self>();
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
struct PushConstantsFragment {
    fg: [f32; 4],
    bg: [f32; 4],
}

impl PushConstantsVertex {
    pub const SIZE: usize = std::mem::size_of::<Self>();
}
