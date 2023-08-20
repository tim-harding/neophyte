mod cell_fill_pipeline;
mod emoji_pipeline;
mod glyph_pipeline;
mod grid;
mod grid_bind_group_layout;
mod highlights;
mod shared;
mod state;
mod texture;

use self::state::State;
use crate::{
    session::Neovim,
    text::fonts::{FontStyle, Fonts},
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
        pollster::block_on(async { State::new(window.clone()).await })
    };
    let mut fonts = Fonts::new();
    let window = window.clone();
    while let Ok(event) = rx.recv() {
        match event {
            RenderEvent::Flush(ui) => {
                state.update(ui, &mut fonts);
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
