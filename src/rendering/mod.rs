mod cmdline_grid;
mod glyph_bind_group;
mod glyph_push_constants;
mod grids;
mod message_grids;
pub mod pipelines;
mod scrolling_grids;
pub mod state;
mod targets;
mod text;
mod texture;
mod wgpu_context;

use std::time::Duration;

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
    Delay(Duration),
}

impl Motion {
    pub fn soonest(self, other: Self) -> Self {
        use Motion::*;
        match (self, other) {
            (Animating, _) | (_, Animating) => Animating,
            (Delay(ms1), Delay(ms2)) => Delay(ms1.min(ms2)),
            (Delay(ms), Still) | (Still, Delay(ms)) => Delay(ms),
            (Still, Still) => Still,
        }
    }
}
