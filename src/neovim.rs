use crate::rpc::{decode, encode, DecodeError, RpcMessage};
use bitfield_struct::bitfield;
use rmpv::Value;
use std::{
    io::{self, ErrorKind},
    process::{ChildStdout, Command, Stdio},
    sync::{mpsc, Arc, Mutex},
    thread,
};
use winit::event::{ElementState, ModifiersState, MouseButton};

#[derive(Debug, Clone)]
pub struct Neovim {
    stdin_tx: mpsc::Sender<RpcMessage>,
    msgid: Arc<Mutex<u64>>,
}

impl Neovim {
    pub fn new() -> io::Result<(Neovim, Handler)> {
        use io::Error;
        let mut child = Command::new("nvim")
            .arg("--embed")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| Error::new(ErrorKind::Other, "Can't open stdout"))?;
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| Error::new(ErrorKind::Other, "Can't open stdin"))?;

        let (tx, rx) = mpsc::channel();
        thread::spawn(move || loop {
            while let Ok(msg) = rx.recv() {
                encode(&mut stdin, msg).unwrap();
            }
        });

        Ok((
            Neovim {
                stdin_tx: tx.clone(),
                msgid: Default::default(),
            },
            Handler {
                stdin_tx: tx,
                stdout,
            },
        ))
    }

    pub fn call(&self, method: &str, args: Vec<Value>) -> u64 {
        let msgid = {
            let mut lock = self.msgid.lock().unwrap();
            let msgid = *lock;
            *lock += 1;
            msgid
        };

        let req = RpcMessage::Request {
            msgid,
            method: method.to_owned(),
            params: args,
        };

        match self.stdin_tx.send(req) {
            Ok(_) => {}
            Err(e) => {
                log::error!("{e}");
            }
        }

        msgid
    }

    // TODO: Proper public API
    pub fn ui_attach(&self) {
        let extensions = [
            "rgb",
            "ext_linegrid",
            "ext_multigrid",
            // "ext_popupmenu",
            // "ext_tabline",
            // "ext_cmdline",
            // "ext_wildmenu",
            // "ext_hlstate",
            // "ext_termcolors",
            // "ext_messages",
        ];
        let extensions = Value::Map(
            extensions
                .into_iter()
                .map(|arg| (arg.into(), true.into()))
                .collect(),
        );
        let attach_args = vec![80u64.into(), 10u64.into(), extensions];
        self.call("nvim_ui_attach", attach_args);
    }

    pub fn input(&self, input: String) {
        let args = vec![input.into()];
        self.call("nvim_input", args);
    }

    pub fn input_mouse(
        &self,
        button: Button,
        action: Action,
        modifiers: Modifiers,
        grid: u64,
        row: u64,
        col: u64,
    ) {
        let args = vec![
            button.into(),
            action.into(),
            modifiers.into(),
            grid.into(),
            row.into(),
            col.into(),
        ];
        self.call("nvim_input_mouse", args);
    }

    pub fn ui_try_resize_grid(&self, grid: u64, width: u64, height: u64) {
        let args: Vec<_> = [grid, width, height]
            .into_iter()
            .map(|n| n.into())
            .collect();
        self.call("nvim_ui_try_resize_grid", args);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Left,
    Right,
    Middle,
    Wheel,
    Move,
}

impl From<Button> for &str {
    fn from(button: Button) -> Self {
        match button {
            Button::Left => "left",
            Button::Right => "right",
            Button::Middle => "middle",
            Button::Wheel => "wheel",
            Button::Move => "move",
        }
    }
}

impl From<Button> for Value {
    fn from(button: Button) -> Self {
        let s: &str = button.into();
        s.to_string().into()
    }
}

impl TryFrom<MouseButton> for Button {
    type Error = ButtonFromWinitError;

