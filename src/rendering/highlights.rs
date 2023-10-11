use crate::{event::hl_attr_define::Rgb, ui::Ui, util::srgb};
use bytemuck::{cast_slice, Pod, Zeroable};
use wgpu::util::DeviceExt;

pub struct Highlights {
    highlights: Vec<HighlightInfo>,
    clear_color: wgpu::Color,
    layout: wgpu::BindGroupLayout,
    bind_group: Option<wgpu::BindGroup>,
}

impl Highlights {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            highlights: vec![],
            clear_color: wgpu::Color::BLACK,
            bind_group: None,
            layout: device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

    pub fn update(&mut self, ui: &Ui, device: &wgpu::Device) {
        if !ui.did_highlights_change {
            return;
        }

        let fg_default = ui
            .default_colors
            .rgb_fg
            .unwrap_or(Rgb::new(255, 255, 255))
            .into_linear();
        let bg_default = ui.default_colors.rgb_bg.unwrap_or(Rgb::new(0, 0, 0));
        self.clear_color = wgpu::Color {
            r: srgb(bg_default.r()) as f64,
            g: srgb(bg_default.g()) as f64,
            b: srgb(bg_default.b()) as f64,
            a: 1.0,
        };
        let bg_default = bg_default.into_linear();

        self.highlights.resize(
            ui.highlights.len().max(1),
            HighlightInfo::new(fg_default, bg_default),
        );

        for (src, dst) in ui.highlights.iter().zip(self.highlights.iter_mut()) {
            if let Some(fg) = src.rgb_attr.foreground {
                dst.fg = fg.into_linear();
            }
            if let Some(bg) = src.rgb_attr.background {
                dst.bg = bg.into_linear();
            }
        }

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Highlight buffer"),
            contents: cast_slice(self.highlights.as_slice()),
            usage: wgpu::BufferUsages::STORAGE,
        });

        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Highlights bind group"),
            layout: &self.layout,
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

    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }

    pub fn bind_group(&self) -> Option<&wgpu::BindGroup> {
        self.bind_group.as_ref()
    }

    pub fn clear_color(&self) -> wgpu::Color {
        self.clear_color
    }
}

// TODO: Split into bind groups for FG and BG highlights for locality
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct HighlightInfo {
    pub fg: [f32; 4],
    pub bg: [f32; 4],
}

impl HighlightInfo {
    pub fn new(fg: [f32; 4], bg: [f32; 4]) -> Self {
        Self { fg, bg }
    }
}
