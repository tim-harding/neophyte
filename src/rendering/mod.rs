mod depth_texture;
mod glyph_bind_group;
mod glyph_push_constants;
mod grid;
mod grids;
mod highlights;
mod pipelines;
mod state;
mod texture;

use self::state::RenderState;
use crate::{
    event::{self, Event, OptionSet, SetTitle},
    neovim::{Action, Button, Modifiers, Neovim},
    text::fonts::Fonts,
    ui::Ui,
    util::vec2::Vec2,
};
use bitfield_struct::bitfield;
use rmpv::Value;
use std::{
    ops::{BitOr, BitOrAssign},
    sync::{
        mpsc::{Receiver, TryRecvError},
        Arc,
    },
};
use winit::window::Window;

pub const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

pub enum RenderEvent {
    Notification(String, Vec<Value>),
    Resized(Vec2<u32>),
    Redraw,
    Scroll {
        delta: Vec2<f64>,
        kind: ScrollKind,
        reset: bool,
        modifiers: Modifiers,
    },
    MouseMove {
        position: Vec2<f64>,
        modifiers: Modifiers,
    },
    Click {
        button: Button,
        action: Action,
        modifiers: Modifiers,
    },
}

pub enum ScrollKind {
    Lines,
    Pixels,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Mouse {
    position: Vec2<u64>,
    scroll: Vec2<i64>,
    buttons: Buttons,
}

#[bitfield(u8)]
#[derive(PartialEq, Eq)]
struct Buttons {
    left: bool,
    right: bool,
    middle: bool,
    #[bits(5)]
    __: u8,
}

impl Buttons {
    pub fn first(&self) -> Option<Button> {
        if self.left() {
            Some(Button::Left)
        } else if self.right() {
            Some(Button::Right)
        } else if self.middle() {
            Some(Button::Middle)
        } else {
            None
        }
    }
}

pub struct RenderLoop {
    render_state: RenderState,
    fonts: Fonts,
    window: Arc<Window>,
    ui: Ui,
    neovim: Neovim,
    mouse: Mouse,
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
            mouse: Mouse::default(),
        }
    }

    pub fn run(mut self, rx: Receiver<RenderEvent>) {
        loop {
            let framerate = self
                .window
                .current_monitor()
                .map(|monitor| monitor.refresh_rate_millihertz())
                .flatten()
                .unwrap_or(60000);
            self.render_state.maybe_render(self.cell_size(), framerate);

            loop {
                let event = match rx.try_recv() {
                    Ok(event) => event,
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => return,
                };

                match event {
                    RenderEvent::Notification(method, params) => {
                        self.handle_notification(method, params)
                    }

                    RenderEvent::Resized(size) => {
                        self.render_state.resize(size, self.cell_size());
                        self.resize_neovim_grid();
                    }

                    RenderEvent::Redraw => self.render_state.request_redraw(),

                    RenderEvent::Scroll {
                        delta,
                        kind,
                        reset,
                        modifiers,
                    } => {
                        let delta: Vec2<i64> = delta.cast_as();
                        if reset {
                            self.mouse.scroll = Vec2::default();
                        }

                        let lines = match kind {
                            ScrollKind::Lines => delta,
                            ScrollKind::Pixels => {
                                self.mouse.scroll += delta;
                                let cell_size: Vec2<i64> = self.cell_size().cast();
                                let lines = self.mouse.scroll / cell_size;
                                self.mouse.scroll -= lines * cell_size;
                                lines
                            }
                        };

                        let Some(grid) = self.ui.grid_under_cursor(
                            self.mouse.position.into(),
                            self.fonts.metrics().into_pixels().cell_size().cast(),
                        ) else {
                            continue;
                        };

                        let action = if lines.y < 0 {
                            Action::WheelDown
                        } else {
                            Action::WheelUp
                        };

                        for _ in 0..lines.y.abs() {
                            self.neovim.input_mouse(
                                Button::Wheel,
                                action,
                                modifiers,
                                grid.grid,
                                grid.position.y,
                                grid.position.x,
                            );
                        }

                        let action = if lines.x < 0 {
                            Action::WheelRight
                        } else {
                            Action::WheelLeft
                        };

                        for _ in 0..lines.x.abs() {
                            self.neovim.input_mouse(
                                Button::Wheel,
                                action,
                                modifiers,
                                grid.grid,
                                grid.position.y,
                                grid.position.x,
                            );
                        }
                    }

                    RenderEvent::MouseMove {
                        position,
                        modifiers,
                    } => {
                        let position: Vec2<i64> = position.cast_as();
                        let surface_size = self.render_state.surface_size();
                        let cell_size = self.cell_size();
                        let inner = (surface_size / cell_size) * cell_size;
                        let margin = (surface_size - inner) / 2;
                        let position = position - margin.cast();
                        let Ok(position) = position.try_cast::<u64>() else {
                            continue;
                        };
                        self.mouse.position = position;
                        if let Some(grid) = self.ui.grid_under_cursor(
                            position,
                            self.fonts.metrics().into_pixels().cell_size().cast(),
                        ) {
                            self.neovim.input_mouse(
                                self.mouse.buttons.first().unwrap_or(Button::Move),
                                // Irrelevant for move
                                Action::ButtonDrag,
                                modifiers,
                                grid.grid,
                                grid.position.y,
                                grid.position.x,
                            );
                        }
                    }

                    RenderEvent::Click {
                        button,
                        action,
                        modifiers,
                    } => {
                        let depressed = match action {
                            Action::ButtonPress => true,
                            Action::ButtonRelease => false,
                            _ => unreachable!(),
                        };
                        match button {
                            Button::Left => self.mouse.buttons.set_left(depressed),
                            Button::Right => self.mouse.buttons.set_right(depressed),
                            Button::Middle => self.mouse.buttons.set_middle(depressed),
                            _ => unreachable!(),
                        }
                        if let Some(grid) = self.ui.grid_under_cursor(
                            self.mouse.position,
                            self.fonts.metrics().into_pixels().cell_size().cast(),
                        ) {
                            self.neovim.input_mouse(
                                button,
                                action,
                                modifiers,
                                grid.grid,
                                grid.position.y,
                                grid.position.x,
                            );
                        }
                    }
                }
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
                self.render_state.request_redraw();
            }
            Event::SetTitle(SetTitle { title }) => self.window.set_title(&title),
            Event::OptionSet(event) => {
                let is_gui_font = matches!(event, OptionSet::Guifont(_));
                self.ui.process(Event::OptionSet(event));
                if is_gui_font {
                    self.fonts.reload(&self.ui.options.guifont);
                    self.render_state.clear_glyph_cache();
                    self.resize_neovim_grid();
                }
            }
            event => self.ui.process(event),
        }
    }

    fn resize_neovim_grid(&mut self) {
        let surface_size = self.render_state.surface_size();
        let size = surface_size / self.cell_size();
        let size: Vec2<u64> = size.cast();
        self.neovim.ui_try_resize_grid(1, size.x, size.y);
    }

    fn cell_size(&self) -> Vec2<u32> {
        self.fonts.metrics().into_pixels().cell_size()
    }
}

pub fn nearest_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Glyph sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Motion {
    #[default]
    Still,
    Animating,
}

impl BitOr for Motion {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Still, Self::Still) => Self::Still,
            _ => Self::Animating,
        }
    }
}

impl BitOrAssign for Motion {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}
