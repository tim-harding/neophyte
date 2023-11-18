mod event;
mod neovim;
mod rendering;
mod rpc;
pub mod text;
mod ui;
mod util;

use crate::{
    neovim::{action::Action, button::Button},
    rendering::Motion,
    text::fonts::FontSetting,
};
use neovim::{stdout_thread::StdoutHandler, Neovim};
use rendering::state::RenderState;
use rmpv::Value;
use rpc::Notification;
use std::thread;
use text::fonts::Fonts;
use ui::{
    options::{FontSize, GuiFont},
    Ui,
};
use util::{vec2::Vec2, Values};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{
        ElementState, Event, KeyEvent, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent,
    },
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy, EventLoopWindowTarget},
    keyboard::{Key, ModifiersState, NamedKey},
    window::{Window, WindowBuilder},
};

// TODO: Maybe rearranging the drop order will improve close time?

fn main() {
    env_logger::builder().format_timestamp(None).init();

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event()
        .build()
        .unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let (neovim, stdout_handler, stdin_handler) = Neovim::new().unwrap();
    neovim.ui_attach();
    let stdin_thread = std::thread::spawn(move || stdin_handler.start());
    let handler = NeovimHandler::new(event_loop.create_proxy());
    let stdout_thread = thread::spawn(move || {
        stdout_handler.start(handler);
    });

    let mut handler = EventHandler::new(neovim, window);
    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop
        .run(move |event, window_target| {
            handler.handle(event, window_target);
        })
        .expect("Failed to start render loop");

    stdout_thread
        .join()
        .expect("Failed to join Neovim stdout thread");
    stdin_thread
        .join()
        .expect("Failed to join Neovim stdin thread");
}

struct EventHandler {
    scale_factor: f32,
    surface_size: Vec2<u32>,
    ui: Ui,
    settings: Settings,
    mouse: Mouse,
    modifiers: ModifiersState,
    fonts: Fonts,
    neovim: Neovim,
    render_state: RenderState,
    window: Window,
}

impl EventHandler {
    pub fn new(neovim: Neovim, window: Window) -> Self {
        let fonts = Fonts::new();
        let render_state = pollster::block_on(async {
            let cell_size = fonts.cell_size();
            RenderState::new(&window, cell_size).await
        });
        Self {
            scale_factor: 1.,
            surface_size: render_state.surface_size(),
            ui: Ui::new(),
            settings: Settings::new(),
            mouse: Mouse::new(),
            modifiers: ModifiersState::default(),
            fonts,
            neovim,
            render_state,
            window,
        }
    }

    pub fn handle(
        &mut self,
        event: Event<UserEvent>,
        window_target: &EventLoopWindowTarget<UserEvent>,
    ) {
        match event {
            Event::UserEvent(user_event) => match user_event {
                UserEvent::Shutdown => window_target.exit(),
                UserEvent::Request(request) => self.request(request),
                UserEvent::Notification(notification) => self.notification(notification),
            },

            Event::WindowEvent {
                window_id,
                ref event,
            } if window_id == self.window.id() => match event {
                WindowEvent::ModifiersChanged(new_modifiers) => {
                    self.modifiers = new_modifiers.state();
                }
                WindowEvent::KeyboardInput { event, .. } => self.keyboard_input(event),
                WindowEvent::CursorMoved { position, .. } => self.cursor_moved(*position),
                WindowEvent::MouseInput { state, button, .. } => self.mouse_input(*state, *button),
                WindowEvent::MouseWheel { delta, phase, .. } => self.mouse_wheel(*delta, *phase),
                WindowEvent::Resized(physical_size) => self.resize(*physical_size),
                WindowEvent::ScaleFactorChanged { scale_factor, .. } => self.rescale(*scale_factor),
                WindowEvent::CloseRequested => window_target.exit(),
                WindowEvent::RedrawRequested => self.redraw(),
                _ => {}
            },

            _ => {}
        }
    }

