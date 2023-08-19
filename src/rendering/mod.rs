mod font;
mod grid;
mod highlights;
mod read;
mod shared;
mod texture;

use self::{
    highlights::HighlightsBindGroupLayout,
    read::{ReadState, ReadStateUpdates},
    shared::Shared,
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
        }
    }
}

pub struct State {
    pub shared: Shared,
    pub grid_constant: grid::Constant,
    pub grid: grid::Write,
    pub font: font::Write,
    pub highlights_bind_group_layout: highlights::HighlightsBindGroupLayout,
    pub highlights: highlights::HighlightsBindGroup,
    read: Option<ReadState>,
}

impl State {
    pub fn update(&mut self, ui: Ui, fonts: &mut Fonts, font_cache: &mut FontCache) {
        self.highlights
            .update(&ui, &self.highlights_bind_group_layout, &self.shared);
        let updates = ReadStateUpdates {
            grid: self
                .grid
                .updates(&self.grid_constant, &self.shared, &ui, fonts, font_cache),
            font: self.font.updates(
                &self.shared,
                font_cache,
                &self.grid_constant,
                &self.highlights_bind_group_layout,
            ),
        };
        match self.read.as_mut() {
            Some(read) => read.apply_updates(updates),
            None => self.read = ReadState::from_updates(updates),
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.shared.resize(size);
    }

    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        if let Some(read) = self.read.as_ref() {
            read.render(&self)
        } else {
            Ok(())
        }
    }

    pub fn rebuild_swap_chain(&mut self) {
        let size = self.shared.surface_size();
        let size = PhysicalSize::new(size.x, size.y);
        self.shared.resize(size)
    }
}

pub async fn init(window: Arc<Window>) -> State {
    let shared = Shared::new(window).await;
    let highlights_bind_group_layout = HighlightsBindGroupLayout::new(&shared.device);
    let (grid_write, grid_constant) = grid::init(
        &shared.device,
        shared.surface_format,
        &highlights_bind_group_layout,
    );
    State {
        font: font::Write::new(&shared.device),
        shared,
        grid_constant,
        highlights_bind_group_layout,
        read: None,
        grid: grid_write,
        highlights: highlights::HighlightsBindGroup::default(),
    }
}