    fn try_from(button: MouseButton) -> Result<Self, Self::Error> {
        match button {
            MouseButton::Left => Ok(Self::Left),
            MouseButton::Right => Ok(Self::Right),
            MouseButton::Middle => Ok(Self::Middle),
            MouseButton::Other(_) => Err(ButtonFromWinitError),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("No Neovim button for the given Winit mouse button")]
pub struct ButtonFromWinitError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    ButtonPress,
    ButtonDrag,
    ButtonRelease,
    WheelUp,
    WheelDown,
    WheelLeft,
    WheelRight,
}

impl From<Action> for &str {
    fn from(action: Action) -> Self {
        match action {
            Action::ButtonPress => "press",
            Action::ButtonDrag => "drag",
            Action::ButtonRelease => "release",
            Action::WheelUp => "up",
            Action::WheelDown => "down",
            Action::WheelLeft => "left",
            Action::WheelRight => "right",
        }
    }
}

impl From<Action> for Value {
    fn from(action: Action) -> Self {
        let s: &str = action.into();
        s.to_string().into()
    }
}

impl From<ElementState> for Action {
    fn from(state: ElementState) -> Self {
        match state {
            ElementState::Pressed => Self::ButtonPress,
            ElementState::Released => Self::ButtonRelease,
        }
    }
}

#[bitfield(u8)]
pub struct Modifiers {
    ctrl: bool,
    shift: bool,
    alt: bool,
    #[bits(5)]
    __: u8,
}

impl From<Modifiers> for String {
    fn from(mods: Modifiers) -> Self {
        let ctrl = if mods.ctrl() { "C" } else { "" };
        let shift = if mods.shift() { "S" } else { "" };
        let alt = if mods.alt() { "A" } else { "" };
        format!("{ctrl}{shift}{alt}")
    }
}

impl From<Modifiers> for Value {
    fn from(modifiers: Modifiers) -> Self {
        let s: String = modifiers.into();
        s.into()
    }
}

impl From<ModifiersState> for Modifiers {
    fn from(state: ModifiersState) -> Self {
        Self::new()
            .with_ctrl(state.ctrl())
            .with_shift(state.shift())
            .with_alt(state.alt())
    }
}

pub struct Handler {
    stdin_tx: mpsc::Sender<RpcMessage>,
    stdout: ChildStdout,
}

impl Handler {
    pub fn start<F, S>(mut self, mut notification_handler: F, shutdown_handler: S)
    where
        F: FnMut(String, Vec<Value>),
        S: Fn(),
    {
        use rmpv::decode::Error;
        loop {
            let msg = match decode(&mut self.stdout) {
                Ok(msg) => msg,
                Err(e) => {
                    match e {
                        DecodeError::Rmpv(e) => {
                            let io_error = match &e {
                                Error::InvalidMarkerRead(e) => Some(e.kind()),
                                Error::InvalidDataRead(e) => Some(e.kind()),
                                Error::DepthLimitExceeded => None,
                            };
                            let Some(io_error) = io_error else {
                                log::error!("{e}");
                                continue;
                            };
                            match io_error {
                                ErrorKind::UnexpectedEof => shutdown_handler(),
                                _ => log::error!("{e}"),
                            }
                        }
                        DecodeError::Parse => log::error!("Failed to parse an RPC message"),
                    }
                    return;
                }
            };

            match msg {
                RpcMessage::Request {
                    msgid,
                    method,
                    params,
                } => {
                    log::info!("RPC Request: {method}, {params:?}");
                    let response = RpcMessage::Response {
                        msgid,
                        result: Value::Nil,
                        error: "Not handled".into(),
                    };
                    match self.stdin_tx.send(response) {
                        Ok(_) => {}
                        Err(e) => {
                            log::error!("{e}");
                            return;
                        }
                    }
                }

                RpcMessage::Response {
                    msgid,
                    result,
                    error,
                } => {
                    if error != Value::Nil {
                        log::error!("RPC response to {msgid}: {error:?}");
                    } else {
                        log::info!("RPC response to {msgid}: {result:?}");
                    };
                }

                RpcMessage::Notification { method, params } => notification_handler(method, params),
            };
        }
    }
}
