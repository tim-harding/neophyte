use std::{
    fs::File,
    io::{self, BufWriter},
};

use super::{
    cmdline_grid::CmdlineGrid,
    grids::Grids,
    pipelines::{blend, cell_fill, cursor, default_fill, gamma_blit, glyph, lines, png_blit},
    text,
    texture::Texture,
    Motion,
};
use crate::{
    event::rgb::Rgb,
    event_handler::settings::Settings,
    text::{cache::FontCache, fonts::Fonts},
    ui::Ui,
    util::vec2::{CellVec, PixelVec, Vec2},
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
    // TODO: Remove this if we no longer want to externalize the cmdline
    cmdline_grid: CmdlineGrid,
    text_bind_group_layout: text::bind_group::BindGroup,
    pub updated_since_last_render: bool,
}

impl RenderState {
    pub async fn new(window: &Window, cell_size: Vec2<u32>, transparent: bool) -> Self {
        let surface_size: PixelVec<u32> = window.inner_size().into();

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

        let alpha_mode = if transparent
            && surface_caps
                .alpha_modes
                .contains(&wgpu::CompositeAlphaMode::PreMultiplied)
        {
            wgpu::CompositeAlphaMode::PreMultiplied
        } else {
            surface_caps.alpha_modes[0]
        };

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_caps
                .formats
                .iter()
                .copied()
                .find(|f| f.is_srgb())
                .unwrap_or(surface_caps.formats[0]),
            width: surface_size.0.x,
            height: surface_size.0.y,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode,
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);

        let grids = Grids::new(&device);

        let target_size: PixelVec<u32> =
            (surface_size.into_cells(cell_size)).into_pixels(cell_size);
        let targets = Targets::new(&device, target_size);

        Self {
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
                monochrome: glyph::Pipeline::new(
                    &device,
                    grids.bind_group_layout(),
                    glyph::Kind::Monochrome,
                ),
                emoji: glyph::Pipeline::new(&device, grids.bind_group_layout(), glyph::Kind::Emoji),
                lines: lines::Pipeline::new(
                    &device,
                    grids.bind_group_layout(),
                    Texture::LINEAR_FORMAT,
                ),
                gamma_blit_final: gamma_blit::Pipeline::new(
                    &device,
                    surface_config.format,
                    &targets.color.view,
                ),
                blit_png: png_blit::Pipeline::new(
                    &device,
                    &targets.color.view,
                    target_size.0.x as f32 / targets.png_size.0.x as f32,
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
            updated_since_last_render: false,
        }
    }

    pub fn update(&mut self, ui: &Ui, fonts: &Fonts, bg_override: Option<[f32; 4]>) {
        self.updated_since_last_render = true;
        self.clear_color =
            bg_override.unwrap_or(ui.default_colors.rgb_bg.unwrap_or(Rgb::BLACK).into_srgb(1.));

        self.grids.retain(|id| ui.grid(id).is_some());

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
            Some(CellVec::new(
                0.,
                ui.grids[0].contents().size.0.y as f32 - 1.,
            )),
            &self.text_bind_group_layout.bind_group_layout,
            &ui.highlights,
            fg,
            fonts,
            &mut self.font_cache,
            &mut self.shape_context,
        );

