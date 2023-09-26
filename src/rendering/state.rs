use super::{
    blit_render_pipeline::BlitRenderPipeline,
    cell_fill_pipeline::CellFillPipeline,
    cursor_bg::CursorBg,
    depth_texture::DepthTexture,
    glyph_bind_group::GlyphBindGroup,
    glyph_pipeline::GlyphPipeline,
    grid::{self, Grid},
    grid_bind_group_layout::GridBindGroupLayout,
    highlights::HighlightsBindGroup,
    texture::Texture,
};
use crate::{
    text::{cache::FontCache, fonts::Fonts},
    ui::Ui,
    util::vec2::Vec2,
};
use bytemuck::{cast_slice, Pod, Zeroable};
use std::sync::Arc;
use swash::shape::ShapeContext;
use wgpu::include_wgsl;
use winit::window::Window;

pub const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

pub struct RenderState {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
    surface_format: wgpu::TextureFormat,
    cursor_bg: CursorBg,
    shape_context: ShapeContext,
    font_cache: FontCache,
    grids: Vec<grid::Grid>,
    monochrome_pipeline: GlyphPipeline,
    emoji_pipeline: GlyphPipeline,
    monochrome_bind_group: GlyphBindGroup,
    emoji_bind_group: GlyphBindGroup,
    cell_fill_pipeline: CellFillPipeline,
    highlights: HighlightsBindGroup,
    grid_bind_group_layout: GridBindGroupLayout,
    draw_order_index_cache: Vec<usize>,
    shared_push_constants: SharedPushConstants,
    blit_render_pipeline: BlitRenderPipeline,
    target_texture: Texture,
    depth_texture: DepthTexture,
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
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: surface_size.x,
            height: surface_size.y,
            present_mode: surface_caps.present_modes[0], // Vsync
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);

        let highlights = HighlightsBindGroup::new(&device);
        let grid_bind_group_layout = GridBindGroupLayout::new(&device);
        let grid_dimensions = (surface_size / cell_size) * cell_size;
        let target_texture = Texture::target(&device, grid_dimensions, TARGET_FORMAT);
        Self {
            blit_render_pipeline: BlitRenderPipeline::new(
                &device,
                surface_config.format,
                &target_texture.view,
            ),
            depth_texture: DepthTexture::new(&device, grid_dimensions),
            target_texture,
            cursor_bg: CursorBg::new(&device, TARGET_FORMAT),
            shape_context: ShapeContext::new(),
            font_cache: FontCache::new(),
            monochrome_pipeline: GlyphPipeline::new(
                device.create_shader_module(include_wgsl!("glyph.wgsl")),
            ),
            emoji_pipeline: GlyphPipeline::new(
                device.create_shader_module(include_wgsl!("emoji.wgsl")),
            ),
            monochrome_bind_group: GlyphBindGroup::new(&device),
            emoji_bind_group: GlyphBindGroup::new(&device),
            cell_fill_pipeline: CellFillPipeline::new(
                &device,
                &highlights.layout(),
                &grid_bind_group_layout.bind_group_layout,
                TARGET_FORMAT,
            ),
            grid_bind_group_layout,
            grids: vec![],
            highlights,
            draw_order_index_cache: vec![],
            shared_push_constants: SharedPushConstants::default(),
            device,
            queue,
            surface,
            surface_config,
            surface_format,
        }
    }

    pub fn update(&mut self, ui: &Ui, fonts: &mut Fonts) {
        let cell_size = fonts.metrics().into_pixels().cell_size();
        let surface_size = self.surface_size();
        let target_size = (surface_size / cell_size) * cell_size;
        self.cursor_bg.update(ui, target_size, cell_size.into());

        let mut i = 0;
        while let Some(grid) = self.grids.get(i) {
            if ui.grid_index(grid.id).is_ok() {
                i += 1;
            } else {
                self.grids.remove(i);
            }
        }

        for ui_grid in ui.grids.iter() {
            let index = match self
                .grids
                .binary_search_by(|probe| probe.id.cmp(&ui_grid.id))
            {
                Ok(index) => index,
                Err(index) => {
                    self.grids.insert(index, Grid::new(ui_grid.id));
                    index
                }
            };
            let grid = &mut self.grids[index];

            if ui_grid.dirty {
                grid.update_content(
                    &self.device,
                    &self.queue,
                    ui_grid,
                    &ui.highlights,
                    fonts,
                    &mut self.font_cache,
                    &mut self.shape_context,
                    &self.grid_bind_group_layout.bind_group_layout,
                );
            }

            let z = 1.0
                - ui.draw_order
                    .iter()
                    .position(|&id| id == ui_grid.id)
                    .unwrap_or(0) as f32
                    / ui.draw_order.len() as f32;

            grid.update_grid_info(fonts, ui_grid, ui.position(ui_grid.id), z);
        }

        self.highlights.update(ui, &self.device);

        self.monochrome_bind_group.update(
            &self.device,
            &self.queue,
            wgpu::TextureFormat::R8Unorm,
            &self.font_cache.monochrome,
        );
        if let Some(monochrome_bind_group_layout) = self.monochrome_bind_group.layout() {
            self.monochrome_pipeline.update(
                &self.device,
                &self.highlights.layout(),
                monochrome_bind_group_layout,
                &self.grid_bind_group_layout.bind_group_layout,
            );
        }

        self.emoji_bind_group.update(
            &self.device,
            &self.queue,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            &self.font_cache.emoji,
        );
        if let Some(emoji_bind_group_layout) = self.emoji_bind_group.layout() {
            self.emoji_pipeline.update(
                &self.device,
                &self.highlights.layout(),
                emoji_bind_group_layout,
                &self.grid_bind_group_layout.bind_group_layout,
            );
        }

        self.shared_push_constants = SharedPushConstants {
            surface_size: target_size,
            cell_size,
        };
    }

    pub fn resize(&mut self, new_size: Vec2<u32>, cell_size: Vec2<u32>) {
        if new_size.x > 0 && new_size.y > 0 {
            self.surface_config.width = new_size.x;
            self.surface_config.height = new_size.y;
            self.surface.configure(&self.device, &self.surface_config);
        }
        let texture_size = (new_size / cell_size) * cell_size;
        self.target_texture = Texture::target(&self.device, texture_size, TARGET_FORMAT);
        self.blit_render_pipeline.update(
            &self.device,
            self.surface_format,
            &self.target_texture.view,
            texture_size,
            new_size,
        );
        self.depth_texture = DepthTexture::new(&self.device, texture_size);
    }

    pub fn render(&mut self, draw_order: &[u64]) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render encoder"),
            });

        let Some(highlights_bind_group) = self.highlights.bind_group() else {
            return Ok(());
        };

        self.draw_order_index_cache.clear();
        for &id in draw_order.iter().rev() {
            let i = self
                .grids
                .binary_search_by(|probe| probe.id.cmp(&{ id }))
                .unwrap();
            self.draw_order_index_cache.push(i);
        }

        let grids = || self.draw_order_index_cache.iter().map(|&i| &self.grids[i]);

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.target_texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.highlights.clear_color()),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            self.cursor_bg.render(&mut render_pass);

            render_pass.set_pipeline(&self.cell_fill_pipeline.pipeline);
            render_pass.set_bind_group(0, highlights_bind_group, &[]);
            render_pass.set_push_constants(
                wgpu::ShaderStages::VERTEX,
                0,
                cast_slice(&[self.shared_push_constants]),
            );
            for grid in grids() {
                if let Some(bg_bind_group) = &grid.bg_bind_group {
                    render_pass.set_bind_group(1, bg_bind_group, &[]);
                    render_pass.set_push_constants(
                        wgpu::ShaderStages::VERTEX,
                        SharedPushConstants::SIZE as u32,
                        cast_slice(&[grid.grid_info]),
                    );
                    render_pass.draw(0..grid.bg_count * 6, 0..1);
                }
            }

            if let (Some(pipeline), Some(glyph_bind_group)) = (
                self.monochrome_pipeline.pipeline(),
                self.monochrome_bind_group.bind_group(),
            ) {
                render_pass.set_pipeline(&pipeline);
                render_pass.set_bind_group(0, highlights_bind_group, &[]);
                render_pass.set_bind_group(1, glyph_bind_group, &[]);
                render_pass.set_push_constants(
                    wgpu::ShaderStages::VERTEX,
                    0,
                    cast_slice(&[self.shared_push_constants]),
                );
                for grid in grids() {
                    if let Some(monochrome_bind_group) = &grid.monochrome_bind_group {
                        render_pass.set_bind_group(2, monochrome_bind_group, &[]);
                        render_pass.set_push_constants(
                            wgpu::ShaderStages::VERTEX,
                            SharedPushConstants::SIZE as u32,
                            cast_slice(&[grid.grid_info]),
                        );
                        render_pass.draw(0..grid.glyph_count * 6, 0..1);
                    }
                }
            }

            if let (Some(pipeline), Some(glyph_bind_group)) = (
                self.emoji_pipeline.pipeline(),
                self.emoji_bind_group.bind_group(),
            ) {
                render_pass.set_pipeline(&pipeline);
                render_pass.set_bind_group(0, highlights_bind_group, &[]);
                render_pass.set_bind_group(1, &glyph_bind_group, &[]);
                render_pass.set_push_constants(
                    wgpu::ShaderStages::VERTEX,
                    0,
                    cast_slice(&[self.shared_push_constants]),
                );
                for grid in grids() {
                    if let Some(emoji_bind_group) = &grid.emoji_bind_group {
                        render_pass.set_bind_group(2, emoji_bind_group, &[]);
                        render_pass.set_push_constants(
                            wgpu::ShaderStages::VERTEX,
                            SharedPushConstants::SIZE as u32,
                            cast_slice(&[grid.grid_info]),
                        );
                        render_pass.draw(0..grid.glyph_count * 6, 0..1);
                    }
                }
            }
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Blit to screen"),
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

            render_pass.set_pipeline(&self.blit_render_pipeline.pipeline);
            render_pass.set_bind_group(0, &self.blit_render_pipeline.bind_group, &[]);
            render_pass.set_push_constants(
                wgpu::ShaderStages::VERTEX,
                0,
                cast_slice(&[self.blit_render_pipeline.push_constants]),
            );
            render_pass.draw(0..6, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    pub fn rebuild_swap_chain(&mut self, cell_size: Vec2<u32>) {
        self.resize(self.surface_size().into(), cell_size);
    }

    pub fn clear(&mut self) {
        self.emoji_bind_group.clear();
        self.emoji_pipeline.clear();
        self.monochrome_bind_group.clear();
        self.monochrome_pipeline.clear();
    }

    pub fn surface_size(&self) -> Vec2<u32> {
        Vec2::new(self.surface_config.width, self.surface_config.height)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct SharedPushConstants {
    pub surface_size: Vec2<u32>,
    pub cell_size: Vec2<u32>,
}

impl SharedPushConstants {
    pub const SIZE: usize = std::mem::size_of::<Self>();
}
