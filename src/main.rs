mod event;
mod neovim;
mod rendering;
mod rpc;
pub mod text;
mod ui;
mod util;

use crate::{
    neovim::{Action, Button},
    rendering::pipelines::cursor::CursorUpdateInfo,
};
use bitfield_struct::bitfield;
use event::{Event, OptionSet};
use neovim::{Neovim, StdoutHandler};
use rendering::{
    highlights::HighlightUpdateInfo,
    state::{Message, RenderState},
};
use rpc::Notification;
use std::{
    sync::{mpsc::Sender, Arc, RwLock},
    thread,
};
use text::fonts::{Fonts, FontsHandle};
use ui::{FontSize, FontsSetting, Ui};
use util::{vec2::Vec2, Values};
use winit::{
    event::{
        ElementState, KeyboardInput, ModifiersState, MouseScrollDelta, TouchPhase, VirtualKeyCode,
        WindowEvent,
    },
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy},
    window::{Window, WindowBuilder},
};

fn main() {
    env_logger::builder().format_timestamp(None).init();
    let (neovim, stdout_handler, stdin_handler) = Neovim::new().unwrap();
    let settings = Arc::new(RwLock::new(Settings::new()));
    let fonts = Arc::new(FontsHandle::new());
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());
    let ui = Arc::new(RwLock::new(Ui::new()));
    let render_state = pollster::block_on(async {
        let cell_size = fonts.read().metrics().into_pixels().cell_size();
        RenderState::new(window.clone(), cell_size).await
    });
    neovim.ui_attach();

    let mut surface_size = render_state.surface_size();
    let (render_thread, render_tx) = render_state.run(fonts.clone());

    let mut stdin_thread = Some(std::thread::spawn(move || stdin_handler.start()));
    let mut stdout_thread = Some({
        let handler = NeovimHandler {
            proxy: event_loop.create_proxy(),
            window: window.clone(),
            fonts: fonts.clone(),
            neovim: neovim.clone(),
            settings: settings.clone(),
            ui: ui.clone(),
            render_tx: render_tx.clone(),
        };
        thread::spawn(move || {
            stdout_handler.start(handler);
        })
    });

    let mut render_thread = Some(render_thread);
    let mut render_tx = Some(render_tx);
    let mut mouse = Mouse::new();
    let mut neovim = Some(neovim);
    let mut modifiers = ModifiersState::default();
    event_loop.run(move |event, _, control_flow| {
        use winit::event::Event;
        *control_flow = ControlFlow::Wait;
        match event {
            Event::UserEvent(user_event) => {
                match user_event {
                    UserEvent::Exit => {
                        // Consume the render thread channel to kill the thread
                        let _ = render_tx.take();
                        // Consume the last Neovim instance to close the channel
                        let _ = neovim.take();

                        // Already terminated since it generated the exit event
                        stdout_thread.take().unwrap().join().unwrap();
                        stdin_thread.take().unwrap().join().unwrap();
                        render_thread.take().unwrap().join().unwrap();

                        *control_flow = ControlFlow::Exit;
                    }

                    UserEvent::ResizeGrid => {
                        resize_neovim_grid(surface_size, &fonts.read(), neovim.as_ref().unwrap());
                    }
                }
            }

            Event::WindowEvent {
                window_id,
                ref event,
            } if window_id == window.id() => match event {
                WindowEvent::ModifiersChanged(new_modifiers) => {
                    modifiers = *new_modifiers;
                }

                WindowEvent::ReceivedCharacter(c) => {
                    let mut f = || {
                        let s = match c {
                            '<' => "lt".to_string(),
                            '\\' => "Bslash".to_string(),
                            '|' => "Bar".to_string(),
                            _ => {
                                if !c.is_control()
                                    && !c.is_whitespace()
                                    && !c.is_ascii_digit()
                                    && !c.is_ascii_alphabetic()
                                {
                                    format!("{c}")
                                } else {
                                    return;
                                }
                            }
                        };
                        send_keys(&s, &mut modifiers, neovim.as_mut().unwrap(), true);
                    };
                    f()
                }

                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(virtual_keycode),
                            ..
                        },
                    ..
                } => {
                    let c = || {
                        Some(match virtual_keycode {
                            VirtualKeyCode::Escape => "Esc",
                            VirtualKeyCode::F1 => "F1",
                            VirtualKeyCode::F2 => "F2",
                            VirtualKeyCode::F3 => "F3",
                            VirtualKeyCode::F4 => "F4",
                            VirtualKeyCode::F5 => "F5",
                            VirtualKeyCode::F6 => "F6",
                            VirtualKeyCode::F7 => "F7",
                            VirtualKeyCode::F8 => "F8",
                            VirtualKeyCode::F9 => "F9",
                            VirtualKeyCode::F10 => "F10",
                            VirtualKeyCode::F11 => "F11",
                            VirtualKeyCode::F12 => "F12",
                            VirtualKeyCode::Back => "BS",
                            VirtualKeyCode::Home => "Home",
                            VirtualKeyCode::Delete => "Del",
                            VirtualKeyCode::End => "End",
                            VirtualKeyCode::PageDown => "PageDown",
                            VirtualKeyCode::PageUp => "PageUp",
                            VirtualKeyCode::Left => "Left",
                            VirtualKeyCode::Up => "Up",
                            VirtualKeyCode::Right => "Right",
                            VirtualKeyCode::Down => "Down",
                            VirtualKeyCode::Return => "Enter",
                            VirtualKeyCode::Space => "Space",
                            VirtualKeyCode::Numpad0 => "k0",
                            VirtualKeyCode::Numpad1 => "k1",
                            VirtualKeyCode::Numpad2 => "k2",
                            VirtualKeyCode::Numpad3 => "k3",
                            VirtualKeyCode::Numpad4 => "k4",
                            VirtualKeyCode::Numpad5 => "k5",
                            VirtualKeyCode::Numpad6 => "k6",
                            VirtualKeyCode::Numpad7 => "k7",
                            VirtualKeyCode::Numpad8 => "k8",
                            VirtualKeyCode::Numpad9 => "k9",
                            VirtualKeyCode::NumpadAdd => "kPlus",
                            VirtualKeyCode::NumpadDivide => "kDivide",
                            VirtualKeyCode::NumpadDecimal => "kPoint",
                            VirtualKeyCode::NumpadComma => "kComma",
                            VirtualKeyCode::NumpadEnter => "kEnter",
                            VirtualKeyCode::NumpadEquals => "kEqual",
                            VirtualKeyCode::NumpadMultiply => "kMultiply",
                            VirtualKeyCode::NumpadSubtract => "kMinus",
                            VirtualKeyCode::Tab => "Tab",
                            VirtualKeyCode::Key1 => "1",
                            VirtualKeyCode::Key2 => "2",
                            VirtualKeyCode::Key3 => "3",
                            VirtualKeyCode::Key4 => "4",
                            VirtualKeyCode::Key5 => "5",
                            VirtualKeyCode::Key6 => "6",
                            VirtualKeyCode::Key7 => "7",
                            VirtualKeyCode::Key8 => "8",
                            VirtualKeyCode::Key9 => "9",
                            VirtualKeyCode::Key0 => "0",
                            VirtualKeyCode::A => "a",
                            VirtualKeyCode::B => "b",
                            VirtualKeyCode::C => "c",
                            VirtualKeyCode::D => "d",
                            VirtualKeyCode::E => "e",
                            VirtualKeyCode::F => "f",
                            VirtualKeyCode::G => "g",
                            VirtualKeyCode::H => "h",
                            VirtualKeyCode::I => "i",
                            VirtualKeyCode::J => "j",
                            VirtualKeyCode::K => "k",
                            VirtualKeyCode::L => "l",
                            VirtualKeyCode::M => "m",
                            VirtualKeyCode::N => "n",
                            VirtualKeyCode::O => "o",
                            VirtualKeyCode::P => "p",
                            VirtualKeyCode::Q => "q",
                            VirtualKeyCode::R => "r",
                            VirtualKeyCode::S => "s",
                            VirtualKeyCode::T => "t",
                            VirtualKeyCode::U => "u",
                            VirtualKeyCode::V => "v",
                            VirtualKeyCode::W => "w",
                            VirtualKeyCode::X => "x",
                            VirtualKeyCode::Y => "y",
                            VirtualKeyCode::Z => "z",
                            _ => return None,
                        })
                    };
                    if let Some(c) = c() {
                        send_keys(c, &mut modifiers, neovim.as_mut().unwrap(), false);
                    }
                }

                WindowEvent::CursorMoved { position, .. } => {
                    let position: Vec2<f64> = (*position).into();
                    let position: Vec2<i64> = position.cast_as();
                    let cell_size = fonts.read().metrics().into_pixels().cell_size();
                    let inner = (surface_size / cell_size) * cell_size;
                    let margin = (surface_size - inner) / 2;
                    let position = position - margin.cast();
                    let Ok(position) = position.try_cast::<u64>() else {
                        return;
                    };
                    mouse.position = position;
                    if let Some(grid) = ui.read().unwrap().grid_under_cursor(
                        position,
                        fonts.read().metrics().into_pixels().cell_size().cast(),
                    ) {
                        neovim.as_ref().unwrap().input_mouse(
                            mouse.buttons.first().unwrap_or(Button::Move),
                            // Irrelevant for move
                            Action::ButtonDrag,
                            modifiers.into(),
                            grid.grid,
                            grid.position.y,
                            grid.position.x,
                        );
                    }
                }

                WindowEvent::MouseInput { state, button, .. } => {
                    let Ok(button) = (*button).try_into() else {
                        return;
                    };

                    let action = (*state).into();
                    let depressed = match action {
                        Action::ButtonPress => true,
                        Action::ButtonRelease => false,
                        _ => unreachable!(),
                    };
                    match button {
                        Button::Left => mouse.buttons.set_left(depressed),
                        Button::Right => mouse.buttons.set_right(depressed),
                        Button::Middle => mouse.buttons.set_middle(depressed),
                        _ => unreachable!(),
                    }
                    if let Some(grid) = ui.read().unwrap().grid_under_cursor(
                        mouse.position,
                        fonts.read().metrics().into_pixels().cell_size().cast(),
                    ) {
                        neovim.as_ref().unwrap().input_mouse(
                            button,
                            action,
                            modifiers.into(),
                            grid.grid,
                            grid.position.y,
                            grid.position.x,
                        );
                    }
                }

                WindowEvent::MouseWheel { delta, phase, .. } => {
                    let reset = matches!(
                        phase,
                        TouchPhase::Started | TouchPhase::Ended | TouchPhase::Cancelled
                    );

                    let (delta, kind) = match delta {
                        MouseScrollDelta::LineDelta(horizontal, vertical) => {
                            (Vec2::new(*horizontal, *vertical).cast(), ScrollKind::Lines)
                        }

                        MouseScrollDelta::PixelDelta(delta) => {
                            ((*delta).into(), ScrollKind::Pixels)
                        }
                    };

                    let modifiers = modifiers.into();

                    let delta: Vec2<i64> = delta.cast_as();
                    if reset {
                        mouse.scroll = Vec2::default();
                    }

                    let lines = match kind {
                        ScrollKind::Lines => delta,
                        ScrollKind::Pixels => {
                            mouse.scroll += delta;
                            let cell_size: Vec2<i64> =
                                fonts.read().metrics().into_pixels().cell_size().cast();
                            let lines = mouse.scroll / cell_size;
                            mouse.scroll -= lines * cell_size;
                            lines
                        }
                    };

                    let Some(grid) = ui.read().unwrap().grid_under_cursor(
                        mouse.position,
                        fonts.read().metrics().into_pixels().cell_size().cast(),
                    ) else {
                        return;
                    };

                    let action = if lines.y < 0 {
                        Action::WheelDown
                    } else {
                        Action::WheelUp
                    };

                    for _ in 0..lines.y.abs() {
                        neovim.as_ref().unwrap().input_mouse(
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
                        neovim.as_ref().unwrap().input_mouse(
                            Button::Wheel,
                            action,
                            modifiers,
                            grid.grid,
                            grid.position.y,
                            grid.position.x,
                        );
                    }
                }

                WindowEvent::Resized(physical_size) => {
                    surface_size = (*physical_size).into();
                    render_tx
                        .as_ref()
                        .unwrap()
                        .send(Message::Resize {
                            screen_size: surface_size,
                            cell_size: fonts.read().metrics().into_pixels().cell_size(),
                        })
                        .unwrap();
                    resize_neovim_grid(surface_size, &fonts.read(), neovim.as_ref().unwrap());
                }

                WindowEvent::ScaleFactorChanged {
                    new_inner_size,
                    // TODO: Use the scale factor
                    scale_factor: _,
                } => {
                    surface_size = (**new_inner_size).into();
                    render_tx
                        .as_ref()
                        .unwrap()
                        .send(Message::Resize {
                            screen_size: surface_size,
                            cell_size: fonts.read().metrics().into_pixels().cell_size(),
                        })
                        .unwrap();
                    resize_neovim_grid(surface_size, &fonts.read(), neovim.as_ref().unwrap());
                }

                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,

                _ => {}
            },

            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let framerate = window
                    .current_monitor()
                    .and_then(|monitor| monitor.refresh_rate_millihertz())
                    .unwrap_or(60_000);
                let delta_seconds = 1_000. / framerate as f32;
                render_tx
                    .as_ref()
                    .unwrap()
                    .send(Message::Redraw(delta_seconds, *settings.read().unwrap()))
                    .unwrap();
            }

            _ => {}
        }
    })
}

