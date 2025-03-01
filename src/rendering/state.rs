use super::{
    Motion, cmdline_grid::CmdlineGrid, grids::Grids, message_grids::MessageGrids,
    pipelines::Pipelines, targets::Targets, text::BindGroupLayout as TextBindGroup,
    wgpu_context::WgpuContext,
};
use crate::{
    event_handler::settings::Settings,
    text::{cache::FontCache, fonts::Fonts},
    ui::Ui,
    util::IntoSrgb,
};
use bytemuck::cast_slice;
use neophyte_linalg::{PixelVec, Vec2};
use neophyte_ui_event::rgb::Rgb;
use std::{
    fs::File,
    io::{self, BufWriter},
    sync::Arc,
    time::Duration,
};
use swash::shape::ShapeContext;
use winit::window::Window;

pub struct RenderState {
    wgpu_context: WgpuContext,
    pipelines: Pipelines,
    targets: Targets,
    grids: Grids,
    shape_context: ShapeContext,
    pub fonts: Fonts,
    font_cache: FontCache,
    clear_color: [f32; 4],
    // TODO: Remove this if we no longer want to externalize the cmdline
    cmdline_grid: CmdlineGrid,
    message_grids: MessageGrids,
    text_bind_group_layout: TextBindGroup,
}

impl RenderState {
    pub fn new(window: Arc<Window>, transparent: bool) -> Self {
        let fonts = Fonts::new();
        let cell_size = fonts.cell_size();
        let wgpu_context = WgpuContext::new(window, transparent);
        let grids = Grids::new(&wgpu_context.device);
        let target_size: PixelVec<u32> =
            (wgpu_context.surface_size().into_cells(cell_size)).into_pixels(cell_size);
        let targets = Targets::new(&wgpu_context.device, target_size);
        Self {
            fonts,
            text_bind_group_layout: TextBindGroup::new(&wgpu_context.device),
            pipelines: Pipelines::new(
                &wgpu_context.device,
                grids.bind_group_layout(),
                &wgpu_context.surface_config,
                &targets,
            ),
            shape_context: ShapeContext::new(),
            font_cache: FontCache::new(),
            grids: Grids::new(&wgpu_context.device),
            targets,
            wgpu_context,
            clear_color: [0.; 4],
            cmdline_grid: CmdlineGrid::new(),
            message_grids: MessageGrids::new(),
        }
    }

    pub fn update(&mut self, ui: &Ui, bg_override: Option<[f32; 4]>) {
        self.clear_color =
            bg_override.unwrap_or(ui.default_colors.rgb_bg.unwrap_or(Rgb::BLACK).into_srgb(1.));

        let fg = ui.default_colors.rgb_fg.unwrap_or(Rgb::WHITE);
        let bg = ui.default_colors.rgb_bg.unwrap_or(Rgb::BLACK);

        self.grids.update(
            &self.wgpu_context.device,
            &self.wgpu_context.queue,
            ui,
            &self.fonts,
            &mut self.font_cache,
            &mut self.shape_context,
        );

        let base_grid_size = ui.grids[0].contents().size.0;
        self.cmdline_grid.update(
            &self.wgpu_context.device,
            &self.wgpu_context.queue,
            &ui.cmdline,
            base_grid_size,
            &self.text_bind_group_layout.bind_group_layout,
            &ui.highlights,
            fg,
            bg,
            &self.fonts,
            &mut self.font_cache,
            &mut self.shape_context,
        );

        self.message_grids.update(
            &self.wgpu_context.device,
            &self.wgpu_context.queue,
            &ui.messages,
            base_grid_size,
            &self.text_bind_group_layout.bind_group_layout,
            &ui.highlights,
            fg,
            bg,
            &self.fonts,
            &mut self.font_cache,
            &mut self.shape_context,
        );

        self.pipelines.update(
            ui,
            &self.wgpu_context,
            &self.targets,
            &self.font_cache,
            self.fonts.cell_size().cast_as(),
        );
    }

    pub fn resize(&mut self, new_size: PixelVec<u32>, cell_size: Vec2<u32>, transparent: bool) {
        if new_size == PixelVec::default() {
            return;
        }

        self.wgpu_context.resize(new_size);

        let target_size: PixelVec<u32> = (new_size.into_cells(cell_size)).into_pixels(cell_size);
        self.targets = Targets::new(&self.wgpu_context.device, target_size);

        self.pipelines.gamma_blit_final.update(
            &self.wgpu_context.device,
            self.wgpu_context.surface_config.format,
            &self.targets.color.view,
            target_size,
            new_size,
            transparent,
        );
        self.pipelines.blit_png.update(
            &self.wgpu_context.device,
            &self.targets.color.view,
            self.targets.png_size.0.x as f32 / target_size.0.x as f32,
        );
    }

    pub fn advance(
        &mut self,
        delta_time: Duration,
        cell_size: Vec2<f32>,
        settings: &Settings,
    ) -> Motion {
        let mut motion = Motion::Still;

        for grid in self.grids.iter_mut() {
            motion = motion.soonest(
                grid.scrolling
                    .advance(delta_time, settings.scroll_speed * cell_size.y),
            );
        }

        const DEFAULT_CURSOR_SPEED: f32 = 100.;
        let cursor_speed = settings.cursor_speed * DEFAULT_CURSOR_SPEED;
        motion = motion.soonest(
            self.pipelines
                .cursor
                .advance(delta_time, cursor_speed, cell_size),
        );
        motion = motion.soonest(self.pipelines.cmdline_cursor.advance(
            delta_time,
            cursor_speed,
            cell_size,
        ));

        motion
    }

    fn current_texture(&mut self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        self.wgpu_context.surface.get_current_texture()
    }

    pub fn render(
        &mut self,
        cell_size: Vec2<u32>,
        settings: &Settings,
        window: &Window,
        frame_number: u32,
    ) {
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
        let mut encoder =
            self.wgpu_context
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
                .chain(
                    self.message_grids
                        .texts()
                        .map(|text| (f32::EPSILON, PixelVec::new(0, 0), text)),
                )
                .chain(std::iter::once((
                    0.,
                    PixelVec::new(0, 0),
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
            wgpu::TexelCopyTextureInfo {
                texture: &self.targets.png.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &self.targets.png_staging,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(self.targets.png_size.0.x * 4),
                    rows_per_image: Some(self.targets.png_size.0.y),
                },
            },
            self.targets.png_size.into(),
        );

        let submission = self
            .wgpu_context
            .queue
            .submit(std::iter::once(encoder.finish()));

        // TODO: Offload to a thread
        if let Some(dir) = settings.render_target.as_ref() {
            let cb = || -> Result<(), SavePngError> {
                let buffer_slice = self.targets.png_staging.slice(..);
                buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
                    result.unwrap();
                });

                self.wgpu_context
                    .device
                    .poll(wgpu::MaintainBase::WaitForSubmissionIndex(submission));
                let data = buffer_slice.get_mapped_range();
                let file_name = format!("{frame_number:0>6}.png");
                let file = File::create(dir.join(file_name))?;
                let w = &mut BufWriter::new(file);
                let mut w =
                    png::Encoder::new(w, self.targets.png_size.0.x, self.targets.png_size.0.y);
                w.set_color(png::ColorType::Rgba);
                w.set_depth(png::BitDepth::Eight);
                w.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
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
        PixelVec::new(
            self.wgpu_context.surface_config.width,
            self.wgpu_context.surface_config.height,
        )
    }
}

#[derive(Debug, thiserror::Error)]
enum SavePngError {
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Png(#[from] png::EncodingError),
}
