//! Paints and animates the cursor. In order to display the text over the
//! cursor, we reuse the alpha from rendering the monochrome text such that the
//! cursor appears to go underneath.

use crate::{
    event::{
        mode_info_set::{CursorShape, ModeInfo},
        rgb::Rgb,
    },
    rendering::{nearest_sampler, texture::Texture, Motion},
    ui::{cmdline::Mode, Ui},
    util::{
        mat3::Mat3,
        nice_s_curve,
        vec2::{CellVec, PixelVec, Vec2},
    },
};
use bytemuck::{cast_slice, Pod, Zeroable};
use std::{mem::size_of, time::Duration};
use wgpu::include_wgsl;

pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    fragment_push_constants: FragmentPushConstants,
    display_info: Option<DisplayInfo>,
    speed: f32,
    transform: Mat3,
    show: bool,
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
            transform: Mat3::IDENTITY,
            show: false,
            speed: 0.,
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
        let mode = &ui.modes[ui.current_mode as usize];
        let (fg, bg) = mode
            .attr_id
            .and_then(|hl_id| ui.highlights.get(hl_id as usize))
            .and_then(|hl| (*hl).as_ref())
            .map(|hl| {
                let fg = hl
                    .foreground
                    .unwrap_or(ui.default_colors.rgb_fg.unwrap_or(Rgb::WHITE));
                let bg = hl
                    .background
                    .unwrap_or(ui.default_colors.rgb_bg.unwrap_or(Rgb::BLACK));
                let blend = hl.blend();
                (bg.into_srgb(blend), fg.into_srgb(blend))
            })
            .unwrap_or((
                ui.default_colors
                    .rgb_fg
                    .unwrap_or(Rgb::WHITE)
                    .into_srgb(1.0),
                ui.default_colors
                    .rgb_bg
                    .unwrap_or(Rgb::BLACK)
                    .into_srgb(1.0),
            ));

        self.fragment_push_constants = FragmentPushConstants {
            fg,
            bg,
            size: cell_size,
            stretch: 0.,
            padding: 0.,
        };

        self.bind_group = bind_group(
            device,
            &self.bind_group_layout,
            monochrome_target,
            &self.sampler,
        );

        let position = match kind {
            CursorKind::Normal => {
                if ui.cursor.enabled {
                    ui.position(ui.cursor.grid)
                        .map(|position| position + ui.cursor.pos.cast_as())
                } else {
                    None
                }
            }

            CursorKind::Cmdline => ui.cmdline.mode.as_ref().map(|mode| match mode {
                Mode::Normal { levels } => {
                    // We guarantee at least one level if the mode is Some
                    let level = levels.last().unwrap();
                    let mut pos =
                        CellVec::new(level.cursor_pos as i64, -(level.content_lines.len() as i64));
                    for line in level.content_lines.iter() {
                        pos.0.y += 1;
                        let line_len = line
                            .chunks
                            .iter()
                            .fold(0, |acc, chunk| acc + chunk.text_chunk.len());
                        if line_len < pos.0.x as usize {
                            pos.0.x -= line_len as i64;
                        } else {
                            break;
                        }
                    }
                    pos.0.x += level.prompt.len() as i64 + 1;
                    let base = CellVec::new(0, ui.grids[0].contents().size.0.y - 1);
                    pos.cast_as::<f32>() + base.cast_as()
                }

                Mode::Block {
                    previous_lines: _,
                    current_line: _,
                } => todo!(),
            }),
        };

        let position = position.map(|pos| pos.cast_as::<f32>());

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
                let position = position.into_pixels(cell_size);
                Some(DisplayInfo {
                    start_position: position,
                    target_position: position,
                    elapsed: Duration::ZERO,
                    fill,
                    cursor_size,
                    blink_rate: BlinkRate::from_mode_info(mode),
                })
            }
            (Some(position), Some(display_info)) => {
                let new_target = position.into_pixels(cell_size);
                let (start_position, elapsed) = if new_target != display_info.target_position {
                    let current_position = display_info
                        .start_position
                        .lerp(display_info.target_position, t(display_info, self.speed));
                    (current_position, Duration::ZERO)
                } else {
                    (display_info.start_position, display_info.elapsed)
                };

                let blink_rate = BlinkRate::from_mode_info(mode);
                Some(DisplayInfo {
                    start_position,
                    target_position: new_target,
                    elapsed,
                    fill,
                    cursor_size,
                    blink_rate,
                })
            }
        };
    }

    pub fn advance(&mut self, delta_time: Duration, speed: f32, cell_size: Vec2<f32>) -> Motion {
        self.speed = speed;
        let Some(display_info) = self.display_info.as_mut() else {
            return Motion::Still;
        };

        display_info.elapsed += delta_time;
        let t = t(display_info, speed).min(1.);
        let current_position = display_info
            .start_position
            .lerp(display_info.target_position, t);
        self.fragment_push_constants.stretch = 0.;
        self.fragment_push_constants.size = cell_size;
        let (motion, transform) =
            if t >= 1.0 || display_info.target_position == display_info.start_position {
                (Motion::Still, Mat3::IDENTITY)
            } else {
                let toward = display_info.target_position - display_info.start_position;
                let length = toward.length();
                let direction = if length < 0.25 {
                    PixelVec::new(1.0, 0.0)
                } else {
                    toward / length
                };
                let angle = f32::atan2(direction.0.x, direction.0.y);

                let cell_diagonal = cell_size.length();
                let dir = direction * (1. - t);
                let translate = PixelVec::splat(0.5) - dir / cell_diagonal * 4.;
                let transform = Mat3::translate(translate.0)
                    * Mat3::rotate(-angle)
                    * Mat3::scale(Vec2::new(1.0, length * (1. - t) / 2.))
                    * Mat3::rotate(angle)
                    * Mat3::translate(-translate.0);

                self.fragment_push_constants.stretch = (1. - t) * length / cell_diagonal;
                (Motion::Animating, transform)
            };

        self.transform = Mat3::translate(current_position.0) * transform;

        let (motion, show) = if let Some(blink_rate) = display_info.blink_rate {
            let wait = Duration::from_millis(blink_rate.wait as u64);
            if display_info.elapsed < wait {
                let until_cycle = wait - display_info.elapsed;
                (motion.soonest(Motion::Delay(until_cycle)), true)
            } else {
                let since_wait = (display_info.elapsed - wait).as_millis();
                let on = blink_rate.on as u128;
                let off = blink_rate.off as u128;
                let cycle_duration = on + off;
                let cycle_time = since_wait % cycle_duration;
                if cycle_time < on {
                    let until_off: u64 = (on - cycle_time).try_into().unwrap_or(0);
                    let until_off = Duration::from_millis(until_off);
                    (motion.soonest(Motion::Delay(until_off)), true)
                } else {
                    let off_time = cycle_time - on;
                    let until_on: u64 = (off - off_time).try_into().unwrap_or(0);
                    let until_on = Duration::from_millis(until_on);
                    (motion.soonest(Motion::Delay(until_on)), false)
                }
            }
        } else {
            (motion, true)
        };
        self.show = show;

        motion
    }

    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        color_target: &wgpu::TextureView,
        target_size: PixelVec<f32>,
    ) {
        let Some(display_info) = self.display_info.as_mut() else {
            return;
        };

        let transform = Mat3::scale(target_size.map(f32::recip).0) * self.transform;

        if self.show {
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
                    // TODO: ScreenVec type for vector in screenspace?
                    cursor_size: display_info.cursor_size / target_size.0,
                }]),
            );
            render_pass.set_push_constants(
                wgpu::ShaderStages::FRAGMENT,
                VertexPushConstants::SIZE,
                cast_slice(&[self.fragment_push_constants]),
            );
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.draw(0..6, 0..1);
        }
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

