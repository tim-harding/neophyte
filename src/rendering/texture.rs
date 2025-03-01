use neophyte_linalg::{PixelVec, Vec2};
use wgpu::util::{DeviceExt, TextureDataOrder};

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    pub const LINEAR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
    pub const SRGB_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
    pub const ATTACHMENT_AND_BINDING: wgpu::TextureUsages =
        wgpu::TextureUsages::RENDER_ATTACHMENT.union(wgpu::TextureUsages::TEXTURE_BINDING);

    pub const DEFAULT_DESCRIPTOR: wgpu::TextureDescriptor<'static> = wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: 0,
            height: 0,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: Self::LINEAR_FORMAT,
        usage: wgpu::TextureUsages::empty(),
        view_formats: &[],
    };

    pub const fn descriptor(
        label: &'static str,
        size: wgpu::Extent3d,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
    ) -> wgpu::TextureDescriptor<'static> {
        wgpu::TextureDescriptor {
            label: Some(label),
            size,
            format,
            usage,
            ..Self::DEFAULT_DESCRIPTOR
        }
    }

    pub fn target(device: &wgpu::Device, descriptor: &wgpu::TextureDescriptor) -> Self {
        let texture = device.create_texture(descriptor);
        let view = texture.create_view(&Default::default());
        Self { texture, view }
    }

    pub fn with_data(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        data: &[u8],
        size: Vec2<u32>,
        format: wgpu::TextureFormat,
    ) -> Self {
        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                size: wgpu::Extent3d {
                    width: size.x,
                    height: size.y,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                label: None,
                view_formats: &[],
            },
            TextureDataOrder::LayerMajor,
            data,
        );

        let view = texture.create_view(&Default::default());
        Self { texture, view }
    }

    pub fn size(&self) -> PixelVec<u32> {
        self.texture.size().into()
    }
}
