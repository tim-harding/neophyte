mod event;
mod neovim;
mod rendering;
mod rpc;
pub mod text;
mod ui;
mod util;

use neovim::{Neovim, StdoutHandler};
use rendering::{Notification, RenderEvent, RenderLoop, RequestKind, ScrollKind};
use std::{
    sync::{
        mpsc::{self, Sender},
        Arc, RwLock,
    },
    thread,
};
use text::fonts::FontsHandle;
use util::{vec2::Vec2, Values};
use winit::{
    event::{
        ElementState, KeyboardInput, ModifiersState, MouseScrollDelta, TouchPhase, VirtualKeyCode,
        WindowEvent,
    },
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy},
    window::WindowBuilder,
};

fn main() {
    env_logger::builder().format_timestamp(None).init();
    let (render_tx, render_rx) = mpsc::channel();
    let (neovim, stdout_handler, stdin_handler) = Neovim::new().unwrap();
    let settings = Arc::new(RwLock::new(Settings::new()));
    let fonts = Arc::new(FontsHandle::new());

    neovim.ui_attach();
    let event_loop = EventLoopBuilder::<()>::with_user_event().build();
    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());

    let mut stdout_thread = Some({
        let render_tx = render_tx.clone();
        let proxy = event_loop.create_proxy();
        let neovim = neovim.clone();
        let settings = settings.clone();
        thread::spawn(move || stdout_thread(stdout_handler, render_tx, proxy, neovim, settings))
    });

    let mut stdin_thread = Some(std::thread::spawn(move || stdin_handler.start()));

    let mut render_loop_thread = Some({
        let window = window.clone();
        let neovim = neovim.clone();
        thread::spawn(move || {
            let render_loop = RenderLoop::new(window, neovim, fonts.clone(), settings.clone());
            render_loop.run(render_rx);
        })
    });

    let mut neovim = Some(neovim);
    let mut render_tx = Some(render_tx);
    let mut modifiers = ModifiersState::default();
    event_loop.run(move |event, _, control_flow| {
        use winit::event::Event;
        *control_flow = ControlFlow::Wait;
        match event {
            Event::UserEvent(()) => {
                // Already terminated since it generated the user event
                stdout_thread.take().unwrap().join().unwrap();

                // Consume the last RenderEvent sender so that the render thread
                // knows to close
                let _ = render_tx.take();
                render_loop_thread.take().unwrap().join().unwrap();

                // Consume the last Neovim instance so the stdin handler knows
                // to close
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
                    render_tx
                        .as_ref()
                        .unwrap()
                        .send(RenderEvent::MouseMove {
                            position: (*position).into(),
                            modifiers: modifiers.into(),
                        })
                        .unwrap();
                }

                WindowEvent::MouseInput { state, button, .. } => {
                    let Ok(button) = (*button).try_into() else {
                        return;
                    };

                    render_tx
                        .as_ref()
                        .unwrap()
                        .send(RenderEvent::Click {
                            button,
                            action: (*state).into(),
                            modifiers: modifiers.into(),
                        })
                        .unwrap();
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
                    render_tx
                        .as_ref()
                        .unwrap()
                        .send(RenderEvent::Scroll {
                            delta,
                            kind,
                            reset,
                            modifiers,
                        })
                        .unwrap();
                }

                WindowEvent::Resized(physical_size) => {
                    render_tx
                        .as_ref()
                        .unwrap()
                        .send(RenderEvent::Resized((*physical_size).into()))
                        .unwrap();
                }

                // TODO: Use the scale factor
                WindowEvent::ScaleFactorChanged {
                    new_inner_size,
                    scale_factor: _,
                } => {
                    render_tx
                        .as_ref()
                        .unwrap()
                        .send(RenderEvent::Resized((**new_inner_size).into()))
                        .unwrap();
                }

                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,

                _ => {}
            },

            Event::RedrawRequested(window_id) if window_id == window.id() => {
                render_tx
                    .as_ref()
                    .unwrap()
                    .send(RenderEvent::RequestRedraw)
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

fn stdout_thread(
    stdout_handler: StdoutHandler,
    render_tx: Sender<RenderEvent>,
    proxy: EventLoopProxy<()>,
    neovim: Neovim,
    settings: Arc<RwLock<Settings>>,
) {
    stdout_handler.start(
        |rpc::Notification { method, params }| match method.as_str() {
            "redraw" => {
                for param in params {
                    match event::Event::try_parse(param.clone()) {
                        Ok(events) => render_tx
                            .send(RenderEvent::Notification(Notification::Redraw(events)))
                            .unwrap(),
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
                render_tx
                    .send(RenderEvent::Notification(Notification::SetFontSize(size)))
                    .unwrap();
            }

            "neophyte.set_font_width" => {
                let mut args = Values::new(params.into_iter().next().unwrap()).unwrap();
                let width: f32 = args.next().unwrap();
                let size = ui::FontSize::Width(width);
                render_tx
                    .send(RenderEvent::Notification(Notification::SetFontSize(size)))
                    .unwrap();
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
                let mut fonts = vec![];
                while let Some(font) = args.next() {
                    fonts.push(font);
                }
                render_tx
                    .send(RenderEvent::Notification(Notification::SetFonts(fonts)))
                    .unwrap();
            }

            _ => log::error!("Unrecognized notification: {method}"),
        },
        |rpc::Request {
             msgid,
             method,
             params,
         }| {
            match method.as_str() {
                "neophyte.get_fonts" => render_tx
                    .send(RenderEvent::Request(rendering::Request {
                        msgid,
                        kind: RequestKind::Fonts,
                    }))
                    .unwrap(),

                "neophyte.get_cursor_speed" => {
                    let cursor_speed = settings.read().unwrap().cursor_speed;
                    neovim.send_response(rpc::Response::result(msgid, cursor_speed.into()));
                }

                "neophyte.get_scroll_speed" => {
                    let scroll_speed = settings.read().unwrap().scroll_speed;
                    neovim.send_response(rpc::Response::result(msgid, scroll_speed.into()));
                }

                "neophyte.get_font_width" => render_tx
                    .send(RenderEvent::Request(rendering::Request {
                        msgid,
                        kind: RequestKind::FontWidth,
                    }))
                    .unwrap(),

                "neophyte.get_font_height" => render_tx
                    .send(RenderEvent::Request(rendering::Request {
                        msgid,
                        kind: RequestKind::FontHeight,
                    }))
                    .unwrap(),

                _ => log::error!("Unknown request: {}, {:?}", method, params),
            }
        },
        || {
            let _ = proxy.send_event(());
        },
    );
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
