mod blit_render_pipeline;
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
    text::fonts::Fonts,
    ui::Ui,
    util::vec2::Vec2,
};
use rmpv::Value;
use std::sync::{mpsc::Receiver, Arc};
use winit::window::Window;

pub enum RenderEvent {
    Notification(String, Vec<Value>),
    Resized(Vec2<u32>),
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
        let fonts = Fonts::new();
        let render_state = {
            let window = window.clone();
            pollster::block_on(async {
                RenderState::new(window.clone(), fonts.metrics().into_pixels().cell_size()).await
            })
        };
        Self {
            window,
            fonts,
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
                    let cell_size = self.fonts.metrics().into_pixels().cell_size();
                    self.render_state.resize(size.into(), cell_size);
                    self.resize_neovim_grid();
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
                    self.fonts.reload(&self.ui.options.guifont);
                    self.render_state.font_cache.clear();
                    self.render_state.monochrome_pipeline.clear();
                    self.resize_neovim_grid();
                }
            }
            event => self.ui.process(event),
        }
    }

    fn resize_neovim_grid(&mut self) {
        let surface_size = self.render_state.shared.surface_size();
        let cell_size = self.fonts.metrics().into_pixels().cell_size();
        let size = surface_size / cell_size;
        let size: Vec2<u64> = size.into();
        self.neovim.ui_try_resize_grid(1, size.x, size.y);
    }
}
