use super::{
    depth_texture::DepthTexture,
    glyph_push_constants::GlyphPushConstants,
    grids::Grids,
    highlights::Highlights,
    pipelines::{blend, cell_fill, cursor, emoji, gamma_blit, monochrome},
    texture::Texture,
};
use crate::{
    text::{cache::FontCache, fonts::Fonts},
    ui::Ui,
    util::vec2::Vec2,
};
use bytemuck::cast_slice;
use std::sync::Arc;
use swash::shape::ShapeContext;
use winit::window::Window;

pub const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

pub struct RenderState {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
    pipelines: Pipelines,
    targets: Targets,
    grids: Grids,
    shape_context: ShapeContext,
    font_cache: FontCache,
    highlights: Highlights,
}

struct Targets {
    monochrome: Texture,
    color: Texture,
    depth: DepthTexture,
}

struct Pipelines {
    cursor: cursor::Pipeline,
    blend: blend::Pipeline,
    cell_fill: cell_fill::Pipeline,
    emoji: emoji::Pipeline,
    gamma_blit: gamma_blit::Pipeline,
    monochrome: monochrome::Pipeline,
}

impl RenderState {
    pub async fn new(window: Arc<Window>, cell_size: Vec2<u32>) -> Self {
        let surface_size: Vec2<u32> = window.inner_size().into();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(window.as_ref()) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::TEXTURE_BINDING_ARRAY
                    | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
                    | wgpu::Features::UNIFORM_BUFFER_AND_STORAGE_TEXTURE_ARRAY_NON_UNIFORM_INDEXING
                    | wgpu::Features::PUSH_CONSTANTS,
                limits: adapter.limits(),
            },
            None,
        )
        .await
        .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_caps
                .formats
                .iter()
                .copied()
                .find(|f| f.is_srgb())
                .unwrap_or(surface_caps.formats[0]),
            width: surface_size.x,
            height: surface_size.y,
            present_mode: surface_caps.present_modes[0], // Vsync
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);

        let highlights = Highlights::new(&device);
        let grid_dimensions = (surface_size / cell_size) * cell_size;

        let grids = Grids::new(&device);

        let targets = Targets {
            monochrome: Texture::target(&device, grid_dimensions, TARGET_FORMAT),
            color: Texture::target(&device, grid_dimensions, TARGET_FORMAT),
            depth: DepthTexture::new(&device, grid_dimensions),
        };

        let pipelines = Pipelines {
            cursor: cursor::Pipeline::new(&device, &targets.monochrome.view),
            blend: blend::Pipeline::new(&device, &targets.color.view),
            cell_fill: cell_fill::Pipeline::new(
                &device,
                highlights.layout(),
                &grids.bind_group_layout(),
                TARGET_FORMAT,
            ),
            emoji: emoji::Pipeline::new(&device),
            gamma_blit: gamma_blit::Pipeline::new(
                &device,
                surface_config.format,
                &targets.color.view,
            ),
            monochrome: monochrome::Pipeline::new(&device),
        };

        Self {
            pipelines,
            targets,
            shape_context: ShapeContext::new(),
            font_cache: FontCache::new(),
            grids: Grids::new(&device),
            highlights,
            device,
            queue,
            surface,
            surface_config,
        }
    }

    pub fn update(&mut self, ui: &Ui, fonts: &mut Fonts) {
        let cell_size = fonts.metrics().into_pixels().cell_size();
        let target_size = (self.surface_size() / cell_size) * cell_size;
        self.pipelines.cursor.update(
            &self.device,
            ui,
            target_size,
            cell_size.into(),
            &self.targets.monochrome.view,
        );
        self.grids.update(
            &self.device,
            &self.queue,
            ui,
            fonts,
            &mut self.font_cache,
            &mut self.shape_context,
        );

        self.highlights.update(ui, &self.device);

        self.pipelines.monochrome.update(
            &self.device,
            &self.queue,
            &self.font_cache.monochrome,
            self.highlights.layout(),
            &self.grids.bind_group_layout(),
        );

        self.pipelines.emoji.update(
            &self.device,
            &self.queue,
            &self.font_cache.emoji,
            &self.grids.bind_group_layout(),
        );

        self.pipelines
            .blend
            .update(&self.device, &self.targets.monochrome.view);
    }

    pub fn resize(&mut self, new_size: Vec2<u32>, cell_size: Vec2<u32>) {
        if new_size == Vec2::default() {
            return;
        }
        self.surface_config.width = new_size.x;
        self.surface_config.height = new_size.y;
        self.surface.configure(&self.device, &self.surface_config);
        let texture_size = (new_size / cell_size) * cell_size;
        self.targets.monochrome = Texture::target(&self.device, texture_size, TARGET_FORMAT);
        self.targets.color = Texture::target(&self.device, texture_size, TARGET_FORMAT);
        self.pipelines.gamma_blit.update(
            &self.device,
            self.surface_config.format,
            &self.targets.color.view,
            texture_size,
            new_size,
        );
        self.targets.depth = DepthTexture::new(&self.device, texture_size);
    }

    pub fn render(&mut self, cell_size: Vec2<u32>) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render encoder"),
            });

        let target_size = {
            let size = self.targets.color.texture.size();
            Vec2::new(size.width, size.height)
        };

        let Some(highlights_bind_group) = self.highlights.bind_group() else {
            return Ok(());
        };

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Cell fill render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.targets.color.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.highlights.clear_color()),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.targets.depth.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.pipelines.cell_fill.pipeline());
            render_pass.set_bind_group(0, highlights_bind_group, &[]);
            for (z, grid) in self.grids.front_to_back() {
                let Some(bg_bind_group) = &grid.cell_fill_bind_group() else {
                    continue;
                };
                render_pass.set_bind_group(1, bg_bind_group, &[]);
                cell_fill::PushConstants {
                    target_size,
                    cell_size,
                    offset: grid.offset(),
                    grid_width: grid.size().x,
                    z,
                }
                .set(&mut render_pass);
                render_pass.draw(0..grid.cell_fill_count() * 6, 0..1);
            }
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Monochrome render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.targets.monochrome.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.targets.depth.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            if let (Some(pipeline), Some(glyph_bind_group)) = (
                self.pipelines.monochrome.pipeline(),
                self.pipelines.monochrome.bind_group(),
            ) {
                render_pass.set_pipeline(pipeline);
                render_pass.set_bind_group(0, highlights_bind_group, &[]);
                render_pass.set_bind_group(1, glyph_bind_group, &[]);
                for (z, grid) in self.grids.front_to_back() {
                    let Some(monochrome_bind_group) = &grid.monochrome_bind_group() else {
                        continue;
                    };
                    render_pass.set_bind_group(2, monochrome_bind_group, &[]);
                    GlyphPushConstants {
                        target_size,
                        offset: grid.offset(),
                        z,
                    }
                    .set(&mut render_pass);
                    render_pass.draw(0..grid.monochrome_count() * 6, 0..1);
                }
            }
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Blend render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.targets.color.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(self.pipelines.blend.pipeline());
            render_pass.set_bind_group(0, self.pipelines.blend.bind_group(), &[]);
            render_pass.draw(0..6, 0..1);
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Cursor render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.targets.color.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            self.pipelines.cursor.render(&mut render_pass);
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Emoji render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.targets.color.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.targets.depth.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            if let (Some(pipeline), Some(glyph_bind_group)) = (
                self.pipelines.emoji.pipeline(),
                self.pipelines.emoji.bind_group(),
            ) {
                render_pass.set_pipeline(pipeline);
                render_pass.set_bind_group(0, glyph_bind_group, &[]);
                for (z, grid) in self.grids.front_to_back() {
                    let Some(emoji_bind_group) = &grid.emoji_bind_group() else {
                        continue;
                    };
                    render_pass.set_bind_group(1, emoji_bind_group, &[]);
                    GlyphPushConstants {
                        target_size,
                        offset: grid.offset(),
                        z,
                    }
                    .set(&mut render_pass);
                    render_pass.draw(0..grid.emoji_count() * 6, 0..1);
                }
            }
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Blit render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.highlights.clear_color()),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.pipelines.gamma_blit.pipeline);
            render_pass.set_bind_group(0, &self.pipelines.gamma_blit.bind_group, &[]);
            render_pass.set_push_constants(
                wgpu::ShaderStages::VERTEX,
                0,
                cast_slice(&[self.pipelines.gamma_blit.push_constants]),
            );
            render_pass.draw(0..6, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    pub fn rebuild_swap_chain(&mut self, cell_size: Vec2<u32>) {
        self.resize(self.surface_size(), cell_size);
    }

    pub fn clear(&mut self) {
        self.pipelines.emoji.clear();
        self.pipelines.monochrome.clear();
    }

    pub fn surface_size(&self) -> Vec2<u32> {
        Vec2::new(self.surface_config.width, self.surface_config.height)
    }
}
