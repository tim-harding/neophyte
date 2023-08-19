mod font;
mod grid;
mod highlights;
mod read;
mod shared;
mod texture;
mod write;

use self::{read::ReadState, shared::Shared, write::WriteState};
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
    let (mut state, mut write_state) = {
        let window = window.clone();
        pollster::block_on(async { init(window.clone()).await })
    };
    let mut fonts = Fonts::new();
    let window = window.clone();
    let mut font_cache = FontCache::new();
    while let Ok(event) = rx.recv() {
        match event {
            RenderEvent::Flush(ui) => {
                state.update(ui, &mut write_state, &mut fonts, &mut font_cache);
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
    constant: ConstantState,
    read: Option<ReadState>,
}

pub struct ConstantState {
    pub shared: Shared,
    pub grid: grid::Constant,
    pub font: font::Constant,
    pub highlights: highlights::Constant,
}

impl State {
    pub fn update(
        &mut self,
        ui: Ui,
        write: &mut WriteState,
        fonts: &mut Fonts,
        font_cache: &mut FontCache,
    ) {
        let updates = write.updates(ui, &self.constant, fonts, font_cache);
        match self.read.as_mut() {
            Some(read) => read.apply_updates(updates),
            None => self.read = ReadState::from_updates(updates),
        }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.constant.shared.resize(size);
    }

    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        if let Some(read) = self.read.as_ref() {
            read.render(&self.constant)
        } else {
            Ok(())
        }
    }

    pub fn rebuild_swap_chain(&mut self) {
        let size = self.constant.shared.surface_size();
        let size = PhysicalSize::new(size.x, size.y);
        self.constant.shared.resize(size)
    }
}

pub async fn init(window: Arc<Window>) -> (State, WriteState) {
    let shared = Shared::new(window).await;
    let (highlights_write, highlights_constant) = highlights::init(&shared.device);
    let (grid_write, grid_constant) =
        grid::init(&shared.device, shared.surface_format, &highlights_constant);
    let (font_write, font_constant) = font::new(&shared.device);

    (
        State {
            constant: ConstantState {
                shared,
                grid: grid_constant,
                font: font_constant,
                highlights: highlights_constant,
            },
            read: None,
        },
        WriteState {
            grid: grid_write,
            font: font_write,
            highlights: highlights_write,
        },
    )
}
