use super::nearest_sampler;
use crate::{rendering::texture::Texture, text::cache::Cached};
use bytemuck::cast_slice;
use neophyte_linalg::Vec2;
use wgpu::util::DeviceExt;

pub struct GlyphBindGroup {
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    last_revision: u32,
    bind_group: Option<wgpu::BindGroup>,
}

impl GlyphBindGroup {
    pub fn new(device: &wgpu::Device) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Glyph bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        GlyphBindGroup {
            layout,
            sampler: nearest_sampler(device),
            last_revision: 0,
            bind_group: None,
        }
    }

    pub fn clear(&mut self) {
        self.bind_group = None;
    }

    pub fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_format: wgpu::TextureFormat,
        cached_glyphs: &Cached,
    ) {
        if self.last_revision == cached_glyphs.revision || cached_glyphs.info.is_empty() {
            return;
        }

        // TODO: Reuse buffer if size did not change
        let texture = Texture::with_data(
            device,
            queue,
            cached_glyphs.atlas.data(),
            Vec2::splat(cached_glyphs.atlas.size()),
            texture_format,
        );

        let font_info_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Glyph info buffer"),
            contents: cast_slice(cached_glyphs.info.as_slice()),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Glyph bind group"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &font_info_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });

        self.bind_group = Some(bind_group);
    }

    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }

    pub fn bind_group(&self) -> Option<&wgpu::BindGroup> {
        self.bind_group.as_ref()
    }
}
