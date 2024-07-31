mod buttons;
pub mod settings;

use self::{buttons::Buttons, settings::Settings};
use crate::{
    event::{self, rgb::Rgb},
    neovim::{action::Action, button::Button, Neovim},
    rendering::{state::RenderState, Motion},
    rpc::{self, Notification},
    text::{font::Metrics, fonts::FontSetting},
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
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::{
        DeviceEvent, DeviceId, ElementState, KeyEvent, MouseButton, MouseScrollDelta, StartCause,
        TouchPhase, WindowEvent,
    },
    event_loop::{ActiveEventLoop, ControlFlow},
    keyboard::{Key, ModifiersState, NamedKey},
    window::{Fullscreen, Window, WindowId},
};

pub struct EventHandler {
    window: Option<Arc<Window>>,
    render_state: Option<RenderState>,
    ui: Ui,
    settings: Settings,
    mouse: Mouse,
    modifiers: ModifiersState,
    neovim: Neovim,
    frame_number: u32,
    last_render_time: Option<Instant>,
}

impl ApplicationHandler<UserEvent> for EventHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let Some(window) = event_loop
            .create_window(
                Window::default_attributes()
                    .with_transparent(self.settings.transparent)
                    .with_title("Neophyte"),
            )
            .ok()
            .map(Arc::new)
        else {
            return;
        };

        self.render_state = Some(RenderState::new(window.clone(), self.settings.transparent));
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::ModifiersChanged(new_modifiers) => {
                self.modifiers = new_modifiers.state();
            }
            WindowEvent::KeyboardInput { event, .. } => self.keyboard_input(&event),
            WindowEvent::CursorMoved { position, .. } => self.cursor_moved(position),
            WindowEvent::MouseInput { state, button, .. } => self.mouse_input(state, button),
            WindowEvent::MouseWheel { delta, phase, .. } => self.mouse_wheel(delta, phase),
            WindowEvent::Resized(physical_size) => self.resized(physical_size),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => self.rescale(scale_factor),
            WindowEvent::CloseRequested => {
                log::info!("Close requested");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                log::debug!("Winit requested redraw");
                self.redraw(event_loop);
            }
            WindowEvent::Focused(focus) => self.neovim.ui_set_focus(focus),
            _ => {}
        }
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        match cause {
            // May need to redraw as a result of blinking cursor timeout
            StartCause::ResumeTimeReached { .. } => self.window().request_redraw(),
            StartCause::WaitCancelled { .. } | StartCause::Poll | StartCause::Init => {}
        }
        log::debug!("New Winit events: {cause:?}");
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::Shutdown => {
                log::info!("Shutting down");
                event_loop.exit();
            }
            UserEvent::Request(request) => self.request(request),
            UserEvent::Notification(notification) => self.notification(notification, event_loop),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {}

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        _event: DeviceEvent,
    ) {
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {}

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {}

    fn memory_warning(&mut self, _event_loop: &ActiveEventLoop) {}
}

impl EventHandler {
    pub fn new(neovim: Neovim, transparent: bool) -> Self {
        Self {
            window: None,
            render_state: None,
            frame_number: 0,
            ui: Ui::new(),
            settings: Settings::new(transparent),
            mouse: Mouse::new(),
            modifiers: ModifiersState::default(),
            neovim,
            last_render_time: None,
        }
    }

    fn window(&self) -> &Window {
        self.window.as_ref().unwrap()
    }

    fn notification(&mut self, notification: Notification, event_loop: &ActiveEventLoop) {
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
                    let size = FontSize::Height(height * self.window().scale_factor() as f32);
                    self.set_font_size(size);
                    self.finish_font_change();
                }

                "neophyte.set_font_width" => {
                    let mut args = Values::new(params.into_iter().next()?)?;
                    let width: f32 = args.next()?;
                    let size = FontSize::Width(width * self.window().scale_factor() as f32);
                    self.set_font_size(size);
                    self.finish_font_change();
                }

