use crate::util::vec2::Vec2;

pub struct DepthTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

impl DepthTexture {
    pub fn new(device: &wgpu::Device, size: Vec2<u32>) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth texture"),
            size: wgpu::Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth16Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let view = texture.create_view(&Default::default());
        Self { texture, view }
    }
}