        self.grids.set_draw_order(
            ui.draw_order
                .iter()
                .map(|draw_item| draw_item.grid)
                .collect(),
        );
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
        self.pipelines
            .monochrome
            .update(&self.device, &self.queue, &self.font_cache.monochrome);
        self.pipelines
            .emoji
            .update(&self.device, &self.queue, &self.font_cache.emoji);
        self.pipelines
            .blend
            .update(&self.device, &self.targets.monochrome.view);
    }

    pub fn resize(&mut self, new_size: PixelVec<u32>, cell_size: Vec2<u32>, transparent: bool) {
        if new_size == PixelVec::default() {
            return;
        }

        self.surface_config.width = new_size.0.x;
        self.surface_config.height = new_size.0.y;
        self.surface.configure(&self.device, &self.surface_config);

        let target_size: PixelVec<u32> = (new_size.into_cells(cell_size)).into_pixels(cell_size);
        self.targets = Targets::new(&self.device, target_size);

        self.pipelines.gamma_blit_final.update(
            &self.device,
            self.surface_config.format,
            &self.targets.color.view,
            target_size,
            new_size,
            transparent,
        );
        self.pipelines.blit_png.update(
            &self.device,
            &self.targets.color.view,
            self.targets.png_size.0.x as f32 / target_size.0.x as f32,
        );
    }

    pub fn advance(
        &mut self,
        delta_seconds: f32,
        cell_size: Vec2<f32>,
        settings: &Settings,
    ) -> Motion {
        let mut motion = Motion::Still;

        for grid in self.grids.iter_mut() {
            motion = motion.soonest(
                grid.scrolling
                    .advance(delta_seconds * settings.scroll_speed * cell_size.y),
            );
        }

        motion = motion.soonest(
            self.pipelines
                .cursor
                .advance(delta_seconds * settings.cursor_speed, cell_size),
        );
        motion = motion.soonest(
            self.pipelines
                .cmdline_cursor
                .advance(delta_seconds * settings.cursor_speed, cell_size),
        );

        motion
    }

    #[time_execution]
    fn current_texture(&mut self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        self.surface.get_current_texture()
    }

    #[time_execution]
    pub fn render(
        &mut self,
        cell_size: Vec2<u32>,
        settings: &Settings,
        window: &Window,
        frame_number: u32,
    ) {
        self.updated_since_last_render = false;
        let output = match self.current_texture() {
            Ok(output) => output,
            Err(e) => {
                match e {
                    wgpu::SurfaceError::Lost => {
                        log::warn!("Rebuilding swap chain");
                        self.resize(self.surface_size(), cell_size, settings.transparent);
                    }
                    _ => log::error!("{e}"),
                }
                return;
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

        let grid_count = self.grids.grid_count() as f32;
        let grids = || {
            self.grids
                .front_to_back()
                .map(|(z, grid)| {
                    (
                        (z as f32 + 1.) / (grid_count + 1.),
                        grid.scrolling.offset().round_to_pixels(cell_size),
                        &grid.text,
                    )
                })
                .chain(std::iter::once((
                    0.,
                    PixelVec::new(0, 0),
                    &self.cmdline_grid.text,
                )))
        };

        // See the module documentation for each pipeline for more details

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

        self.pipelines.cursor.render(
            &mut encoder,
            &self.targets.color.view,
            target_size.cast_as(),
        );

        self.pipelines.cmdline_cursor.render(
            &mut encoder,
            &self.targets.color.view,
            target_size.cast_as(),
        );

        self.pipelines.emoji.render(
            &mut encoder,
            grids(),
            &self.targets.color.view,
            &self.targets.depth.view,
            target_size,
            cell_size,
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

        if settings.render_target.is_some() {
            self.pipelines.blit_png.render(
                &mut encoder,
                &self.targets.png.view,
                wgpu::Color {
                    r: (self.clear_color[0] as f64).powf(2.2),
                    g: (self.clear_color[1] as f64).powf(2.2),
                    b: (self.clear_color[2] as f64).powf(2.2),
                    a: (self.clear_color[3] as f64).powf(2.2),
                },
            );
        }

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
                    bytes_per_row: Some(self.targets.png_size.0.x * 4),
                    rows_per_image: Some(self.targets.png_size.0.y),
                },
            },
            self.targets.png_size.into(),
        );

        let submission = self.queue.submit(std::iter::once(encoder.finish()));

        // TODO: Offload to a thread
        if let Some(dir) = settings.render_target.as_ref() {
            let cb = || -> Result<(), SavePngError> {
                let buffer_slice = self.targets.png_staging.slice(..);
                buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
                    result.unwrap();
                });

                self.device
                    .poll(wgpu::MaintainBase::WaitForSubmissionIndex(submission));
                let data = buffer_slice.get_mapped_range();
                let file_name = format!("{frame_number:0>6}.png");
                let file = File::create(dir.join(file_name))?;
                let w = &mut BufWriter::new(file);
                let mut w =
                    png::Encoder::new(w, self.targets.png_size.0.x, self.targets.png_size.0.y);
                w.set_color(png::ColorType::Rgba);
                w.set_depth(png::BitDepth::Eight);
                w.set_srgb(png::SrgbRenderingIntent::Perceptual);
                let mut w = w.write_header()?;
                w.write_image_data(cast_slice(&data))?;
                drop(data);
                Ok(())
            };
            match cb() {
                Ok(_) => {}
                Err(e) => log::error!("{e}"),
            }
            self.targets.png_staging.unmap();
        }

        window.pre_present_notify();
        output.present();
    }

    pub fn clear_glyph_cache(&mut self) {
        self.font_cache.clear();
        self.pipelines.emoji.clear();
        self.pipelines.monochrome.clear();
    }

    pub fn surface_size(&self) -> PixelVec<u32> {
        PixelVec::new(self.surface_config.width, self.surface_config.height)
    }
}

struct Targets {
    monochrome: Texture,
    color: Texture,
    depth: Texture,
    png: Texture,
    png_staging: wgpu::Buffer,
    png_size: PixelVec<u32>,
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

struct Pipelines {
    cursor: cursor::Pipeline,
    cmdline_cursor: cursor::Pipeline,
    blend: blend::Pipeline,
    default_fill: default_fill::Pipeline,
    cell_fill: cell_fill::Pipeline,
    monochrome: glyph::Pipeline,
    emoji: glyph::Pipeline,
    gamma_blit_final: gamma_blit::Pipeline,
    blit_png: png_blit::Pipeline,
    lines: lines::Pipeline,
}

#[derive(Debug, thiserror::Error)]
enum SavePngError {
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Png(#[from] png::EncodingError),
}
