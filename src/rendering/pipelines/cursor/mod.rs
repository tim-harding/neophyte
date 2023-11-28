use crate::{
    event::mode_info_set::CursorShape,
    rendering::{nearest_sampler, texture::Texture, Motion},
    ui::{cmdline::Mode, Ui},
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
    display_info: Option<DisplayInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct DisplayInfo {
    start_position: Vec2<f32>,
    target_position: Vec2<f32>,
    elapsed: f32,
    fill: Vec2<f32>,
    cursor_size: Vec2<f32>,
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
                    format: Texture::LINEAR_FORMAT,
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
            fragment_push_constants: FragmentPushConstants::default(),
            display_info: None,
        }
    }

    pub fn update(
        &mut self,
        device: &wgpu::Device,
        ui: &Ui,
        kind: CursorKind,
        cell_size: Vec2<f32>,
        monochrome_target: &wgpu::TextureView,
    ) {
        let (fg, bg) = ui
            .highlight_groups
            .get("Cursor")
            .and_then(|hl_id| ui.highlights.get((*hl_id) as usize))
            .and_then(|hl| {
                let fg = hl.foreground?;
                let bg = hl.background?;
                Some((fg, bg))
            })
            .unwrap_or((
                ui.default_colors.rgb_fg.unwrap_or_default(),
                ui.default_colors.rgb_bg.unwrap_or_default(),
            ));

        self.fragment_push_constants = FragmentPushConstants {
            fg: bg.into_linear(),
            bg: fg.into_linear(),
            size: cell_size,
            speed: 0.,
            padding: 0.,
        };

        self.bind_group = bind_group(
            device,
            &self.bind_group_layout,
            monochrome_target,
            &self.sampler,
        );

        let position = match kind {
            CursorKind::Normal => ui
                .cursor
                .enabled
                .then_some(ui.position(ui.cursor.grid) + ui.cursor.pos.cast_as()),
            CursorKind::Cmdline => ui.cmdline.mode.as_ref().map(|mode| match mode {
                Mode::Normal { levels } => {
                    // We guarantee at least one level if the mode is Some
                    let level = levels.last().unwrap();
                    let mut pos =
                        Vec2::new(level.cursor_pos as i64, -(level.content_lines.len() as i64));
                    for line in level.content_lines.iter() {
                        pos.y += 1;
                        let line_len = line
                            .chunks
                            .iter()
                            .fold(0, |acc, chunk| acc + chunk.text_chunk.len());
                        if line_len < pos.x as usize {
                            pos.x -= line_len as i64;
                        } else {
                            break;
                        }
                    }
                    pos.x += level.prompt.len() as i64 + 1;
                    let base = Vec2::new(0, ui.grids[0].contents().size.y - 1);
                    pos.cast_as::<f32>() + base.cast_as()
                }

                Mode::Block {
                    previous_lines: _,
                    current_line: _,
                } => todo!(),
            }),
        };

        let position: Option<Vec2<f32>> = position.map(|pos| pos.cast_as());

        let mode = &ui.modes[ui.current_mode as usize];
        let fill = mode.cell_percentage.unwrap_or(10) as f32 / 100.0;
        let fill = match mode.cursor_shape.unwrap_or(CursorShape::Block) {
            CursorShape::Block => Vec2::new(1.0, 1.0),
            CursorShape::Horizontal => Vec2::new(1.0, fill),
            CursorShape::Vertical => Vec2::new(fill, 1.0),
        };
        let cursor_size = cell_size - 1.;

        self.display_info = match (position, self.display_info.as_ref()) {
            (None, None) | (None, Some(_)) => None,
            (Some(position), None) => {
                let position = position * cell_size;
                Some(DisplayInfo {
                    start_position: position,
                    target_position: position,
                    elapsed: 1.0,
                    fill,
                    cursor_size,
                })
            }
            (Some(position), Some(display_info)) => {
                let new_target = cell_size * position;
                let (start_position, elapsed) = if new_target != display_info.target_position {
                    let current_position = display_info
                        .start_position
                        .lerp(display_info.target_position, t(display_info));
                    (current_position, 0.0)
                } else {
                    (display_info.start_position, display_info.elapsed)
                };

                Some(DisplayInfo {
                    start_position,
                    target_position: new_target,
                    elapsed,
                    fill,
                    cursor_size,
                })
            }
        };
    }

    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        color_target: &wgpu::TextureView,
        delta_seconds: f32,
        target_size: Vec2<f32>,
        cell_size: Vec2<f32>,
    ) -> Motion {
        let Some(display_info) = self.display_info.as_mut() else {
            return Motion::Still;
        };

        display_info.elapsed += delta_seconds;
        let t = t(display_info).min(1.);
        let current_position = display_info
            .start_position
            .lerp(display_info.target_position, t);
        self.fragment_push_constants.speed = 0.;
        self.fragment_push_constants.size = cell_size;
        let (motion, transform) = if t >= 1.0 {
            (Motion::Still, Mat3::IDENTITY)
        } else {
            let toward = display_info.target_position - display_info.start_position;
            let length = toward.length();
            let direction = if length < 0.25 {
                Vec2::new(1.0, 0.0)
            } else {
                toward / length
            };
            let angle = f32::atan2(direction.x, direction.y);

            let cell_diagonal = cell_size.length();
            let dir = direction * (1. - t);
            let translate = Vec2::splat(0.5) - dir / cell_diagonal * 4.;
            let transform = Mat3::translate(translate)
                * Mat3::rotate(-angle)
                * Mat3::scale(Vec2::new(1.0, length * (1. - t) / 2.))
                * Mat3::rotate(angle)
                * Mat3::translate(-translate);

            self.fragment_push_constants.speed = (1. - t) * length / cell_diagonal;
            (Motion::Animating, transform)
        };

        let transform = Mat3::scale(target_size.map(f32::recip))
            * Mat3::translate(current_position)
            * transform;

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Cursor render pass"),
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
        render_pass.set_push_constants(
            wgpu::ShaderStages::VERTEX,
            0,
            cast_slice(&[VertexPushConstants {
                transform,
                fill: display_info.fill,
                cursor_size: display_info.cursor_size / target_size,
            }]),
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
    fill: Vec2<f32>,
    cursor_size: Vec2<f32>,
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
    size: Vec2<f32>,
    speed: f32,
    padding: f32,
}

impl VertexPushConstants {
    pub const SIZE: u32 = size_of::<Self>() as u32;
}

pub enum CursorKind {
    Normal,
    Cmdline,
}

fn t(display_info: &DisplayInfo) -> f32 {
    let length = (display_info.target_position - display_info.start_position).length();
    if length < 0.25 {
        1.0
    } else {
        let length = length.sqrt() / 100.;
        let normal = (display_info.elapsed / length).min(1.);
        let t = 1.0 - normal;
        1.0 - t * t
    }
}
