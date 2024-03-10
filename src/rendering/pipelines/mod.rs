pub mod blend;
pub mod cell_fill;
pub mod cursor;
pub mod default_fill;
pub mod gamma_blit;
pub mod lines;
pub mod png_blit;
pub mod text;

use super::{targets::Targets, texture::Texture, wgpu_context::WgpuContext};
use crate::{text::cache::FontCache, ui::Ui, util::vec2::Vec2};

pub struct Pipelines {
    pub cursor: cursor::Pipeline,
    pub cmdline_cursor: cursor::Pipeline,
    pub blend: blend::Pipeline,
    pub default_fill: default_fill::Pipeline,
    pub cell_fill: cell_fill::Pipeline,
    pub monochrome: text::Pipeline,
    pub emoji: text::Pipeline,
    pub gamma_blit_final: gamma_blit::Pipeline,
    pub blit_png: png_blit::Pipeline,
    pub lines: lines::Pipeline,
}

impl Pipelines {
    pub fn new(
        device: &wgpu::Device,
        text_bind_group_layout: &wgpu::BindGroupLayout,
        surface_config: &wgpu::SurfaceConfiguration,
        targets: &Targets,
    ) -> Self {
        Pipelines {
            cursor: cursor::Pipeline::new(&device, &targets.monochrome.view),
            cmdline_cursor: cursor::Pipeline::new(&device, &targets.monochrome.view),
            blend: blend::Pipeline::new(&device, &targets.color.view),
            default_fill: default_fill::Pipeline::new(&device, Texture::LINEAR_FORMAT),
            cell_fill: cell_fill::Pipeline::new(
                &device,
                text_bind_group_layout,
                Texture::LINEAR_FORMAT,
            ),
            monochrome: text::Pipeline::new(
                &device,
                text_bind_group_layout,
                text::Kind::Monochrome,
            ),
            emoji: text::Pipeline::new(&device, text_bind_group_layout, text::Kind::Emoji),
            lines: lines::Pipeline::new(&device, text_bind_group_layout, Texture::LINEAR_FORMAT),
            gamma_blit_final: gamma_blit::Pipeline::new(
                &device,
                surface_config.format,
                &targets.color.view,
            ),
            blit_png: png_blit::Pipeline::new(
                &device,
                &targets.color.view,
                surface_config.width as f32 / targets.png_size.0.x as f32,
            ),
        }
    }

    pub fn update(
        &mut self,
        ui: &Ui,
        wgpu_context: &WgpuContext,
        targets: &Targets,
        font_cache: &FontCache,
        cell_size: Vec2<f32>,
    ) {
        self.cursor.update(
            &wgpu_context.device,
            ui,
            cursor::CursorKind::Normal,
            cell_size,
            &targets.monochrome.view,
        );
        self.cmdline_cursor.update(
            &wgpu_context.device,
            ui,
            cursor::CursorKind::Cmdline,
            cell_size,
            &targets.monochrome.view,
        );
        self.monochrome.update(
            &wgpu_context.device,
            &wgpu_context.queue,
            &font_cache.monochrome,
        );
        self.emoji
            .update(&wgpu_context.device, &wgpu_context.queue, &font_cache.emoji);
        self.blend
            .update(&wgpu_context.device, &targets.monochrome.view);
    }
}
