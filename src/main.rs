mod event;
mod neovim;
mod rendering;
mod rpc;
pub mod text;
mod ui;
mod util;

use bitfield_struct::bitfield;
use event::{Event, OptionSet, SetTitle};
use neovim::Neovim;
use rendering::state::RenderState;
use std::{
    sync::{Arc, RwLock},
    thread,
};
use text::fonts::{Fonts, FontsHandle};
use ui::{FontSize, FontsSetting};
use util::{vec2::Vec2, Values};
use winit::{
    event::{
        ElementState, KeyboardInput, ModifiersState, MouseScrollDelta, TouchPhase, VirtualKeyCode,
        WindowEvent,
    },
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};

use crate::neovim::{Action, Button};

fn main() {
    env_logger::builder().format_timestamp(None).init();
    let (neovim, stdout_handler, stdin_handler) = Neovim::new().unwrap();
    let settings = Arc::new(RwLock::new(Settings::new()));
    let fonts = Arc::new(FontsHandle::new());
    let event_loop = EventLoopBuilder::<()>::with_user_event().build();
    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());
    let (ui_handle, ui_thread) = ui::actor::new();
    let render_state = Arc::new(RwLock::new(pollster::block_on(async {
        RenderState::new(
            window.clone(),
            fonts.read().metrics().into_pixels().cell_size(),
        )
        .await
    })));
    neovim.ui_attach();

    let mut ui_thread = Some(ui_thread.run());

    let mut stdout_thread = Some({
        let proxy = event_loop.create_proxy();
        let render_state = render_state.clone();
        let fonts = fonts.clone();
        let neovim = neovim.clone();
        let settings = settings.clone();
        let ui_handle = ui_handle.clone();
        let window = window.clone();
        thread::spawn(move || {
            stdout_handler.start(
                |rpc::Notification { method, params }| match method.as_str() {
                    "redraw" => {
                        for param in params {
                            match event::Event::try_parse(param.clone()) {
                                Ok(events) => {
                                    for event in events.iter().cloned() {
                                        log::info!("{event:?}");
                                        match event {
                                            Event::Flush => {
                                                let mut render_state =
                                                    render_state.write().unwrap();
                                                render_state
                                                    .update(&ui_handle.get(), fonts.as_ref());
                                                ui_handle.process(Event::Flush);
                                                render_state.request_redraw();
                                            }
                                            Event::SetTitle(SetTitle { title }) => {
                                                window.set_title(&title)
                                            }
                                            Event::OptionSet(event) => {
                                                let is_gui_font =
                                                    matches!(event, OptionSet::Guifont(_));
                                                ui_handle.process(Event::OptionSet(event));
                                                if is_gui_font {
                                                    fonts.write().set_fonts(
                                                        &ui_handle.get().options.guifont,
                                                    );
                                                    resize_neovim_grid(
                                                        &render_state.read().unwrap(),
                                                        &fonts.read(),
                                                        &neovim,
                                                    );
                                                }
                                            }
                                            event => ui_handle.process(event),
                                        }
                                    }

                                    ui_handle.swap();
                                    for event in events {
                                        ui_handle.process(event);
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

                    "neophyte.set_font_height" => {
                        let mut args = Values::new(params.into_iter().next().unwrap()).unwrap();
                        let height: f32 = args.next().unwrap();
                        let size = ui::FontSize::Height(height);
                        fonts.write().set_font_size(size);
                        resize_neovim_grid(&render_state.read().unwrap(), &fonts.read(), &neovim);
                    }

                    "neophyte.set_font_width" => {
                        let mut args = Values::new(params.into_iter().next().unwrap()).unwrap();
                        let width: f32 = args.next().unwrap();
                        let size = ui::FontSize::Width(width);
                        fonts.write().set_font_size(size);
                        resize_neovim_grid(&render_state.read().unwrap(), &fonts.read(), &neovim);
                    }

                    "neophyte.set_cursor_speed" => {
                        let mut args = Values::new(params.into_iter().next().unwrap()).unwrap();
                        let speed: f32 = args.next().unwrap();
                        settings.write().unwrap().cursor_speed = speed;
                    }

                    "neophyte.set_scroll_speed" => {
                        let mut args = Values::new(params.into_iter().next().unwrap()).unwrap();
                        let speed: f32 = args.next().unwrap();
                        settings.write().unwrap().scroll_speed = speed;
                    }

                    "neophyte.set_fonts" => {
                        let mut args = Values::new(params.into_iter().next().unwrap()).unwrap();
                        let mut font_names = vec![];
                        while let Some(font) = args.next() {
                            font_names.push(font);
                        }
                        let mut lock = fonts.write();
                        let em = lock.metrics().em;
                        lock.set_fonts(&FontsSetting {
                            fonts: font_names,
                            size: FontSize::Height(em),
                        });
                        drop(lock);
                        resize_neovim_grid(&render_state.read().unwrap(), &fonts.read(), &neovim);
                    }

                    _ => log::error!("Unrecognized notification: {method}"),
                },
                |rpc::Request {
                     msgid,
                     method,
                     params,
                 }| {
                    match method.as_str() {
                        "neophyte.get_fonts" => {
                            let names = fonts.read().iter().map(|font| font.name.clone()).collect();
                            neovim.send_response(rpc::Response::result(msgid, names));
                        }

                        "neophyte.get_cursor_speed" => {
                            let cursor_speed = settings.read().unwrap().cursor_speed;
                            neovim.send_response(rpc::Response::result(msgid, cursor_speed.into()));
                        }

                        "neophyte.get_scroll_speed" => {
                            let scroll_speed = settings.read().unwrap().scroll_speed;
                            neovim.send_response(rpc::Response::result(msgid, scroll_speed.into()));
                        }

                        "neophyte.get_font_width" => {
                            let width = fonts.read().metrics().width;
                            neovim.send_response(rpc::Response::result(msgid, width.into()));
                        }

                        "neophyte.get_font_height" => {
                            let width = fonts.read().metrics().em;
                            neovim.send_response(rpc::Response::result(msgid, width.into()));
                        }

                        _ => log::error!("Unknown request: {}, {:?}", method, params),
                    }
                },
                || {
                    let _ = proxy.send_event(());
                },
            );
        })
    });

    let mut stdin_thread = Some(std::thread::spawn(move || stdin_handler.start()));

    let mut mouse = Mouse::new();
    let mut ui_handle = Some(ui_handle);
    let mut neovim = Some(neovim);
    let mut modifiers = ModifiersState::default();
    event_loop.run(move |event, _, control_flow| {
        use winit::event::Event;
        *control_flow = ControlFlow::Wait;
        match event {
            Event::UserEvent(()) => {
                // Already terminated since it generated the user event
                stdout_thread.take().unwrap().join().unwrap();

                // Consume the last UI handle to close the channel
                let _ = ui_handle.take();
                ui_thread.take().unwrap().join().unwrap();

                // Consume the last Neovim instance to close the channel
                let _ = neovim.take();
                stdin_thread.take().unwrap().join().unwrap();

                *control_flow = ControlFlow::Exit;
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
                    let surface_size = render_state.read().unwrap().surface_size();
                    let cell_size = fonts.read().metrics().into_pixels().cell_size();
                    let inner = (surface_size / cell_size) * cell_size;
                    let margin = (surface_size - inner) / 2;
                    let position = position - margin.cast();
                    let Ok(position) = position.try_cast::<u64>() else {
                        return;
                    };
                    mouse.position = position;
                    if let Some(grid) = ui_handle.as_ref().unwrap().get().grid_under_cursor(
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
                    if let Some(grid) = ui_handle.as_ref().unwrap().get().grid_under_cursor(
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

                    let Some(grid) = ui_handle.as_ref().unwrap().get().grid_under_cursor(
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
                    render_state.write().unwrap().resize(
                        (*physical_size).into(),
                        fonts.read().metrics().into_pixels().cell_size(),
                    );
                    resize_neovim_grid(
                        &render_state.read().unwrap(),
                        &fonts.read(),
                        neovim.as_ref().unwrap(),
                    );
                }

                // TODO: Use the scale factor
                WindowEvent::ScaleFactorChanged {
                    new_inner_size,
                    scale_factor: _,
                } => {
                    render_state.write().unwrap().resize(
                        (**new_inner_size).into(),
                        fonts.read().metrics().into_pixels().cell_size(),
                    );
                    resize_neovim_grid(
                        &render_state.read().unwrap(),
                        &fonts.read(),
                        neovim.as_ref().unwrap(),
                    );
                }

                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,

                _ => {}
            },

            Event::MainEventsCleared => window.request_redraw(),

            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let framerate = window
                    .current_monitor()
                    .and_then(|monitor| monitor.refresh_rate_millihertz())
                    .unwrap_or(60000);
                render_state.write().unwrap().maybe_render(
                    fonts.read().metrics().into_pixels().cell_size(),
                    framerate,
                    *settings.read().unwrap(),
                );
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

fn resize_neovim_grid(render_state: &RenderState, fonts: &Fonts, neovim: &Neovim) {
    let surface_size = render_state.surface_size();
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
