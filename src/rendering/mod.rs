mod cmdline_grid;
mod glyph_bind_group;
mod glyph_push_constants;
mod grids;
pub mod pipelines;
mod scrolling_grids;
pub mod state;
mod text;
mod texture;

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
    DelayMs(u32),
}

impl Motion {
    pub fn soonest(self, other: Self) -> Self {
        use Motion::*;
        match (self, other) {
            (Animating, _) | (_, Animating) => Animating,
            (DelayMs(ms1), DelayMs(ms2)) => DelayMs(ms1.min(ms2)),
            (DelayMs(ms), Still) | (Still, DelayMs(ms)) => DelayMs(ms),
            (Still, Still) => Still,
        }
    }
}
