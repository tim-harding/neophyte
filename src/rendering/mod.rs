mod font;
mod grid;
mod highlights;
mod read;
mod shared;
mod texture;

use self::{
    grid::GridBindGroupLayout, highlights::HighlightsBindGroupLayout, read::render, shared::Shared,
};
use crate::{
    session::Neovim,
    text::{
        cache::FontCache,
        fonts::{FontStyle, Fonts},
    },
    ui::Ui,
};
use std::sync::{mpsc::Receiver, Arc};
use winit::{dpi::PhysicalSize, window::Window};

pub enum RenderEvent {
    Flush(Ui),
    Resized(PhysicalSize<u32>),
    Redraw,
    FontsChanged(Vec<String>, u32),
}

pub fn render_loop(window: Arc<Window>, neovim: Neovim, rx: Receiver<RenderEvent>) {
    let mut state = {
        let window = window.clone();
        pollster::block_on(async { init(window.clone()).await })
    };
    let mut fonts = Fonts::new();
    let window = window.clone();
    let mut font_cache = FontCache::new();
    while let Ok(event) = rx.recv() {
        match event {
            RenderEvent::Flush(ui) => {
                state.update(ui, &mut fonts, &mut font_cache);
                window.request_redraw();
            }
            RenderEvent::Resized(size) => {
                // TODO: Factor this stuff out. Duplicate logic from grid
                // rendering.
                state.resize(size);
                let metrics = fonts
                    .with_style(FontStyle::Regular)
                    .unwrap()
                    .as_ref()
                    .metrics(&[]);
                let scale_factor = fonts.size() as f32 / metrics.average_width;
                let em_px = (metrics.units_per_em as f32 * scale_factor).ceil() as u32;
                let descent_px = (metrics.descent as f32 * scale_factor).ceil() as u32;
                let cell_height_px = em_px + descent_px;
                neovim.ui_try_resize_grid(
                    1,
                    (size.width / fonts.size()) as u64,
                    (size.height / cell_height_px) as u64,
                )
            }

            RenderEvent::Redraw => match state.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => state.rebuild_swap_chain(),
                Err(wgpu::SurfaceError::OutOfMemory) => panic!("Out of memory"),
                Err(e) => eprintln!("{e:?}"),
            },

            RenderEvent::FontsChanged(names, size) => {
                fonts.reload(names, size);
                // TODO: Clear font cache
                // TODO: Clear textures on the GPU
            }
        }
    }
}

pub struct State {
    pub shared: Shared,
    pub grid: grid::Write,
    pub font: font::Write,
    pub highlights_bind_group_layout: highlights::HighlightsBindGroupLayout,
    pub highlights: highlights::HighlightsBindGroup,
    pub grid_bind_group_layout: grid::GridBindGroupLayout,
}

impl State {
    pub fn update(&mut self, ui: Ui, fonts: &mut Fonts, font_cache: &mut FontCache) {
        self.highlights
            .update(&ui, &self.highlights_bind_group_layout, &self.shared);
        self.font.updates(
            &self.shared,
            font_cache,
            &self.highlights_bind_group_layout,
            &self.grid_bind_group_layout,
        );
        self.grid.updates(
            &self.shared,
            &ui,
            fonts,
            font_cache,
            &self.grid_bind_group_layout,
        );
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.shared.resize(size);
    }

    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        render(&self)
    }

    pub fn rebuild_swap_chain(&mut self) {
        let size = self.shared.surface_size();
        let size = PhysicalSize::new(size.x, size.y);
        self.shared.resize(size)
    }
}

async fn init(window: Arc<Window>) -> State {
    let shared = Shared::new(window).await;
    let highlights_bind_group_layout = HighlightsBindGroupLayout::new(&shared.device);
    let grid_bind_group_layout = GridBindGroupLayout::new(&shared.device);
    let grid_write = grid::Write::new(
        &shared.device,
        shared.surface_format,
        &highlights_bind_group_layout,
        &grid_bind_group_layout,
    );
    State {
        font: font::Write::new(&shared.device),
        shared,
        grid_bind_group_layout,
        highlights_bind_group_layout,
        grid: grid_write,
        highlights: highlights::HighlightsBindGroup::default(),
    }
}