#[derive(Debug, Clone, Copy, PartialEq)]
struct DisplayInfo {
    start_position: PixelVec<f32>,
    target_position: PixelVec<f32>,
    elapsed: Duration,
    fill: Vec2<f32>,
    cursor_size: Vec2<f32>,
    blink_rate: Option<BlinkRate>,
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

#[repr(C, align(16))]
#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
struct FragmentPushConstants {
    fg: [f32; 4],
    bg: [f32; 4],
    size: Vec2<f32>,
    stretch: f32,
    padding: f32,
}

impl VertexPushConstants {
    pub const SIZE: u32 = size_of::<Self>() as u32;
}

pub enum CursorKind {
    Normal,
    Cmdline,
}

fn t(display_info: &DisplayInfo, speed: f32) -> f32 {
    let length = (display_info.target_position - display_info.start_position).length();
    if length < 0.25 {
        1.0
    } else {
        nice_s_curve(display_info.elapsed.as_secs_f32() * speed, length)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BlinkRate {
    on: u32,
    off: u32,
    wait: u32,
}

impl BlinkRate {
    pub fn from_mode_info(mode: &ModeInfo) -> Option<Self> {
        match (mode.blinkon, mode.blinkoff, mode.blinkwait) {
            (Some(on), Some(off), Some(wait)) if on > 0 && off > 0 && wait > 0 => {
                Some(Self { on, off, wait })
            }
            _ => None,
        }
    }
}
