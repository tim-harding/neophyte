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
use ui::{
    options::{FontSize, FontsSetting},
    Ui,
};
use util::{vec2::Vec2, Values};
use winit::{
    event::{ElementState, MouseScrollDelta, TouchPhase, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy},
    keyboard::{Key, ModifiersState, NamedKey},
    window::{Window, WindowBuilder},
};

fn main() {
    env_logger::builder().format_timestamp(None).init();
    let (neovim, stdout_handler, stdin_handler) = Neovim::new().unwrap();
    let fonts = Arc::new(FontsHandle::new());
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event()
        .build()
        .unwrap();
    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());
    let ui = Arc::new(RwLock::new(Ui::new()));
    let render_state = pollster::block_on(async {
        let cell_size = fonts.read().cell_size();
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
            settings: Settings::new(),
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
    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop
        .run(move |event, window_target| {
            use winit::event::Event;
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

                            window_target.exit();
                        }

                        UserEvent::ResizeGrid => {
                            resize_neovim_grid(
                                surface_size,
                                &fonts.read(),
                                neovim.as_ref().unwrap(),
                            );
                        }
                    }
                }

                Event::WindowEvent {
                    window_id,
                    ref event,
                } if window_id == window.id() => match event {
                    WindowEvent::ModifiersChanged(new_modifiers) => {
                        modifiers = new_modifiers.state();
                    }

                    WindowEvent::KeyboardInput { event, .. } => {
                        match event.state {
                            ElementState::Pressed => {}
                            ElementState::Released => return,
                        }
                        match &event.logical_key {
                            Key::Named(key) => {
                                let c = || {
                                    Some(match key {
                                        NamedKey::Enter => "Enter",
                                        NamedKey::Tab => "Tab",
                                        NamedKey::Space => "Space",
                                        NamedKey::ArrowDown => "Down",
                                        NamedKey::ArrowLeft => "Left",
                                        NamedKey::ArrowRight => "Right",
                                        NamedKey::ArrowUp => "Up",
                                        NamedKey::End => "End",
                                        NamedKey::Home => "Home",
                                        NamedKey::PageDown => "PageDown",
                                        NamedKey::PageUp => "PageUp",
                                        NamedKey::Backspace => "BS",
                                        NamedKey::Delete => "Del",
                                        NamedKey::Escape => "Esc",
                                        NamedKey::F1 => "F1",
                                        NamedKey::F2 => "F2",
                                        NamedKey::F3 => "F3",
                                        NamedKey::F4 => "F4",
                                        NamedKey::F5 => "F5",
                                        NamedKey::F6 => "F6",
                                        NamedKey::F7 => "F7",
                                        NamedKey::F8 => "F8",
                                        NamedKey::F9 => "F9",
                                        NamedKey::F10 => "F10",
                                        NamedKey::F11 => "F11",
                                        NamedKey::F12 => "F12",
                                        NamedKey::F13 => "F13",
                                        NamedKey::F14 => "F14",
                                        NamedKey::F15 => "F15",
                                        NamedKey::F16 => "F16",
                                        NamedKey::F17 => "F17",
                                        NamedKey::F18 => "F18",
                                        NamedKey::F19 => "F19",
                                        NamedKey::F20 => "F20",
                                        NamedKey::F21 => "F21",
                                        NamedKey::F22 => "F22",
                                        NamedKey::F23 => "F23",
                                        NamedKey::F24 => "F24",
                                        NamedKey::F25 => "F25",
                                        NamedKey::F26 => "F26",
                                        NamedKey::F27 => "F27",
                                        NamedKey::F28 => "F28",
                                        NamedKey::F29 => "F29",
                                        NamedKey::F30 => "F30",
                                        NamedKey::F31 => "F31",
                                        NamedKey::F32 => "F32",
                                        NamedKey::F33 => "F33",
                                        NamedKey::F34 => "F34",
                                        NamedKey::F35 => "F35",
                                        _ => return None,
                                    })
                                };

                                if let Some(c) = c() {
                                    send_keys(c, &mut modifiers, neovim.as_mut().unwrap(), false);
                                }
                            }

                            Key::Character(c) => {
                                let s = match c.as_str() {
                                    "<" => "Lt",
                                    "\\" => "Bslash",
                                    "|" => "Bar",
                                    _ => c.as_str(),
                                };
                                send_keys(s, &mut modifiers, neovim.as_mut().unwrap(), true);
                            }

                            Key::Unidentified(_) | Key::Dead(_) => {}
                        }
                    }

                    WindowEvent::CursorMoved { position, .. } => {
                        let position: Vec2<f64> = (*position).into();
                        let position: Vec2<i64> = position.cast_as();
                        let cell_size = fonts.read().cell_size();
                        let inner = (surface_size / cell_size) * cell_size;
                        let margin = (surface_size - inner) / 2;
                        let position = position - margin.cast();
                        let Ok(position) = position.try_cast::<u64>() else {
                            return;
                        };
                        mouse.position = position;
                        if let Some(grid) = ui
                            .read()
                            .unwrap()
                            .grid_under_cursor(position, fonts.read().cell_size().cast())
                        {
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
                            Button::Left => mouse.buttons = mouse.buttons.with_left(depressed),
                            Button::Right => mouse.buttons = mouse.buttons.with_right(depressed),
                            Button::Middle => mouse.buttons = mouse.buttons.with_middle(depressed),
                            _ => unreachable!(),
                        }
                        if let Some(grid) = ui
                            .read()
                            .unwrap()
                            .grid_under_cursor(mouse.position, fonts.read().cell_size().cast())
                        {
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
                                let cell_size: Vec2<i64> = fonts.read().cell_size().cast();
                                let lines = mouse.scroll / cell_size;
                                mouse.scroll -= lines * cell_size;
                                lines
                            }
                        };

                        let Some(grid) = ui
                            .read()
                            .unwrap()
                            .grid_under_cursor(mouse.position, fonts.read().cell_size().cast())
                        else {
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
                                cell_size: fonts.read().cell_size(),
                            })
                            .unwrap();
                        resize_neovim_grid(surface_size, &fonts.read(), neovim.as_ref().unwrap());
                    }

                    WindowEvent::CloseRequested => window_target.exit(),

                    WindowEvent::RedrawRequested => {
                        let framerate = window
                            .current_monitor()
                            .and_then(|monitor| monitor.refresh_rate_millihertz())
                            .unwrap_or(60_000);
                        let delta_seconds = 1_000. / framerate as f32;
                        render_tx
                            .as_ref()
                            .unwrap()
                            .send(Message::Redraw(delta_seconds))
                            .unwrap();
                    }

                    _ => {} // TODO: Handle scale factor change
                },

                _ => {}
            }
        })
        .unwrap();
}

