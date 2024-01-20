mod buttons;
pub mod settings;

use self::{buttons::Buttons, settings::Settings};
use crate::{
    event::{self, rgb::Rgb},
    neovim::{action::Action, button::Button, Neovim},
    rendering::state::RenderState,
    rpc::{self, Notification},
    text::fonts::{FontSetting, Fonts},
    ui::{
        options::{FontSize, GuiFont},
        Ui,
    },
    util::{
        vec2::{PixelVec, Vec2},
        Values,
    },
    UserEvent,
};
use rmpv::Value;
use std::time::Instant;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{
        ElementState, Event, KeyEvent, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent,
    },
    event_loop::EventLoopWindowTarget,
    keyboard::{Key, ModifiersState, NamedKey},
    window::Window,
};

pub struct EventHandler {
    scale_factor: f32,
    surface_size: PixelVec<u32>,
    ui: Ui,
    settings: Settings,
    mouse: Mouse,
    modifiers: ModifiersState,
    fonts: Fonts,
    neovim: Neovim,
    render_state: RenderState,
    windows: Vec<Window>,
    frame_number: u32,
    last_render_time: Instant,
}

impl EventHandler {
    pub fn new(neovim: Neovim, window: Window, transparent: bool) -> Self {
        let fonts = Fonts::new();
        let render_state = pollster::block_on(async {
            let cell_size = fonts.cell_size();
            RenderState::new(&window, cell_size, transparent).await
        });
        Self {
            windows: vec![window],
            scale_factor: 1.,
            frame_number: 0,
            surface_size: render_state.surface_size(),
            ui: Ui::new(),
            settings: Settings::new(transparent),
            mouse: Mouse::new(),
            modifiers: ModifiersState::default(),
            fonts,
            neovim,
            render_state,
            last_render_time: Instant::now(),
        }
    }

    pub fn handle(
        &mut self,
        event: Event<UserEvent>,
        window_target: &EventLoopWindowTarget<UserEvent>,
    ) {
        match event {
            Event::UserEvent(user_event) => match user_event {
                UserEvent::Shutdown => {
                    log::info!("Shutting down");
                    window_target.exit();
                }
                UserEvent::Request(request) => self.request(request),
                UserEvent::Notification(notification) => {
                    self.notification(notification, window_target)
                }
            },

            Event::NewEvents(_) => log::info!("New Winit events"),
            Event::AboutToWait => self.request_redraw(),

            Event::WindowEvent {
                window_id: _,
                ref event,
            } => match event {
                WindowEvent::ModifiersChanged(new_modifiers) => {
                    self.modifiers = new_modifiers.state();
                }
                WindowEvent::KeyboardInput { event, .. } => self.keyboard_input(event),
                WindowEvent::CursorMoved { position, .. } => self.cursor_moved(*position),
                WindowEvent::MouseInput { state, button, .. } => self.mouse_input(*state, *button),
                WindowEvent::MouseWheel { delta, phase, .. } => self.mouse_wheel(*delta, *phase),
                WindowEvent::Resized(physical_size) => self.resized(*physical_size),
                WindowEvent::ScaleFactorChanged { scale_factor, .. } => self.rescale(*scale_factor),
                WindowEvent::CloseRequested => {
                    log::info!("Close requested");
                    window_target.exit();
                }
                WindowEvent::RedrawRequested => {
                    log::info!("Winit requested redraw");
                    self.redraw();
                }
                _ => {}
            },

            _ => {}
        }
    }

