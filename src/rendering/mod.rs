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
    event::{Event, OptionSet, SetTitle},
    neovim::{Action, Button, Modifiers, Neovim},
    rpc,
    text::fonts::Fonts,
    ui::{FontSize, FontsSetting, Ui},
    util::vec2::Vec2,
};
use bitfield_struct::bitfield;
use std::{
    ops::{BitOr, BitOrAssign},
    sync::{
        mpsc::{Receiver, TryRecvError},
        Arc, RwLock,
    },
};
use winit::window::Window;

pub const TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

pub enum RenderEvent {
    Notification(Notification),
    Request(Request),
    Resized(Vec2<u32>),
    RequestRedraw,
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

pub enum Notification {
    Redraw(Vec<Event>),
    SetFontSize(FontSize),
    SetFonts(Vec<String>),
    SetScrollSpeed(f32),
    SetCursorSpeed(f32),
}

pub struct Request {
    pub msgid: u64,
    pub kind: RequestKind,
}

pub enum RequestKind {
    Fonts,
    ScrollSpeed,
    CursorSpeed,
    FontWidth,
    FontHeight,
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
    fonts: Arc<RwLock<Fonts>>,
    window: Arc<Window>,
    ui: Ui,
    neovim: Neovim,
    mouse: Mouse,
}

impl RenderLoop {
    pub fn new(window: Arc<Window>, neovim: Neovim, fonts: Arc<RwLock<Fonts>>) -> Self {
        let render_state = {
            let window = window.clone();
            pollster::block_on(async {
                RenderState::new(
                    window.clone(),
                    fonts.read().unwrap().metrics().into_pixels().cell_size(),
                )
                .await
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
                .and_then(|monitor| monitor.refresh_rate_millihertz())
                .unwrap_or(60000);
            self.render_state.maybe_render(self.cell_size(), framerate);

            loop {
                let event = match rx.try_recv() {
                    Ok(event) => event,
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => return,
                };

                match event {
                    RenderEvent::Notification(notification) => match notification {
                        Notification::Redraw(events) => {
                            for event in events {
                                self.handle_event(event);
                            }
                        }

                        Notification::SetFontSize(size) => {
                            self.fonts.write().unwrap().set_font_size(size);
                            self.render_state.clear_glyph_cache();
                            self.resize_neovim_grid();
                        }

                        Notification::SetFonts(fonts) => {
                            self.fonts.write().unwrap().set_fonts(&FontsSetting {
                                fonts,
                                size: FontSize::Height(self.fonts.read().unwrap().metrics().em),
                            });
                            self.render_state.clear_glyph_cache();
                            self.resize_neovim_grid();
                        }

                        Notification::SetScrollSpeed(speed) => {
                            self.render_state.settings_mut().scroll_speed = speed;
                        }

                        Notification::SetCursorSpeed(speed) => {
                            self.render_state.settings_mut().cursor_speed = speed;
                        }
                    },

                    RenderEvent::Resized(size) => {
                        self.render_state.resize(size, self.cell_size());
                        self.resize_neovim_grid();
                    }

                    RenderEvent::RequestRedraw => self.render_state.request_redraw(),

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
                            self.mouse.position,
                            self.fonts
                                .read()
                                .unwrap()
                                .metrics()
                                .into_pixels()
                                .cell_size()
                                .cast(),
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
                            self.fonts
                                .read()
                                .unwrap()
                                .metrics()
                                .into_pixels()
                                .cell_size()
                                .cast(),
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
                            self.fonts
                                .read()
                                .unwrap()
                                .metrics()
                                .into_pixels()
                                .cell_size()
                                .cast(),
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

                    RenderEvent::Request(request) => match request.kind {
                        RequestKind::Fonts => {
                            let names = self
                                .fonts
                                .read()
                                .unwrap()
                                .iter()
                                .map(|font| font.name.clone())
                                .collect();
                            self.neovim
                                .send_response(rpc::Response::result(request.msgid, names));
                        }
                        RequestKind::ScrollSpeed => {
                            let scroll_speed = self.render_state.settings().scroll_speed;
                            self.neovim.send_response(rpc::Response::result(
                                request.msgid,
                                scroll_speed.into(),
                            ));
                        }
                        RequestKind::CursorSpeed => {
                            let cursor_speed = self.render_state.settings().cursor_speed;
                            self.neovim.send_response(rpc::Response::result(
                                request.msgid,
                                cursor_speed.into(),
                            ));
                        }
                        RequestKind::FontWidth => {
                            let width = self.fonts.read().unwrap().metrics().width;
                            self.neovim
                                .send_response(rpc::Response::result(request.msgid, width.into()));
                        }
                        RequestKind::FontHeight => {
                            let width = self.fonts.read().unwrap().metrics().em;
                            self.neovim
                                .send_response(rpc::Response::result(request.msgid, width.into()));
                        }
                    },
                }
            }
        }
    }

    fn handle_event(&mut self, event: Event) {
        log::info!("{event:?}");
        match event {
            Event::Flush => {
                self.render_state
                    .update(&self.ui, &mut self.fonts.write().unwrap());
                self.ui.process(Event::Flush);
                self.render_state.request_redraw();
            }
            Event::SetTitle(SetTitle { title }) => self.window.set_title(&title),
            Event::OptionSet(event) => {
                let is_gui_font = matches!(event, OptionSet::Guifont(_));
                self.ui.process(Event::OptionSet(event));
                if is_gui_font {
                    self.fonts
                        .write()
                        .unwrap()
                        .set_fonts(&self.ui.options.guifont);
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
        self.fonts
            .read()
            .unwrap()
            .metrics()
            .into_pixels()
            .cell_size()
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