fn send_keys(c: &str, modifiers: &mut ModifiersState, neovim: &mut Neovim, ignore_shift: bool) {
    let shift = modifiers.shift_key() && !ignore_shift;
    let ctrl = modifiers.control_key();
    let alt = modifiers.alt_key();
    let logo = modifiers.super_key();
    let c = if ctrl || alt || logo || shift {
        let ctrl = if ctrl { "C" } else { "" };
        let shift = if shift { "S" } else { "" };
        let alt = if alt { "A" } else { "" };
        let logo = if logo { "D" } else { "" };
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
    let size = surface_size / fonts.cell_size();
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

#[derive(PartialEq, Eq, Clone, Copy, Default, PartialOrd, Ord, Debug)]
struct Buttons(u8);

#[rustfmt::skip]
impl Buttons {
    const LEFT:   u8 = 0b001;
    const RIGHT:  u8 = 0b010;
    const MIDDLE: u8 = 0b100;
}

impl Buttons {
    pub const fn with_left(self, value: bool) -> Self {
        Self(self.0 | (Self::LEFT * value as u8))
    }

    pub const fn with_right(self, value: bool) -> Self {
        Self(self.0 | (Self::RIGHT * value as u8))
    }

    pub const fn with_middle(self, value: bool) -> Self {
        Self(self.0 | (Self::MIDDLE * value as u8))
    }

    pub const fn left(self) -> bool {
        self.0 & Self::LEFT > 0
    }

    pub const fn right(self) -> bool {
        self.0 & Self::RIGHT > 0
    }

    pub const fn middle(self) -> bool {
        self.0 & Self::MIDDLE > 0
    }

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
    settings: Settings,
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
                        .send(Message::UpdateCursor(CursorUpdateInfo::from_ui(
                            &ui,
                            self.fonts.read().cell_size().cast_as(),
                        )))
                        .unwrap();
                    self.render_tx
                        .send(Message::UpdateDrawOrder(ui.draw_order.clone()))
                        .unwrap();
                    for grid in ui.grids.iter() {
                        for grid in ui.deleted_grids.iter() {
                            self.render_tx.send(Message::DeleteGrid(*grid)).unwrap();
                        }
                        // TODO: Split grid and window updates
                        if grid.dirty.contents() || grid.dirty.window() {
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
                let size = FontSize::Height(height);
                self.fonts.write().set_font_size(size);
                self.proxy.send_event(UserEvent::ResizeGrid).unwrap();
            }

            "neophyte.set_font_width" => {
                let mut args = Values::new(params.into_iter().next().unwrap()).unwrap();
                let width: f32 = args.next().unwrap();
                let size = FontSize::Width(width);
                self.fonts.write().set_font_size(size);
                self.proxy.send_event(UserEvent::ResizeGrid).unwrap();
            }

            "neophyte.set_cursor_speed" => {
                let mut args = Values::new(params.into_iter().next().unwrap()).unwrap();
                let speed: f32 = args.next().unwrap();
                self.settings.cursor_speed = speed;
                self.render_tx
                    .send(Message::UpdateSettings(self.settings))
                    .unwrap();
            }

            "neophyte.set_scroll_speed" => {
                let mut args = Values::new(params.into_iter().next().unwrap()).unwrap();
                let speed: f32 = args.next().unwrap();
                self.settings.scroll_speed = speed;
                self.render_tx
                    .send(Message::UpdateSettings(self.settings))
                    .unwrap();
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
                self.settings.underline_offset = offset;
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
                let cursor_speed = self.settings.cursor_speed;
                self.neovim
                    .send_response(rpc::Response::result(msgid, cursor_speed.into()));
            }

            "neophyte.get_scroll_speed" => {
                let scroll_speed = self.settings.scroll_speed;
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
                let offset = self.settings.underline_offset;
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