    fn notification(&mut self, notification: Notification) {
        let Notification { method, params } = notification;
        match method.as_str() {
            "redraw" => self.handle_redraw_notification(params),

            "neophyte.set_font_height" => {
                let mut args = Values::new(params.into_iter().next().unwrap()).unwrap();
                let height: f32 = args.next().unwrap();
                let size = FontSize::Height(height * self.scale_factor);
                self.fonts.set_font_size(size);
                self.render_state
                    .resize(self.surface_size, self.fonts.cell_size());
                resize_neovim_grid(self.surface_size, &self.fonts, &self.neovim);
            }

            "neophyte.set_font_width" => {
                let mut args = Values::new(params.into_iter().next().unwrap()).unwrap();
                let width: f32 = args.next().unwrap();
                let size = FontSize::Width(width * self.scale_factor);
                self.fonts.set_font_size(size);
                self.render_state
                    .resize(self.surface_size, self.fonts.cell_size());
                resize_neovim_grid(self.surface_size, &self.fonts, &self.neovim);
            }

            "neophyte.set_cursor_speed" => {
                let mut args = Values::new(params.into_iter().next().unwrap()).unwrap();
                let speed: f32 = args.next().unwrap();
                self.settings.cursor_speed = speed;
                self.window.request_redraw();
            }

            "neophyte.set_scroll_speed" => {
                let mut args = Values::new(params.into_iter().next().unwrap()).unwrap();
                let speed: f32 = args.next().unwrap();
                self.settings.scroll_speed = speed;
                self.window.request_redraw();
            }

            "neophyte.set_fonts" => {
                let args = Values::new(params.into_iter().next().unwrap()).unwrap();
                let font_settings = args.map().unwrap();
                let em = self.fonts.metrics().em;
                self.fonts.set_fonts(font_settings, FontSize::Height(em));
                self.render_state
                    .resize(self.surface_size, self.fonts.cell_size());
                resize_neovim_grid(self.surface_size, &self.fonts, &self.neovim);
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

    fn handle_redraw_notification(&mut self, params: Vec<Value>) {
        for param in params {
            match event::Event::try_parse(param.clone()) {
                Ok(events) => {
                    for event in events.iter().cloned() {
                        self.ui.process(event);
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

        if self.ui.did_flush {
            if let Some(guifont_update) = self.ui.guifont_update.take() {
                let GuiFont { fonts, size } = guifont_update;
                self.fonts.set_fonts(
                    fonts
                        .into_iter()
                        .map(|name| FontSetting::with_name(name))
                        .collect(),
                    size,
                );
                self.render_state.clear_glyph_cache();
                self.render_state
                    .resize(self.surface_size, self.fonts.cell_size());
                resize_neovim_grid(self.surface_size, &self.fonts, &self.neovim);
            }
            self.render_state.update(&self.ui, &self.fonts);
            self.ui.clear_dirty();
            self.window.request_redraw();
        }
    }

    fn request(&mut self, request: rpc::Request) {
        let rpc::Request {
            msgid,
            method,
            params,
        } = request;
        match method.as_str() {
            "neophyte.get_fonts" => {
                let names = self
                    .fonts
                    .iter()
                    .map(|font| font.setting.name.clone())
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
                let width = self.fonts.metrics().width;
                self.neovim
                    .send_response(rpc::Response::result(msgid, width.into()));
            }

            "neophyte.get_font_height" => {
                let width = self.fonts.metrics().em;
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

    fn keyboard_input(&mut self, event: &KeyEvent) {
        match event.state {
            ElementState::Pressed => {}
            ElementState::Released => return,
        }
        match &event.logical_key {
            Key::Named(key) => {
                let c = || {
                    use NamedKey::*;
                    Some(match key {
                        Enter => "Enter",
                        Tab => "Tab",
                        Space => "Space",
                        ArrowDown => "Down",
                        ArrowLeft => "Left",
                        ArrowRight => "Right",
                        ArrowUp => "Up",
                        End => "End",
                        Home => "Home",
                        PageDown => "PageDown",
                        PageUp => "PageUp",
                        Backspace => "BS",
                        Delete => "Del",
                        Escape => "Esc",
                        F1 => "F1",
                        F2 => "F2",
                        F3 => "F3",
                        F4 => "F4",
                        F5 => "F5",
                        F6 => "F6",
                        F7 => "F7",
                        F8 => "F8",
                        F9 => "F9",
                        F10 => "F10",
                        F11 => "F11",
                        F12 => "F12",
                        F13 => "F13",
                        F14 => "F14",
                        F15 => "F15",
                        F16 => "F16",
                        F17 => "F17",
                        F18 => "F18",
                        F19 => "F19",
                        F20 => "F20",
                        F21 => "F21",
                        F22 => "F22",
                        F23 => "F23",
                        F24 => "F24",
                        F25 => "F25",
                        F26 => "F26",
                        F27 => "F27",
                        F28 => "F28",
                        F29 => "F29",
                        F30 => "F30",
                        F31 => "F31",
                        F32 => "F32",
                        F33 => "F33",
                        F34 => "F34",
                        F35 => "F35",
                        _ => return None,
                    })
                };

                if let Some(c) = c() {
                    send_keys(c, &mut self.modifiers, &self.neovim, false);
                }
            }

            Key::Character(c) => {
                let s = match c.as_str() {
                    "<" => "Lt",
                    "\\" => "Bslash",
                    "|" => "Bar",
                    _ => c.as_str(),
                };
                send_keys(s, &mut self.modifiers, &self.neovim, true);
            }

            Key::Unidentified(_) | Key::Dead(_) => {}
        }
    }

    fn cursor_moved(&mut self, position: PhysicalPosition<f64>) {
        let position: Vec2<f64> = position.into();
        let position: Vec2<i64> = position.cast_as();
        let cell_size = self.fonts.cell_size();
        let inner = (self.surface_size / cell_size) * cell_size;
        let margin = (self.surface_size - inner) / 2;
        let position = position - margin.cast();
        let Ok(position) = position.try_cast::<u64>() else {
            return;
        };
        self.mouse.position = position;
        if let Some(grid) = self
            .ui
            .grid_under_cursor(position, self.fonts.cell_size().cast())
        {
            self.neovim.input_mouse(
                self.mouse.buttons.first().unwrap_or(Button::Move),
                // Irrelevant for move
                Action::ButtonDrag,
                self.modifiers.into(),
                grid.grid,
                grid.position.y,
                grid.position.x,
            );
        }
    }

    fn mouse_input(&mut self, state: ElementState, button: MouseButton) {
        let Ok(button) = button.try_into() else {
            return;
        };

        let action = state.into();
        let depressed = match action {
            Action::ButtonPress => true,
            Action::ButtonRelease => false,
            _ => unreachable!(),
        };
        match button {
            Button::Left => self.mouse.buttons = self.mouse.buttons.with_left(depressed),
            Button::Right => self.mouse.buttons = self.mouse.buttons.with_right(depressed),
            Button::Middle => self.mouse.buttons = self.mouse.buttons.with_middle(depressed),
            _ => unreachable!(),
        }
        if let Some(grid) = self
            .ui
            .grid_under_cursor(self.mouse.position, self.fonts.cell_size().cast())
        {
            self.neovim.input_mouse(
                button,
                action,
                self.modifiers.into(),
                grid.grid,
                grid.position.y,
                grid.position.x,
            );
        }
    }

    fn mouse_wheel(&mut self, delta: MouseScrollDelta, phase: TouchPhase) {
        let reset = matches!(
            phase,
            TouchPhase::Started | TouchPhase::Ended | TouchPhase::Cancelled
        );

        let (delta, kind) = match delta {
            MouseScrollDelta::LineDelta(horizontal, vertical) => {
                (Vec2::new(horizontal, vertical).cast(), ScrollKind::Lines)
            }

            MouseScrollDelta::PixelDelta(delta) => (delta.into(), ScrollKind::Pixels),
        };

        let modifiers = self.modifiers.into();

        let delta: Vec2<i64> = delta.cast_as();
        if reset {
            self.mouse.scroll = Vec2::default();
        }

        let lines = match kind {
            ScrollKind::Lines => delta,
            ScrollKind::Pixels => {
                self.mouse.scroll += delta;
                let cell_size: Vec2<i64> = self.fonts.cell_size().cast();
                let lines = self.mouse.scroll / cell_size;
                self.mouse.scroll -= lines * cell_size;
                lines
            }
        };

        let Some(grid) = self
            .ui
            .grid_under_cursor(self.mouse.position, self.fonts.cell_size().cast())
        else {
            return;
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

    fn resize(&mut self, physical_size: PhysicalSize<u32>) {
        self.surface_size = physical_size.into();
        resize_neovim_grid(self.surface_size, &self.fonts, &self.neovim);
        self.render_state
            .resize(self.surface_size, self.fonts.cell_size());
    }

    fn rescale(&mut self, new_scale_factor: f64) {
        self.scale_factor = new_scale_factor as f32;
        let new_font_size = FontSize::Height(self.fonts.metrics().em * self.scale_factor);
        self.fonts.set_font_size(new_font_size);
    }

    fn redraw(&mut self) {
        let framerate = self
            .window
            .current_monitor()
            .and_then(|monitor| monitor.refresh_rate_millihertz())
            .unwrap_or(60_000);
        let delta_seconds = 1_000. / framerate as f32;
        let motion = self.render_state.render(
            self.fonts.cell_size(),
            delta_seconds,
            self.settings,
            &self.window,
        );
        match motion {
            Motion::Still => {}
            Motion::Animating => self.window.request_redraw(),
        }
        log::info!("Rendered with result {motion:?}");
    }
}

fn send_keys(c: &str, modifiers: &mut ModifiersState, neovim: &Neovim, ignore_shift: bool) {
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
    Notification(rpc::Notification),
    Request(rpc::Request),
    Shutdown,
}

struct NeovimHandler {
    proxy: EventLoopProxy<UserEvent>,
}

impl NeovimHandler {
    pub fn new(proxy: EventLoopProxy<UserEvent>) -> Self {
        Self { proxy }
    }
}

// Explicitly ignoring errors here because if we close the app through Neophyte
// instead of Neovim, the main thread will have already dropped the event loop.
impl StdoutHandler for NeovimHandler {
    fn handle_notification(&mut self, notification: rpc::Notification) {
        let _ = self.proxy.send_event(UserEvent::Notification(notification));
    }

    fn handle_request(&mut self, request: rpc::Request) {
        let _ = self.proxy.send_event(UserEvent::Request(request));
    }

    fn handle_shutdown(&mut self) {
        let _ = self.proxy.send_event(UserEvent::Shutdown);
    }
}
