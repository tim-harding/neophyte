use crate::{rendering::texture::Texture, text::cache::Cached};
use bytemuck::cast_slice;
use std::num::NonZeroU32;
use wgpu::util::DeviceExt;

pub struct GlyphBindGroup {
    textures: Vec<Texture>,
    next_glyph_to_upload: usize,
    sampler: wgpu::Sampler,
    deferred: Option<Deferred>,
}

struct Deferred {
    bind_group: wgpu::BindGroup,
    layout: wgpu::BindGroupLayout,
}

impl GlyphBindGroup {
    pub fn new(device: &wgpu::Device) -> Self {
        GlyphBindGroup {
            textures: vec![],
            next_glyph_to_upload: 0,
            sampler: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Glyph sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }),
            deferred: None,
        }
    }

    pub fn clear(&mut self) {
        self.textures.clear();
        self.next_glyph_to_upload = 0;
        self.deferred = None;
    }

    pub fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_format: wgpu::TextureFormat,
        cached_glyphs: &Cached,
    ) {
        if self.next_glyph_to_upload == cached_glyphs.data.len() {
            return;
        }

        for (data, size) in cached_glyphs
            .data
            .iter()
            .zip(cached_glyphs.size.iter())
            .skip(self.next_glyph_to_upload)
        {
            self.textures.push(Texture::with_data(
                device,
                queue,
                data.as_slice(),
                *size,
                texture_format,
            ));
        }

        self.next_glyph_to_upload = self.textures.len();
        let views: Vec<_> = self.textures.iter().map(|texture| &texture.view).collect();
        let tex_count = Some(NonZeroU32::new(self.textures.len() as u32).unwrap());

        let font_info_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Glyph info buffer"),
            contents: cast_slice(cached_glyphs.size.as_slice()),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                    count: tex_count,
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

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Glyph bind group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(views.as_slice()),
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

        self.deferred = Some(Deferred {
            bind_group,
            layout: bind_group_layout,
        });
    }

    pub fn layout(&self) -> Option<&wgpu::BindGroupLayout> {
        self.deferred.as_ref().map(|deferred| &deferred.layout)
    }

    pub fn bind_group(&self) -> Option<&wgpu::BindGroup> {
        self.deferred.as_ref().map(|deferred| &deferred.bind_group)
    }
}