fn send_keys(c: &str, modifiers: &mut ModifiersState, neovim: &mut Neovim, ignore_shift: bool) {
    let shift = modifiers.shift() && !ignore_shift;
    let c = if modifiers.ctrl() || modifiers.alt() || modifiers.logo() || shift {
        let ctrl = if modifiers.ctrl() { "C" } else { "" };
        let shift = if shift { "S" } else { "" };
        let alt = if modifiers.alt() { "A" } else { "" };
        let logo = if modifiers.logo() { "D" } else { "" };
        format!("<{ctrl}{shift}{alt}{logo}-{c}>")
    } else {
        match c.len() {
            1 => c.to_string(),
            _ => format!("<{c}>"),
        }
    };
    neovim.input(c);
}

fn resize_neovim_grid(surface_size: Vec2<u32>, fonts: &Fonts, neovim: &Neovim) {
    let size = surface_size / fonts.metrics().into_pixels().cell_size();
    let size: Vec2<u64> = size.cast();
    neovim.ui_try_resize_grid(1, size.x, size.y);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Settings {
    /// Multiplier of the default cursor speed
    pub cursor_speed: f32,
    /// Multiplier of the default scroll speed
    pub scroll_speed: f32,
    /// Additional offset to apply to underlines
    pub underline_offset: i32,
}

impl Settings {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            cursor_speed: 1.,
            scroll_speed: 1.,
            underline_offset: 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Mouse {
    position: Vec2<u64>,
    scroll: Vec2<i64>,
    buttons: Buttons,
}

impl Mouse {
    pub fn new() -> Self {
        Self::default()
    }
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

pub enum ScrollKind {
    Lines,
    Pixels,
}

#[derive(Debug)]
pub enum UserEvent {
    Exit,
    ResizeGrid,
}

struct NeovimHandler {
    proxy: EventLoopProxy<UserEvent>,
    window: Arc<Window>,
    fonts: Arc<FontsHandle>,
    neovim: Neovim,
    settings: Arc<RwLock<Settings>>,
    ui: Arc<RwLock<Ui>>,
    render_tx: Sender<Message>,
}

impl StdoutHandler for NeovimHandler {
    fn handle_notification(&mut self, notification: rpc::Notification) {
        let Notification { method, params } = notification;
        match method.as_str() {
            "redraw" => {
                let mut ui = self.ui.write().unwrap();
                let mut flushed = false;
                for param in params {
                    match event::Event::try_parse(param.clone()) {
                        Ok(events) => {
                            for event in events.iter().cloned() {
                                log::info!("{event:?}");
                                match event {
                                    Event::Flush => {
                                        flushed = true;
                                        ui.process(Event::Flush);
                                    }
                                    Event::OptionSet(event) => {
                                        let updated_fonts = matches!(event, OptionSet::Guifont(_));
                                        ui.process(Event::OptionSet(event));
                                        if updated_fonts {
                                            // TODO: Probably extract from loop
                                            self.fonts.write().set_fonts(&ui.options.guifont);
                                            self.proxy.send_event(UserEvent::ResizeGrid).unwrap();
                                        }
                                    }
                                    event => ui.process(event),
                                }
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

                if flushed {
                    // TODO: Only send changes
                    self.render_tx
                        .send(Message::UpdateHighlights(HighlightUpdateInfo::from_ui(&ui)))
                        .unwrap();
                    self.render_tx
                        .send(Message::UpdateCursor(CursorUpdateInfo::from_ui(&ui)))
                        .unwrap();
                    self.render_tx
                        .send(Message::UpdateDrawOrder(ui.draw_order.clone()))
                        .unwrap();
                    for grid in ui.grids.iter() {
                        for grid in ui.deleted_grids.iter() {
                            self.render_tx.send(Message::DeleteGrid(*grid)).unwrap();
                        }
                        // TODO: Split grid and window updates
                        if grid.is_grid_dirty() || grid.is_window_dirty() {
                            self.render_tx
                                .send(Message::UpdateGrid {
                                    position: ui.position(grid.id),
                                    grid: grid.clone(),
                                })
                                .unwrap()
                        }
                    }
                    ui.clear_dirty();
                }
            }

            "neophyte.set_font_height" => {
                let mut args = Values::new(params.into_iter().next().unwrap()).unwrap();
                let height: f32 = args.next().unwrap();
                let size = ui::FontSize::Height(height);
                self.fonts.write().set_font_size(size);
                self.proxy.send_event(UserEvent::ResizeGrid).unwrap();
            }

            "neophyte.set_font_width" => {
                let mut args = Values::new(params.into_iter().next().unwrap()).unwrap();
                let width: f32 = args.next().unwrap();
                let size = ui::FontSize::Width(width);
                self.fonts.write().set_font_size(size);
                self.proxy.send_event(UserEvent::ResizeGrid).unwrap();
            }

            "neophyte.set_cursor_speed" => {
                let mut args = Values::new(params.into_iter().next().unwrap()).unwrap();
                let speed: f32 = args.next().unwrap();
                self.settings.write().unwrap().cursor_speed = speed;
            }

            "neophyte.set_scroll_speed" => {
                let mut args = Values::new(params.into_iter().next().unwrap()).unwrap();
                let speed: f32 = args.next().unwrap();
                self.settings.write().unwrap().scroll_speed = speed;
            }

            "neophyte.set_fonts" => {
                let mut args = Values::new(params.into_iter().next().unwrap()).unwrap();
                let mut font_names = vec![];
                while let Some(font) = args.next() {
                    font_names.push(font);
                }
                let mut lock = self.fonts.write();
                let em = lock.metrics().em;
                lock.set_fonts(&FontsSetting {
                    fonts: font_names,
                    size: FontSize::Height(em),
                });
                drop(lock);
                self.proxy.send_event(UserEvent::ResizeGrid).unwrap();
            }

            "neophyte.set_underline_offset" => {
                let mut args = Values::new(params.into_iter().next().unwrap()).unwrap();
                let offset: f32 = args.next().unwrap();
                let offset: i32 = offset as i32;
                self.settings.write().unwrap().underline_offset = offset;
                self.window.request_redraw();
            }

            _ => log::error!("Unrecognized notification: {method}"),
        }
    }

    fn handle_request(&mut self, request: rpc::Request) {
        let rpc::Request {
            msgid,
            method,
            params,
        } = request;
        match method.as_str() {
            "neophyte.get_fonts" => {
                let names = self
                    .fonts
                    .read()
                    .iter()
                    .map(|font| font.name.clone())
                    .collect();
                self.neovim
                    .send_response(rpc::Response::result(msgid, names));
            }

            "neophyte.get_cursor_speed" => {
                let cursor_speed = self.settings.read().unwrap().cursor_speed;
                self.neovim
                    .send_response(rpc::Response::result(msgid, cursor_speed.into()));
            }

            "neophyte.get_scroll_speed" => {
                let scroll_speed = self.settings.read().unwrap().scroll_speed;
                self.neovim
                    .send_response(rpc::Response::result(msgid, scroll_speed.into()));
            }

            "neophyte.get_font_width" => {
                let width = self.fonts.read().metrics().width;
                self.neovim
                    .send_response(rpc::Response::result(msgid, width.into()));
            }

            "neophyte.get_font_height" => {
                let width = self.fonts.read().metrics().em;
                self.neovim
                    .send_response(rpc::Response::result(msgid, width.into()));
            }

            "neophyte.get_underline_offset" => {
                let offset = self.settings.read().unwrap().underline_offset;
                self.neovim
                    .send_response(rpc::Response::result(msgid, offset.into()));
            }

            _ => log::error!("Unknown request: {}, {:?}", method, params),
        }
    }

    fn handle_shutdown(&mut self) {
        let _ = self.proxy.send_event(UserEvent::Exit);
    }
}
