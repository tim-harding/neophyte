use super::ConstantState;
use crate::{event::hl_attr_define::Rgb, ui::Ui};
use bytemuck::{cast_slice, Pod, Zeroable};
use wgpu::util::DeviceExt;

#[derive(Default)]
pub struct Write {
    pub highlights: Vec<HighlightInfo>,
}

impl Write {
    pub fn updates(&mut self, ui: &Ui, constant: &ConstantState) -> Option<Read> {
        let fg_default = ui.default_colors.rgb_fg.unwrap_or(Rgb::new(255, 255, 255));
        let bg_default = ui.default_colors.rgb_bg.unwrap_or(Rgb::new(0, 0, 0));

        let srgb = |n| (n as f64 / 255.0).powf(2.2);
        let clear_color = wgpu::Color {
            r: srgb(bg_default.r()),
            g: srgb(bg_default.g()),
            b: srgb(bg_default.b()),
            a: 1.0,
        };

        let srgb = |n| (n as f32 / 255.0).powf(2.2);
        let srgb = |c: Rgb| [srgb(c.r()), srgb(c.g()), srgb(c.b()), 1.0];
        for highlight in ui.highlights.iter() {
            let i = *highlight.0 as usize;
            if i + 1 > self.highlights.len() {
                self.highlights.resize(i + 1, HighlightInfo::default());
            }
            self.highlights[i] = HighlightInfo {
                fg: srgb(highlight.1.rgb_attr.foreground.unwrap_or(fg_default)),
                bg: srgb(highlight.1.rgb_attr.background.unwrap_or(bg_default)),
            };
        }

        let buffer = constant
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Highlight buffer"),
                contents: cast_slice(self.highlights.as_slice()),
                usage: wgpu::BufferUsages::STORAGE,
            });

        let bind_group = constant
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Highlights bind group"),
                layout: &constant.highlights.bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &buffer,
                        offset: 0,
                        size: None,
                    }),
                }],
            });

        Some(Read {
            clear_color,
            bind_group,
        })
    }
}

pub struct Read {
    pub clear_color: wgpu::Color,
    pub bind_group: wgpu::BindGroup,
}

pub struct Constant {
    pub bind_group_layout: wgpu::BindGroupLayout,
}

pub fn init(device: &wgpu::Device) -> (Write, Constant) {
    (
        Write::default(),
        Constant {
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
        },
    )
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct HighlightInfo {
    pub fg: [f32; 4],
    pub bg: [f32; 4],
}
