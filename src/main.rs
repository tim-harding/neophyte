mod event;
mod rendering;
mod rpc;
mod session;
pub mod text;
mod ui;
mod util;

use rendering::{RenderEvent, RenderLoop};
use session::Neovim;
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};
use winit::{
    event::{ElementState, KeyboardInput, ModifiersState, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    env_logger::builder().format_timestamp(None).init();
    let (render_tx, render_rx) = mpsc::channel();
    let (mut neovim, handler) = Neovim::new().unwrap();

    let wants_shutdown = Arc::new(Mutex::new(false));
    {
        let render_tx = render_tx.clone();
        let wants_shutdown = wants_shutdown.clone();
        thread::spawn(move || {
            handler.start(
                |method, params| {
                    render_tx
                        .send(RenderEvent::Notification(method, params))
                        .unwrap();
                },
                || {
                    *wants_shutdown.lock().unwrap() = true;
                },
            );
        });
    }

    neovim.ui_attach();
    let event_loop = EventLoop::new();
    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());

    {
        let window = window.clone();
        let neovim = neovim.clone();
        thread::spawn(move || {
            let render_loop = RenderLoop::new(window, neovim);
            render_loop.run(render_rx);
        });
    }

    let mut modifiers = ModifiersState::default();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = if *wants_shutdown.lock().unwrap() {
            ControlFlow::Exit
        } else {
            ControlFlow::Wait
        };
        match event {
            winit::event::Event::WindowEvent {
                window_id,
                ref event,
            } if window_id == window.id() => match event {
                WindowEvent::ModifiersChanged(new_modifiers) => {
                    modifiers = *new_modifiers;
                }

                WindowEvent::ReceivedCharacter(c) => {
                    if !c.is_control()
                        && !c.is_whitespace()
                        && !c.is_ascii_digit()
                        && !c.is_ascii_alphabetic()
                    {
                        send_keys(&format!("{c}"), &mut modifiers, &mut neovim, true);
                    }
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
                        send_keys(c, &mut modifiers, &mut neovim, false);
                    }
                }

                WindowEvent::Resized(physical_size) => {
                    render_tx
                        .send(RenderEvent::Resized(*physical_size))
                        .unwrap();
                }

                // TODO: Use the scale factor
                WindowEvent::ScaleFactorChanged {
                    new_inner_size,
                    scale_factor: _,
                } => {
                    render_tx
                        .send(RenderEvent::Resized(**new_inner_size))
                        .unwrap();
                }

                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,

                _ => {}
            },

            winit::event::Event::RedrawRequested(window_id) if window_id == window.id() => {
                render_tx.send(RenderEvent::Redraw).unwrap();
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
