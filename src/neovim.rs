use crate::rpc::{self, decode, encode, DecodeError, Message, Request};
use bitfield_struct::bitfield;
use rmpv::Value;
use std::{
    collections::BinaryHeap,
    io::{self, ErrorKind},
    process::{ChildStdin, ChildStdout, Command, Stdio},
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex, RwLock,
    },
};
use winit::event::{ElementState, ModifiersState, MouseButton};

#[derive(Debug, Clone)]
pub struct Neovim {
    tx: mpsc::Sender<rpc::Message>,
    incoming: Arc<RwLock<Incoming>>,
    next_msgid: Arc<Mutex<u64>>,
}

impl Neovim {
    pub fn new() -> io::Result<(Neovim, StdoutHandler, StdinHandler)> {
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
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| Error::new(ErrorKind::Other, "Can't open stdin"))?;

        let (tx, rx) = mpsc::channel();
        let incoming = Arc::new(RwLock::new(Incoming::new()));
        Ok((
            Neovim {
                tx,
                incoming: incoming.clone(),
                next_msgid: Default::default(),
            },
            StdoutHandler { incoming, stdout },
            StdinHandler { rx, stdin },
        ))
    }

    pub fn send_response(&self, response: rpc::Response) {
        self.incoming
            .write()
            .unwrap()
            .push_response(response, &self.tx);
    }

    fn call(&self, method: &str, args: Vec<Value>) -> u64 {
        let msgid = {
            let mut lock = self.next_msgid.lock().unwrap();
            let msgid = *lock;
            *lock += 1;
            msgid
        };

        let req = Request {
            msgid,
            method: method.to_owned(),
            params: args,
        };

        match self.tx.send(req.into()) {
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

// NOTE: Responses must be given in reverse order of requests (like "unwinding a stack").

#[derive(Debug, Clone, Default)]
struct Incoming {
    requests: Vec<u64>,
    responses: BinaryHeap<QueuedResponse>,
}

impl Incoming {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_request(&mut self, msgid: u64) {
        self.requests.push(msgid);
    }

    pub fn push_response(&mut self, response: rpc::Response, tx: &mpsc::Sender<rpc::Message>) {
        self.responses.push(response.into());
        while let Some(ready) = self.next_ready() {
            tx.send(ready.into()).unwrap();
        }
    }

    fn next_ready(&mut self) -> Option<rpc::Response> {
        if let (Some(id), Some(response)) = (self.requests.last(), self.responses.peek()) {
            if *id == response.0.msgid {
                self.requests.pop();
                self.responses.pop().map(|response| response.into())
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
struct QueuedResponse(rpc::Response);

impl PartialEq for QueuedResponse {
    fn eq(&self, other: &Self) -> bool {
        self.0.msgid == other.0.msgid
    }
}

impl Eq for QueuedResponse {}

impl Ord for QueuedResponse {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.msgid.cmp(&other.0.msgid)
    }
}

impl PartialOrd for QueuedResponse {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

impl From<rpc::Response> for QueuedResponse {
    fn from(response: rpc::Response) -> Self {
        Self(response)
    }
}

impl From<QueuedResponse> for rpc::Response {
    fn from(value: QueuedResponse) -> Self {
        value.0
    }
}

pub struct StdinHandler {
    rx: Receiver<Message>,
    stdin: ChildStdin,
}

impl StdinHandler {
    pub fn start(self) {
        let Self { rx, mut stdin } = self;
        while let Ok(msg) = rx.recv() {
            match encode(&mut stdin, msg) {
                Ok(_) => {}
                Err(_) => return,
            }
        }
    }
}

pub struct StdoutHandler {
    incoming: Arc<RwLock<Incoming>>,
    stdout: ChildStdout,
}

impl StdoutHandler {
    pub fn start<N, R, S>(
        mut self,
        mut notification_handler: N,
        mut request_handler: R,
        shutdown_handler: S,
    ) where
        N: FnMut(rpc::Notification),
        R: FnMut(rpc::Request),
        S: Fn(),
    {
        use rmpv::decode::Error;
        loop {
            let msg = match decode(&mut self.stdout) {
                Ok(msg) => msg,
                Err(e) => {
                    match e {
                        DecodeError::Rmpv(e) => {
                            if let Some(io_error) = match &e {
                                Error::InvalidMarkerRead(e) => Some(e.kind()),
                                Error::InvalidDataRead(e) => Some(e.kind()),
                                Error::DepthLimitExceeded => None,
                            } {
                                match io_error {
                                    ErrorKind::UnexpectedEof => {}
                                    _ => log::error!("{e}"),
                                }
                            } else {
                                log::error!("{e}");
                            };
                        }
                        DecodeError::Parse => log::error!("Failed to parse an RPC message"),
                    }
                    shutdown_handler();
                    return;
                }
            };

            match msg {
                Message::Request(request) => {
                    log::info!("RPC Request: {}, {:?}", request.method, request.params);
                    self.incoming.write().unwrap().push_request(request.msgid);
                    request_handler(request);
                }

                Message::Response(rpc::Response {
                    msgid,
                    result,
                    error,
                }) => {
                    if error != Value::Nil {
                        log::error!("RPC response to {msgid}: {error:?}");
                    } else {
                        log::info!("RPC response to {msgid}: {result:?}");
                    };
                }

                Message::Notification(notification) => notification_handler(notification),
            };
        }
    }
}