    fn notification(
        &mut self,
        notification: Notification,
        window_target: &EventLoopWindowTarget<UserEvent>,
    ) {
        let inner = || {
            let Notification { method, params } = notification;
            if method.as_str() != "redraw" {
                log::info!("Got notification {method} with {params:?}");
            }
            match method.as_str() {
                "redraw" => self.handle_redraw_notification(params),

                "neophyte.set_font_height" => {
                    let mut args = Values::new(params.into_iter().next()?)?;
                    let height: f32 = args.next()?;
                    let size = FontSize::Height(height * self.scale_factor);
                    self.fonts.set_font_size(size);
                    self.finish_font_change();
                }

                "neophyte.set_font_width" => {
                    let mut args = Values::new(params.into_iter().next()?)?;
                    let width: f32 = args.next()?;
                    let size = FontSize::Width(width * self.scale_factor);
                    self.fonts.set_font_size(size);
                    self.finish_font_change();
                }

                "neophyte.set_cursor_speed" => {
                    let mut args = Values::new(params.into_iter().next()?)?;
                    let speed: f32 = args.next()?;
                    self.settings.cursor_speed = speed;
                    self.request_redraw();
                }

                "neophyte.set_scroll_speed" => {
                    let mut args = Values::new(params.into_iter().next()?)?;
                    let speed: f32 = args.next()?;
                    self.settings.scroll_speed = speed;
                    self.request_redraw();
                }

                "neophyte.set_fonts" => {
                    let args = Values::new(params.into_iter().next()?)?;
                    let font_settings = args.map()?;
                    let em = self.fonts.metrics().em;
                    self.fonts.set_fonts(font_settings, FontSize::Height(em));
                    self.finish_font_change();
                }

                "neophyte.set_underline_offset" => {
                    let mut args = Values::new(params.into_iter().next()?)?;
                    let offset: f32 = args.next()?;
                    let offset: i32 = offset as i32;
                    self.settings.underline_offset = offset;
                    self.request_redraw();
                }

                "neophyte.set_render_size" => {
                    let mut args = Values::new(params.into_iter().next()?)?;
                    let width = args.next()?;
                    let height = args.next()?;
                    self.settings.render_size = Some(PixelVec::new(width, height));
                    self.resize();
                }

                "neophyte.unset_render_size" => {
                    self.settings.render_size = None;
                    self.resize();
                }

                "neophyte.start_render" => {
                    let mut args = Values::new(params.into_iter().next()?)?;
                    let path: String = args.next()?;
                    self.settings.render_target = Some(path.into());
                    self.frame_number = 0;
                }

                "neophyte.end_render" => {
                    self.settings.render_target = None;
                }

                "neophyte.set_bg_override" => {
                    let mut args = Values::new(params.into_iter().next()?)?;
                    let r = args.next()?;
                    let g = args.next()?;
                    let b = args.next()?;
                    let a: u8 = args.next()?;
                    let rgba = Rgb::new(r, g, b).into_srgb(a as f32 / 255.);
                    self.settings.bg_override = Some(rgba);
                }

                "neophyte.leave" => {
                    window_target.exit();
                }

                "neophyte.buf_leave" => {
                    self.ui.ignore_next_scroll = true;
                }

                _ => log::error!("Unrecognized notification: {method}"),
            }
            Some(())
        };
        let _ = inner();
    }

    fn handle_redraw_notification(&mut self, params: Vec<Value>) {
        log::info!("Neovim redraw start");
        for param in params {
            match event::Event::try_parse(param) {
                Ok(events) => {
                    for event in events.into_iter() {
                        log::debug!("{event:?}");
                        self.ui.process(event);
                    }
                }

                Err(e) => match e {
                    event::Error::UnknownEvent(name) => log::error!("Unknown event: {name}"),
                    _ => log::error!("{e}"),
                },
            }
        }

        if self.ui.did_flush {
            if let Some(guifont_update) = self.ui.guifont_update.take() {
                let GuiFont { fonts, size } = guifont_update;
                self.fonts.set_fonts(
                    fonts.into_iter().map(FontSetting::with_name).collect(),
                    size,
                );
                self.finish_font_change();
            }

            let bg_override = if self.settings.transparent {
                self.settings.bg_override
            } else {
                None
            };

            self.render_state.update(&self.ui, &self.fonts, bg_override);
            self.ui.clear_dirty();
            self.request_redraw();
        }
        log::info!("Neovim redraw end");
    }

    fn request(&mut self, request: rpc::Request) {
        let rpc::Request {
            msgid,
            method,
            params,
        } = request;
        log::info!("Got request {method} with {params:?}");
        match method.as_str() {
            "neophyte.is_running" => {
                self.neovim
                    .send_response(rpc::Response::result(msgid, true.into()));
            }

            "neophyte.get_fonts" => {
                let names = self
                    .fonts
                    .families()
                    .map(|family| family.setting.name.clone())
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
                let width = self.fonts.metrics().width / self.scale_factor;
                self.neovim
                    .send_response(rpc::Response::result(msgid, width.into()));
            }

            "neophyte.get_font_height" => {
                let width = self.fonts.metrics().em / self.scale_factor;
                self.neovim
                    .send_response(rpc::Response::result(msgid, width.into()));
            }

            "neophyte.get_underline_offset" => {
                let offset = self.settings.underline_offset;
                self.neovim
                    .send_response(rpc::Response::result(msgid, offset.into()));
            }

            "neophyte.get_render_size" => {
                let render_size = self.render_size();
                self.neovim.send_response(rpc::Response::result(
                    msgid,
                    Value::Map(vec![
                        ("width".into(), render_size.0.x.into()),
                        ("height".into(), render_size.0.y.into()),
                    ]),
                ));
            }

            _ => log::error!("Unknown request: {}, {:?}", method, params),
        }
    }