                "neophyte.set_cursor_speed" => {
                    let mut args = Values::new(params.into_iter().next()?)?;
                    let speed: f32 = args.next()?;
                    self.settings.cursor_speed = speed;
                    self.window().request_redraw();
                }

                "neophyte.set_scroll_speed" => {
                    let mut args = Values::new(params.into_iter().next()?)?;
                    let speed: f32 = args.next()?;
                    self.settings.scroll_speed = speed;
                    self.window().request_redraw();
                }

                "neophyte.set_fonts" => {
                    let args = Values::new(params.into_iter().next()?)?;
                    let font_settings = args.map()?;
                    let em = self.metrics().em;
                    self.set_fonts(font_settings, FontSize::Height(em));
                    self.finish_font_change();
                }

                "neophyte.set_underline_offset" => {
                    let mut args = Values::new(params.into_iter().next()?)?;
                    let offset: f32 = args.next()?;
                    let offset: i32 = offset as i32;
                    self.settings.underline_offset = offset;
                    self.window().request_redraw();
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

                "neophyte.end_render" => self.settings.render_target = None,

                "neophyte.set_bg_override" => {
                    let mut args = Values::new(params.into_iter().next()?)?;
                    let r = args.next()?;
                    let g = args.next()?;
                    let b = args.next()?;
                    let a: u8 = args.next()?;
                    let rgba = Rgb::new(r, g, b).into_srgb(a as f32 / 255.);
                    self.settings.bg_override = Some(rgba);
                }

                "neophyte.set_fullscreen" => {
                    let mut args = Values::new(params.into_iter().next()?)?;
                    let is_fullscreen: bool = args.next()?;
                    let fullscreen = is_fullscreen.then_some(Fullscreen::Borderless(None));
                    self.window().set_fullscreen(fullscreen);
                }

                "neophyte.leave" => event_loop.exit(),
                "neophyte.buf_leave" => self.ui.ignore_next_scroll = true,
                "neophyte.enable_raw_input" => self.settings.raw_input = true,
                "neophyte.disable_raw_input" => self.settings.raw_input = false,
                "neophyte.enable_frame_events" => {
                    self.settings.send_frame_events = true;
                    self.frame_number = 0;
                }
                "neophyte.disable_frame_events" => self.settings.send_frame_events = false,

                _ => log::error!("Unrecognized notification: {method}"),
            }
            Some(())
        };
        let _ = inner();
    }

    fn handle_redraw_notification(&mut self, params: Vec<Value>) {
        log::debug!("Neovim redraw start");
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
                self.set_fonts(
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

            self.render_state
                .as_mut()
                .unwrap()
                .update(&self.ui, bg_override);
            self.ui.clear_dirty();
            self.window().request_redraw();
        }
        log::debug!("Neovim redraw end");
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
                    .render_state
                    .as_mut()
                    .unwrap()
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
                let width = self.metrics().width / self.window().scale_factor() as f32;
                self.neovim
                    .send_response(rpc::Response::result(msgid, width.into()));
            }

            "neophyte.get_font_height" => {
                let width = self.metrics().em / self.window().scale_factor() as f32;
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

            "neophyte.get_fullscreen" => {
                let is_fullscreen = self.window().fullscreen().is_some();
                self.neovim
                    .send_response(rpc::Response::result(msgid, is_fullscreen.into()));
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
        let cell_size = self.cell_size();
        let surface_size = self.render_state.as_ref().unwrap().surface_size();
        let inner = (surface_size.into_cells(cell_size)).into_pixels(cell_size);
        let margin = (surface_size - inner) / 2;
        let position = position - margin.cast();
        let Ok(position) = position.try_cast::<u32>() else {
            return;
        };
        self.mouse.position = position;
        if let Some(grid) = self.ui.grid_under_cursor(position, self.cell_size().cast()) {
            let position = position.into_cells(cell_size);
            self.neovim.input_mouse(
                self.mouse.buttons.first().unwrap_or(Button::Move),
                // Irrelevant for move
                Action::ButtonDrag,
                self.modifiers.into(),
                grid.grid,
                position.0.y,
                position.0.x,
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
            .grid_under_cursor(self.mouse.position, self.cell_size().cast())
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
                let cell_size: Vec2<i32> = self.cell_size().try_cast().unwrap();
                let lines = self.mouse.scroll / cell_size;
                self.mouse.scroll -= lines * cell_size;
                lines
            }
        };

        let Some(grid) = self
            .ui
            .grid_under_cursor(self.mouse.position, self.cell_size().cast())
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
        self.resize();
    }

    fn rescale(&mut self, new_scale_factor: f64) {
        log::info!("Got rescale: {new_scale_factor}");
        let new_font_size = FontSize::Height(self.metrics().em * new_scale_factor as f32);
        self.set_font_size(new_font_size);
    }

    fn redraw(&mut self, event_loop: &ActiveEventLoop) {
        let elapsed = self
            .last_render_time
            .map(|last_render_time| {
                let now = Instant::now();
                let elapsed = now.duration_since(last_render_time);
                self.last_render_time = Some(now);
                log::debug!("Got winit redraw: {elapsed:?}");
                elapsed
            })
            .unwrap_or_else(|| {
                self.window()
                    .current_monitor()
                    // TODO: Use refresh rate of of VideoModeHandle for exclusive fullscreen
                    .and_then(|monitor| monitor.refresh_rate_millihertz())
                    .map(|mhz| Duration::from_secs_f64(1000.0 / mhz as f64))
                    .unwrap_or(Duration::from_millis(16))
            });

        let cell_size = self.cell_size();
        let render_state = self.render_state.as_mut().unwrap();
        let motion = render_state.advance(elapsed, cell_size.cast_as(), &self.settings);
        render_state.render(
            cell_size,
            &self.settings,
            self.window.as_ref().unwrap(),
            self.frame_number,
        );

        if self.settings.send_frame_events {
            self.neovim.exec_lua(
                "require('neophyte').receive_frame_event(...)".to_string(),
                vec![self.frame_number.into()],
            )
        }
        self.frame_number = self.frame_number.saturating_add(1);

        match motion {
            Motion::Still => self.last_render_time = None,
            Motion::Animating => {
                self.window().request_redraw();
                if self.last_render_time.is_none() {
                    self.last_render_time = Some(Instant::now());
                }
            }
            Motion::Delay(duration) => {
                event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now() + duration));
                self.last_render_time = None;
            }
        }
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

        if self.settings.raw_input {
            self.neovim.exec_lua(
                "require('neophyte').receive_raw_input(...)".to_string(),
                vec![c.clone().into()],
            );
        }

        self.neovim.input(c);
    }

    fn set_font_size(&mut self, size: FontSize) {
        self.render_state
            .as_mut()
            .unwrap()
            .fonts
            .set_font_size(size);
    }

    fn metrics(&self) -> Metrics {
        self.render_state.as_ref().unwrap().fonts.metrics()
    }

    fn set_fonts(&mut self, fonts: Vec<FontSetting>, size: FontSize) {
        self.render_state
            .as_mut()
            .unwrap()
            .fonts
            .set_fonts(fonts, size)
    }

    fn cell_size(&self) -> Vec2<u32> {
        self.render_state.as_ref().unwrap().fonts.cell_size()
    }

    fn finish_font_change(&mut self) {
        self.render_state.as_mut().unwrap().clear_glyph_cache();
        self.resize();
    }

    fn resize(&mut self) {
        let render_size = self.render_size();
        self.resize_neovim_grid();
        let cell_size = self.cell_size();
        self.render_state.as_mut().unwrap().resize(
            render_size,
            cell_size,
            self.settings.transparent,
        );
    }

    fn resize_neovim_grid(&mut self) {
        let size = self.render_size().into_cells(self.cell_size());
        self.neovim.ui_try_resize_grid(1, size.0.x, size.0.y);
    }

    fn render_size(&mut self) -> PixelVec<u32> {
        if let Some(size) = self.settings.render_size {
            size
        } else {
            self.window.as_ref().unwrap().inner_size().into()
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
