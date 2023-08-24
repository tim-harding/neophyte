mod cell_fill_pipeline;
mod cursor;
mod depth_texture;
mod glyph_pipeline;
mod grid;
mod grid_bind_group_layout;
mod highlights;
mod shared;
mod state;
mod texture;

use self::state::RenderState;
use crate::{
    event::{self, Event, OptionSet, SetTitle},
    session::Neovim,
    text::fonts::{FontStyle, Fonts},
    ui::Ui,
};
use rmpv::Value;
use std::sync::{mpsc::Receiver, Arc};
use winit::{dpi::PhysicalSize, window::Window};

pub enum RenderEvent {
    Notification(String, Vec<Value>),
    Resized(PhysicalSize<u32>),
    Redraw,
}

pub struct RenderLoop {
    render_state: RenderState,
    fonts: Fonts,
    window: Arc<Window>,
    ui: Ui,
    neovim: Neovim,
}

impl RenderLoop {
    pub fn new(window: Arc<Window>, neovim: Neovim) -> Self {
        let render_state = {
            let window = window.clone();
            pollster::block_on(async { RenderState::new(window.clone()).await })
        };
        Self {
            window,
            fonts: Fonts::new(),
            ui: Ui::new(),
            render_state,
            neovim,
        }
    }

    pub fn run(mut self, rx: Receiver<RenderEvent>) {
        while let Ok(event) = rx.recv() {
            match event {
                RenderEvent::Notification(method, params) => {
                    self.handle_notification(method, params)
                }

                RenderEvent::Resized(size) => {
                    self.render_state.resize(size);
                    self.resize_grid();
                }

                RenderEvent::Redraw => match self.render_state.render(&self.ui.draw_order) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => self.render_state.rebuild_swap_chain(),
                    Err(wgpu::SurfaceError::OutOfMemory) => panic!("Out of memory"),
                    Err(e) => eprintln!("{e:?}"),
                },
            }
        }
    }

    fn handle_notification(&mut self, method: String, params: Vec<Value>) {
        match method.as_str() {
            "redraw" => {
                for param in params {
                    match Event::try_parse(param.clone()) {
                        Ok(events) => {
                            for event in events {
                                self.handle_event(event);
                            }
                        }
                        Err(e) => match e {
                            event::Error::UnknownEvent(name) => {
                                log::error!("Unknown event: {name}\n{param:#?}");
                            }
                            _ => log::error!("{e}"),
                        },
                    }
                }
            }
            _ => log::error!("Unrecognized notification: {method}"),
        }
    }

    fn handle_event(&mut self, event: Event) {
        log::info!("{event:?}");
        match event {
            Event::Flush => {
                self.render_state.update(&self.ui, &mut self.fonts);
                self.ui.process(Event::Flush);
                self.window.request_redraw();
            }
            Event::SetTitle(SetTitle { title }) => self.window.set_title(&title),
            Event::OptionSet(event) => {
                let is_gui_font = matches!(event, OptionSet::Guifont(_));
                self.ui.process(Event::OptionSet(event));
                if is_gui_font {
                    let guifont = &self.ui.options.guifont;
                    self.fonts.reload(guifont.0.as_slice(), guifont.1);
                    self.render_state.font_cache.clear();
                    self.render_state.glyph_pipeline.clear();
                    self.resize_grid();
                }
            }
            event => self.ui.process(event),
        }
    }

    fn resize_grid(&mut self) {
        let size = self.render_state.shared.surface_size();
        let metrics = self
            .fonts
            .with_style(FontStyle::Regular)
            .metrics(self.fonts.size());
        self.neovim.ui_try_resize_grid(
            1,
            (size.x / self.fonts.size()) as u64,
            (size.y / metrics.cell_size_px.y) as u64,
        )
    }
}
