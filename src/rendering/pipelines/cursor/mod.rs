use crate::{
    event::mode_info_set::CursorShape,
    rendering::{nearest_sampler, Motion, TARGET_FORMAT},
    ui::Ui,
    util::{mat3::Mat3, vec2::Vec2},
};
use bytemuck::{cast_slice, Pod, Zeroable};
use std::mem::size_of;
use wgpu::include_wgsl;

pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    fragment_push_constants: FragmentPushConstants,
    start_position: Vec2<f32>,
    target_position: Vec2<f32>,
    elapsed: f32,
    fill: Vec2<f32>,
}

impl Pipeline {
    pub fn new(device: &wgpu::Device, monochrome_target: &wgpu::TextureView) -> Self {
        let shader = device.create_shader_module(include_wgsl!("cursor.wgsl"));
        let sampler = nearest_sampler(device);

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
                    range: 0..VertexPushConstants::SIZE,
                },
                wgpu::PushConstantRange {
                    stages: wgpu::ShaderStages::FRAGMENT,
                    range: VertexPushConstants::SIZE
                        ..(VertexPushConstants::SIZE + FragmentPushConstants::SIZE),
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
            bind_group_layout,
            bind_group,
            sampler,
            target_position: Vec2::default(),
            start_position: Vec2::default(),
            elapsed: f32::MAX,
            fragment_push_constants: FragmentPushConstants::default(),
            fill: Vec2::default(),
        }
    }

    pub fn update(
        &mut self,
        device: &wgpu::Device,
        ui: &Ui,
        cell_size: Vec2<f32>,
        monochrome_target: &wgpu::TextureView,
    ) {
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

        self.fragment_push_constants = FragmentPushConstants {
            fg: bg.into_linear(),
            bg: fg.into_linear(),
        };

        let mode = &ui.modes[ui.current_mode as usize];
        let fill = mode.cell_percentage.unwrap_or(10) as f32 / 100.0;
        self.fill = match mode.cursor_shape.unwrap_or(CursorShape::Block) {
            CursorShape::Block => Vec2::new(1.0, 1.0),
            CursorShape::Horizontal => Vec2::new(1.0, fill),
            CursorShape::Vertical => Vec2::new(fill, 1.0),
        };

        let new_target = ui.position(ui.cursor.grid) + ui.cursor.pos.cast_as();
        let new_target = cell_size * new_target.cast_as();

        if new_target != self.target_position {
            let current_position = self.start_position.lerp(self.target_position, self.t());
            self.start_position = current_position;
            self.elapsed = 0.0;
        }

        self.target_position = new_target;
        self.bind_group = bind_group(
            device,
            &self.bind_group_layout,
            monochrome_target,
            &self.sampler,
        );
    }

    fn t(&self) -> f32 {
        let length = (self.target_position - self.start_position).length();
        if length < 0.25 {
            f32::MAX
        } else {
            let length = length.sqrt() / 100.;
            let normal = (self.elapsed / length).min(1.);
            let a = 1.0 - normal;
            1.0 - a * a
        }
    }

    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        color_target: &wgpu::TextureView,
        delta_seconds: f32,
        cell_size: Vec2<f32>,
        target_size: Vec2<f32>,
    ) -> Motion {
        let toward = self.target_position - self.start_position;
        let length = toward.length();
        let direction = if length < 0.25 {
            Vec2::new(1.0, 0.0)
        } else {
            toward / length
        };
        let angle = f32::atan2(direction.x, direction.y);

        dbg!(delta_seconds);
        self.elapsed += delta_seconds;
        let t = self.t();
        let current_position = self.start_position.lerp(self.target_position, t);

        let transform = Mat3::scale(target_size.map(f32::recip))
                    * Mat3::translate(current_position)
                    * Mat3::translate(cell_size / 2.0)
                    * Mat3::rotate(-angle)
                    * Mat3::scale(Vec2::new(1.0, 1.0 + t * 40.0))
                    * Mat3::rotate(angle)
                    // * Mat3::skew(Vec2::new(-(angle * 2.0).sin(), 0.0))
                    * Mat3::translate(-cell_size / 2.0)
                    * Mat3::scale(cell_size);

        let transform = Mat3::scale(target_size.map(f32::recip))
            * Mat3::translate(current_position)
            * Mat3::scale(cell_size);

        let motion = if t >= 1.0 {
            Motion::Still
        } else {
            Motion::Animating
        };

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Cursor render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_push_constants(
            wgpu::ShaderStages::VERTEX,
            0,
            cast_slice(&[VertexPushConstants { transform }]),
        );
        render_pass.set_push_constants(
            wgpu::ShaderStages::FRAGMENT,
            VertexPushConstants::SIZE,
            cast_slice(&[self.fragment_push_constants]),
        );
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..6, 0..1);

        motion
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
#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
struct VertexPushConstants {
    transform: Mat3,
}

impl FragmentPushConstants {
    #[allow(unused)]
    pub const SIZE: u32 = size_of::<Self>() as u32;
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
struct FragmentPushConstants {
    fg: [f32; 4],
    bg: [f32; 4],
}

impl VertexPushConstants {
    pub const SIZE: u32 = size_of::<Self>() as u32;
}