    fn keyboard_input(&mut self, event: &KeyEvent) {
        match event.state {
            ElementState::Pressed => {}
            ElementState::Released => return,
        }

        log::info!("Got keyboard input: {event:?}");
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
                    self.send_keys(c, false);
                }
            }

            Key::Character(c) => {
                let s = match c.as_str() {
                    "<" => "Lt",
                    "\\" => "Bslash",
                    "|" => "Bar",
                    _ => c.as_str(),
                };
                self.send_keys(s, true);
            }

            Key::Unidentified(_) | Key::Dead(_) => {}
        }
    }

    fn cursor_moved(&mut self, position: PhysicalPosition<f64>) {
        log::info!("Got cursor move: {position:?}");
        let position: PixelVec<f64> = position.into();
        let position = position.cast_as::<i64>();
        let cell_size = self.fonts.cell_size();
        let inner = (self.surface_size.into_cells(cell_size)).into_pixels(cell_size);
        let margin = (self.surface_size - inner) / 2;
        let position = position - margin.cast();
        let Ok(position) = position.try_cast::<u32>() else {
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
                grid.position.0.y,
                grid.position.0.x,
            );
        }
    }

    fn mouse_input(&mut self, state: ElementState, button: MouseButton) {
        log::info!("Got mouse input: {button:?}, {state:?}");
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
                grid.position.0.y,
                grid.position.0.x,
            );
        }
    }

    fn mouse_wheel(&mut self, delta: MouseScrollDelta, phase: TouchPhase) {
        log::info!("Got mouse wheel: {delta:?}, {phase:?}");
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

        let delta: Vec2<i32> = delta.cast_as();
        if reset {
            self.mouse.scroll = Vec2::default();
        }

        let lines = match kind {
            ScrollKind::Lines => delta,
            ScrollKind::Pixels => {
                self.mouse.scroll += delta;
                let cell_size: Vec2<i32> = self.fonts.cell_size().try_cast().unwrap();
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
                grid.position.0.y,
                grid.position.0.x,
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
                grid.position.0.y,
                grid.position.0.x,
            );
        }
    }

    fn resized(&mut self, physical_size: PhysicalSize<u32>) {
        log::info!("Got resize: {physical_size:?}");
        self.surface_size = physical_size.into();
        self.resize();
    }

    fn rescale(&mut self, new_scale_factor: f64) {
        log::info!("Got rescale: {new_scale_factor}");
        self.scale_factor = new_scale_factor as f32;
        let new_font_size = FontSize::Height(self.fonts.metrics().em * self.scale_factor);
        self.fonts.set_font_size(new_font_size);
    }

    fn redraw(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_render_time);
        self.last_render_time = now;
        log::info!("Got redraw: {elapsed:?}");

        self.render_state
            .advance(elapsed, self.fonts.cell_size().cast_as(), &self.settings);
        self.render_state.render(
            self.fonts.cell_size(),
            &self.settings,
            &self.windows[0], // TODO
            self.frame_number,
        );

        self.frame_number = self.frame_number.saturating_add(1);
    }

    fn send_keys(&mut self, c: &str, ignore_shift: bool) {
        let shift = self.modifiers.shift_key() && !ignore_shift;
        let ctrl = self.modifiers.control_key();
        let alt = self.modifiers.alt_key();
        let logo = self.modifiers.super_key();
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
        self.neovim.input(c);
    }

    fn finish_font_change(&mut self) {
        self.render_state.clear_glyph_cache();
        self.resize();
    }

    fn resize(&mut self) {
        let render_size = self.render_size();
        self.resize_neovim_grid();
        self.render_state.resize(
            render_size,
            self.fonts.cell_size(),
            self.settings.transparent,
        );
    }

    fn resize_neovim_grid(&mut self) {
        let size = self.render_size().into_cells(self.fonts.cell_size());
        self.neovim.ui_try_resize_grid(1, size.0.x, size.0.y);
    }

    fn render_size(&mut self) -> PixelVec<u32> {
        if let Some(size) = self.settings.render_size {
            size
        } else {
            self.surface_size
        }
    }

    fn request_redraw(&mut self) {
        for window in &self.windows {
            window.request_redraw();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Mouse {
    position: PixelVec<u32>,
    scroll: Vec2<i32>,
    buttons: Buttons,
}

impl Mouse {
    pub fn new() -> Self {
        Self::default()
    }
}

pub enum ScrollKind {
    Lines,
    Pixels,
}
