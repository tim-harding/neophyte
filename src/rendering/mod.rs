pub mod state;
mod state_font;
mod state_read;
mod state_surface_config;
mod state_write;
mod texture;

use self::state::{State, StateWinit};
use crate::{
    session::Neovim,
    text::font::{metrics, Font},
    ui::Ui,
};
use std::{
    sync::{mpsc::Receiver, Arc},
    thread,
};
use swash::FontRef;
use wgpu::SurfaceError;
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, ModifiersState, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub async fn run(rx: Receiver<Ui>, mut neovim: Neovim) {
    let font = Font::from_file("/usr/share/fonts/OTF/CascadiaCode-Regular.otf", 0).unwrap();
    let event_loop = EventLoop::new();
    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());
    let mut state = State::new(window.clone(), font.clone()).await; // TODO: Font cloning
    let winit_state = state.winit_state();

    {
        let window = window.clone();
        thread::spawn(move || {
            while let Ok(ui) = rx.recv() {
                state.update_text(ui);
                window.request_redraw();
            }
        });
    }

    let mut modifiers = ModifiersState::default();
    let mut scale_factor = 1.0;
    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            window_id,
            ref event,
        } if window_id == window.id() => match event {
            WindowEvent::ModifiersChanged(new_modifiers) => {
                modifiers = *new_modifiers;
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
                        VirtualKeyCode::F13 => "F13",
                        VirtualKeyCode::F14 => "F14",
                        VirtualKeyCode::F15 => "F15",
                        VirtualKeyCode::F16 => "F16",
                        VirtualKeyCode::F17 => "F17",
                        VirtualKeyCode::F18 => "F18",
                        VirtualKeyCode::F19 => "F19",
                        VirtualKeyCode::F20 => "F20",
                        VirtualKeyCode::F21 => "F21",
                        VirtualKeyCode::F22 => "F22",
                        VirtualKeyCode::F23 => "F23",
                        VirtualKeyCode::F24 => "F24",
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
                        VirtualKeyCode::Caret => "^",
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
                        VirtualKeyCode::Apostrophe => "'",
                        VirtualKeyCode::Asterisk => "*",
                        VirtualKeyCode::Backslash => "Bslash",
                        VirtualKeyCode::Colon => ":",
                        VirtualKeyCode::Comma => ",",
                        VirtualKeyCode::Equals => "=",
                        VirtualKeyCode::Grave => "`",
                        VirtualKeyCode::Minus => "-",
                        VirtualKeyCode::Period => ".",
                        VirtualKeyCode::Plus => "+",
                        VirtualKeyCode::RBracket => "[",
                        VirtualKeyCode::Semicolon => ";",
                        VirtualKeyCode::Slash => "/",
                        VirtualKeyCode::Tab => "Tab",
                        VirtualKeyCode::Underline => "_",
                        _ => return None,
                    })
                };
                if let Some(c) = c() {
                    let c = if modifiers.is_empty() {
                        match c.len() {
                            1 => c.to_string(),
                            _ => format!("<{c}>"),
                        }
                    } else {
                        let ctrl = if modifiers.ctrl() { "C" } else { "" };
                        let shift = if modifiers.shift() { "S" } else { "" };
                        let alt = if modifiers.alt() { "A" } else { "" };
                        let logo = if modifiers.logo() { "D" } else { "" };
                        format!("<{ctrl}{shift}{alt}{logo}-{c}>")
                    };
                    neovim.input(c);
                }
            }

            WindowEvent::Resized(physical_size) => {
                resize(
                    &winit_state,
                    scale_factor,
                    *physical_size,
                    &mut neovim,
                    font.as_ref(),
                );
            }

            WindowEvent::ScaleFactorChanged {
                new_inner_size,
                scale_factor: new_scale_factor,
            } => {
                scale_factor = *new_scale_factor as f32;
                resize(
                    &winit_state,
                    scale_factor,
                    **new_inner_size,
                    &mut neovim,
                    font.as_ref(),
                );
            }

            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,

            _ => {}
        },

        Event::RedrawRequested(window_id) if window_id == window.id() => {
            match winit_state.render() {
                Ok(_) => {}
                Err(SurfaceError::Lost) => winit_state.rebuild_swap_chain(),
                Err(SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{e:?}"),
            }
        }

        _ => {}
    })
}

fn resize(
    state: &StateWinit,
    scale: f32,
    size: PhysicalSize<u32>,
    neovim: &mut Neovim,
    font: FontRef,
) {
    state.resize(size);
    let metrics = metrics(font, 24.0 * scale);
    neovim.ui_try_resize_grid(
        1,
        size.width as u64 / metrics.advance as u64,
        size.height as u64 / metrics.cell_height() as u64,
    )
}
