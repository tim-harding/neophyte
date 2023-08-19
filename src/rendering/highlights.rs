use super::shared::Shared;
use crate::{event::hl_attr_define::Rgb, ui::Ui, util::srgb};
use bytemuck::{cast_slice, Pod, Zeroable};
use wgpu::util::DeviceExt;

#[derive(Default)]
pub struct HighlightsBindGroup {
    pub highlights: Vec<HighlightInfo>,
    pub clear_color: wgpu::Color,
    pub bind_group: Option<wgpu::BindGroup>,
}

impl HighlightsBindGroup {
    pub fn update(
        &mut self,
        ui: &Ui,
        highlights_bind_group_layout: &HighlightsBindGroupLayout,
        shared: &Shared,
    ) {
        let fg_default = ui.default_colors.rgb_fg.unwrap_or(Rgb::new(255, 255, 255));
        let bg_default = ui.default_colors.rgb_bg.unwrap_or(Rgb::new(0, 0, 0));

        self.clear_color = wgpu::Color {
            r: srgb(bg_default.r()) as f64,
            g: srgb(bg_default.g()) as f64,
            b: srgb(bg_default.b()) as f64,
            a: 1.0,
        };

        if self.highlights.is_empty() {
            self.highlights.resize(
                1,
                HighlightInfo {
                    fg: [
                        srgb(fg_default.r()),
                        srgb(fg_default.g()),
                        srgb(fg_default.b()),
                        1.0,
                    ],
                    bg: [
                        srgb(bg_default.r()),
                        srgb(bg_default.g()),
                        srgb(bg_default.b()),
                        1.0,
                    ],
                },
            )
        }

        let srgb = |c: Rgb| [srgb(c.r()), srgb(c.g()), srgb(c.b()), 1.0];
        for id in ui.new_highlights.iter() {
            let highlight = ui.highlights.get(id).unwrap();
            let i = *id as usize;
            if i + 1 > self.highlights.len() {
                self.highlights.resize(i + 1, HighlightInfo::default());
            }
            self.highlights[i] = HighlightInfo {
                fg: srgb(highlight.rgb_attr.foreground.unwrap_or(fg_default)),
                bg: srgb(highlight.rgb_attr.background.unwrap_or(bg_default)),
            };
        }

        if !ui.new_highlights.is_empty() {
            let buffer = shared
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Highlight buffer"),
                    contents: cast_slice(self.highlights.as_slice()),
                    usage: wgpu::BufferUsages::STORAGE,
                });

            self.bind_group = Some(shared.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Highlights bind group"),
                layout: &highlights_bind_group_layout.bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &buffer,
                        offset: 0,
                        size: None,
                    }),
                }],
            }));
        }
    }
}

// TODO: Split into bind groups for FG and BG highlights
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct HighlightInfo {
    pub fg: [f32; 4],
    pub bg: [f32; 4],
}

pub struct HighlightsBindGroupLayout {
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl HighlightsBindGroupLayout {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            bind_group_layout: device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Highlights bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            }),
        }
    }
}
