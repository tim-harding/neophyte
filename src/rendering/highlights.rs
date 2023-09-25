use super::shared::Shared;
use crate::{event::hl_attr_define::Rgb, ui::Ui, util::srgb};
use bytemuck::{cast_slice, Pod, Zeroable};
use wgpu::util::DeviceExt;

pub struct HighlightsBindGroup {
    pub highlights: Vec<HighlightInfo>,
    pub clear_color: wgpu::Color,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: Option<wgpu::BindGroup>,
}

impl HighlightsBindGroup {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            highlights: vec![],
            clear_color: wgpu::Color::BLACK,
            bind_group: None,
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

    pub fn update(&mut self, ui: &Ui, shared: &Shared) {
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
                    fg: fg_default.into_linear(),
                    bg: bg_default.into_linear(),
                },
            )
        }

        for id in ui.new_highlights.iter() {
            let highlight = ui.highlights.get(id).unwrap();
            let i = *id as usize;
            if i + 1 > self.highlights.len() {
                self.highlights.resize(i + 1, HighlightInfo::default());
            }
            self.highlights[i] = HighlightInfo {
                fg: highlight
                    .rgb_attr
                    .foreground
                    .unwrap_or(fg_default)
                    .into_linear(),
                bg: highlight
                    .rgb_attr
                    .background
                    .unwrap_or(bg_default)
                    .into_linear(),
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
                layout: &self.bind_group_layout,
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
