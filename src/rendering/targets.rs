use super::texture::Texture;
use neophyte_linalg::PixelVec;

pub struct Targets {
    pub monochrome: Texture,
    pub color: Texture,
    pub depth: Texture,
    pub png: Texture,
    pub png_staging: wgpu::Buffer,
    pub png_size: PixelVec<u32>,
}

impl Targets {
    pub fn new(device: &wgpu::Device, size: PixelVec<u32>) -> Self {
        let png_size = PixelVec::new(((size.0.x + 63) / 64) * 64, size.0.y);
        Self {
            monochrome: Texture::target(
                device,
                &Texture::descriptor(
                    "Monochrome texture",
                    size.into(),
                    Texture::LINEAR_FORMAT,
                    Texture::ATTACHMENT_AND_BINDING,
                ),
            ),
            color: Texture::target(
                device,
                &Texture::descriptor(
                    "Monochrome texture",
                    size.into(),
                    Texture::LINEAR_FORMAT,
                    Texture::ATTACHMENT_AND_BINDING,
                ),
            ),
            depth: Texture::target(
                device,
                &Texture::descriptor(
                    "Depth texture",
                    size.into(),
                    Texture::DEPTH_FORMAT,
                    wgpu::TextureUsages::RENDER_ATTACHMENT,
                ),
            ),
            png: Texture::target(
                device,
                &Texture::descriptor(
                    "Monochrome texture",
                    png_size.into(),
                    Texture::SRGB_FORMAT,
                    Texture::ATTACHMENT_AND_BINDING | wgpu::TextureUsages::COPY_SRC,
                ),
            ),
            png_staging: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("PNG staging buffer"),
                size: png_size.area() as u64 * 4,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            png_size,
        }
    }
}
