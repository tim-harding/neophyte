use std::{fs::File, io::BufWriter};

use super::{
    cmdline_grid::CmdlineGrid,
    grids::Grids,
    pipelines::{blend, cell_fill, cursor, default_fill, emoji, gamma_blit, lines, monochrome},
    text,
    texture::Texture,
    Motion,
};
use crate::{
    event::rgb::Rgb,
    event_handler::settings::Settings,
    text::{cache::FontCache, fonts::Fonts},
    ui::Ui,
    util::vec2::Vec2,
};
use bytemuck::cast_slice;
use swash::shape::ShapeContext;
use winit::window::Window;

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
    clear_color: [f32; 4],
    cmdline_grid: CmdlineGrid,
    text_bind_group_layout: text::bind_group::BindGroup,
    png_count: usize,
}

struct Targets {
    monochrome: Texture,
    color: Texture,
    depth: Texture,
    png: Texture,
    png_staging: wgpu::Buffer,
    png_size: Vec2<u32>,
}

impl Targets {
    pub fn new(device: &wgpu::Device, size: Vec2<u32>) -> Self {
        let png_size = Vec2::new(((size.x + 63) / 64) * 64, size.y);
        Self {
            monochrome: Texture::target(
                &device,
                &Texture::descriptor(
                    "Monochrome texture",
                    size.into(),
                    Texture::LINEAR_FORMAT,
                    Texture::ATTACHMENT_AND_BINDING,
                ),
            ),
            color: Texture::target(
                &device,
                &Texture::descriptor(
                    "Monochrome texture",
                    size.into(),
                    Texture::LINEAR_FORMAT,
                    Texture::ATTACHMENT_AND_BINDING,
                ),
            ),
            depth: Texture::target(
                &device,
                &Texture::descriptor(
                    "Depth texture",
                    size.into(),
                    Texture::DEPTH_FORMAT,
                    wgpu::TextureUsages::RENDER_ATTACHMENT,
                ),
            ),
            png: Texture::target(
                &device,
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

struct Pipelines {
    cursor: cursor::Pipeline,
    cmdline_cursor: cursor::Pipeline,
    blend: blend::Pipeline,
    default_fill: default_fill::Pipeline,
    cell_fill: cell_fill::Pipeline,
    emoji: emoji::Pipeline,
    gamma_blit_final: gamma_blit::Pipeline,
    gamma_blit_png: gamma_blit::Pipeline,
    monochrome: monochrome::Pipeline,
    lines: lines::Pipeline,
}

impl RenderState {
    pub async fn new(window: &Window, cell_size: Vec2<u32>) -> Self {
        let surface_size: Vec2<u32> = window.inner_size().into();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
            flags: wgpu::InstanceFlags::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });

        let surface =
            unsafe { instance.create_surface(window) }.expect("Failed to create graphics surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to get a graphics adapter");

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
        .expect("Failed to get a graphics device");

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
            present_mode: wgpu::PresentMode::AutoVsync,
            // TODO: Set premultiplied and update clear color and cell fill with
            // alpha appropriately
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);

        let grids = Grids::new(&device);

        let target_size: Vec2<u32> = (surface_size / cell_size) * cell_size;
        let targets = Targets::new(&device, target_size);

        Self {
            png_count: 0,
            text_bind_group_layout: text::bind_group::BindGroup::new(&device),
            pipelines: Pipelines {
                cursor: cursor::Pipeline::new(&device, &targets.monochrome.view),
                cmdline_cursor: cursor::Pipeline::new(&device, &targets.monochrome.view),
                blend: blend::Pipeline::new(&device, &targets.color.view),
                default_fill: default_fill::Pipeline::new(&device, Texture::LINEAR_FORMAT),
                cell_fill: cell_fill::Pipeline::new(
                    &device,
                    grids.bind_group_layout(),
                    Texture::LINEAR_FORMAT,
                ),
                emoji: emoji::Pipeline::new(&device),
                gamma_blit_final: gamma_blit::Pipeline::new(
                    &device,
                    surface_config.format,
                    &targets.color.view,
                ),
                gamma_blit_png: gamma_blit::Pipeline::new(
                    &device,
                    Texture::SRGB_FORMAT,
                    &targets.color.view,
                ),
                monochrome: monochrome::Pipeline::new(&device),
                lines: lines::Pipeline::new(
                    &device,
                    grids.bind_group_layout(),
                    Texture::LINEAR_FORMAT,
                ),
            },
            shape_context: ShapeContext::new(),
            font_cache: FontCache::new(),
            grids: Grids::new(&device),
            targets,
            device,
            queue,
            surface,
            surface_config,
            clear_color: [0.; 4],
            cmdline_grid: CmdlineGrid::new(),
        }
    }

    pub fn update(&mut self, ui: &Ui, fonts: &Fonts) {
        self.clear_color = ui.default_colors.rgb_bg.unwrap_or(Rgb::BLACK).into_linear();
        for grid in ui.deleted_grids.iter() {
            self.grids.remove_grid(*grid);
        }

        let fg = ui.default_colors.rgb_fg.unwrap_or(Rgb::WHITE);

        for grid in ui.grids.iter() {
            self.grids.update(
                &self.device,
                &self.queue,
                grid,
                ui.position(grid.id),
                &ui.highlights,
                fg,
                fonts,
                &mut self.font_cache,
                &mut self.shape_context,
            );
        }

        self.cmdline_grid.update(
            &self.device,
            &self.queue,
            &ui.cmdline,
            Vec2::new(0., ui.grids[0].contents().size.y as f32 - 1.),
            fonts.cell_size().cast_as(),
            &self.text_bind_group_layout.bind_group_layout,
            &ui.highlights,
            fg,
            fonts,
            &mut self.font_cache,
            &mut self.shape_context,
        );

        self.grids.set_draw_order(ui.draw_order.clone());
        self.pipelines.cursor.update(
            &self.device,
            ui,
            cursor::CursorKind::Normal,
            fonts.cell_size().cast_as(),
            &self.targets.monochrome.view,
        );
        self.pipelines.cmdline_cursor.update(
            &self.device,
            ui,
            cursor::CursorKind::Cmdline,
            fonts.cell_size().cast_as(),
            &self.targets.monochrome.view,
        );
        self.pipelines.monochrome.update(
            &self.device,
            &self.queue,
            &self.font_cache.monochrome,
            self.grids.bind_group_layout(),
        );
        self.pipelines.emoji.update(
            &self.device,
            &self.queue,
            &self.font_cache.emoji,
            self.grids.bind_group_layout(),
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

        let target_size: Vec2<u32> = (new_size / cell_size) * cell_size;
        self.targets = Targets::new(&self.device, target_size);

        self.pipelines.gamma_blit_final.update(
            &self.device,
            self.surface_config.format,
            &self.targets.color.view,
            target_size,
            new_size,
        );
        self.pipelines.gamma_blit_png.update(
            &self.device,
            Texture::SRGB_FORMAT,
            &self.targets.color.view,
            target_size,
            new_size,
        );
    }

    pub fn render(
        &mut self,
        cell_size: Vec2<u32>,
        delta_seconds: f32,
        settings: Settings,
        window: &Window,
    ) -> Motion {
        let output = match self.surface.get_current_texture() {
            Ok(output) => output,
            Err(e) => {
                match e {
                    wgpu::SurfaceError::Lost => {
                        log::warn!("Rebuilding swap chain");
                        self.resize(self.surface_size(), cell_size);
                    }
                    _ => log::error!("{e}"),
                }
                return Motion::Still;
            }
        };

        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render encoder"),
            });
        let target_size = self.targets.color.texture.size().into();
        let mut motion = Motion::Still;

        for grid in self.grids.iter_mut() {
            motion |= grid
                .scrolling
                .advance(delta_seconds * settings.scroll_speed * cell_size.y as f32);
        }

        let grid_count = self.grids.grid_count() as f32;
        let grids = || {
            self.grids
                .front_to_back()
                .map(|(z, grid)| {
                    (
                        (z as f32 + 1.) / (grid_count + 1.),
                        grid.offset(cell_size.y as f32),
                        &grid.text,
                    )
                })
                .chain(std::iter::once((
                    0.,
                    self.cmdline_grid.offset(),
                    &self.cmdline_grid.text,
                )))
        };

        self.pipelines.default_fill.render(
            &mut encoder,
            grids().map(|(z, _, grid)| (z, grid)),
            &self.targets.color.view,
            &self.targets.depth.view,
            target_size,
            cell_size,
            self.clear_color,
        );

        self.pipelines.cell_fill.render(
            &mut encoder,
            grids(),
            &self.targets.color.view,
            &self.targets.depth.view,
            target_size,
            cell_size,
        );

        self.pipelines.monochrome.render(
            &mut encoder,
            grids(),
            &self.targets.monochrome.view,
            &self.targets.depth.view,
            target_size,
            cell_size,
        );

        self.pipelines.lines.render(
            &mut encoder,
            grids(),
            &self.targets.monochrome.view,
            &self.targets.depth.view,
            target_size,
            cell_size,
            settings.underline_offset,
        );

        self.pipelines
            .blend
            .render(&mut encoder, &self.targets.color.view);

        motion |= self.pipelines.cursor.render(
            &mut encoder,
            &self.targets.color.view,
            delta_seconds * settings.cursor_speed,
            target_size.cast_as(),
            cell_size.cast_as(),
        );

        motion |= self.pipelines.cmdline_cursor.render(
            &mut encoder,
            &self.targets.color.view,
            delta_seconds * settings.cursor_speed,
            target_size.cast_as(),
            cell_size.cast_as(),
        );

        self.pipelines.emoji.render(
            &mut encoder,
            grids(),
            &self.targets.color.view,
            &self.targets.depth.view,
            cell_size,
            target_size,
        );

        self.pipelines.gamma_blit_final.render(
            &mut encoder,
            &output_view,
            wgpu::Color {
                r: (self.clear_color[0] as f64).powf(2.2),
                g: (self.clear_color[1] as f64).powf(2.2),
                b: (self.clear_color[2] as f64).powf(2.2),
                a: (self.clear_color[3] as f64).powf(2.2),
            },
        );

        self.pipelines.gamma_blit_png.render(
            &mut encoder,
            &self.targets.png.view,
            wgpu::Color {
                r: (self.clear_color[0] as f64).powf(2.2),
                g: (self.clear_color[1] as f64).powf(2.2),
                b: (self.clear_color[2] as f64).powf(2.2),
                a: (self.clear_color[3] as f64).powf(2.2),
            },
        );

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &self.targets.png.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &self.targets.png_staging,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(self.targets.png_size.x * 4),
                    rows_per_image: Some(self.targets.png_size.y),
                },
            },
            self.targets.png_size.into(),
        );

        let submission = self.queue.submit(std::iter::once(encoder.finish()));

        let buffer_slice = self.targets.png_staging.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            result.unwrap();
        });

        self.device
            .poll(wgpu::MaintainBase::WaitForSubmissionIndex(submission));
        let png_number = self.png_count;
        self.png_count += 1;
        let data = buffer_slice.get_mapped_range();
        let file = File::create(format!("render/out_{png_number}.png")).unwrap();
        let ref mut w = BufWriter::new(file);
        let mut w = png::Encoder::new(w, self.targets.png_size.x, self.targets.png_size.y);
        w.set_color(png::ColorType::Rgba);
        w.set_depth(png::BitDepth::Eight);
        w.set_source_gamma(png::ScaledFloat::new(1.0));
        let mut w = w.write_header().unwrap();
        w.write_image_data(cast_slice(&data)).unwrap();
        drop(data);
        self.targets.png_staging.unmap();

        window.pre_present_notify();
        output.present();
        motion
    }

    pub fn clear_glyph_cache(&mut self) {
        self.font_cache.clear();
        self.pipelines.emoji.clear();
        self.pipelines.monochrome.clear();
    }

    pub fn surface_size(&self) -> Vec2<u32> {
        Vec2::new(self.surface_config.width, self.surface_config.height)
    }
}
