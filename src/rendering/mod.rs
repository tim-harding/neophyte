use std::ops::{BitOr, BitOrAssign};

mod depth_texture;
mod glyph_bind_group;
mod glyph_push_constants;
mod grid;
mod grids;
pub mod pipelines;
pub mod state;
mod texture;

pub const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

pub fn nearest_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Glyph sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Motion {
    #[default]
    Still,
    Animating,
}

impl BitOr for Motion {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Still, Self::Still) => Self::Still,
            _ => Self::Animating,
        }
    }
}

impl BitOrAssign for Motion {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}
