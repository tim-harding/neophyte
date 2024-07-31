pub mod action;
pub mod button;
mod incoming;
mod modifiers;
mod stdin_thread;
pub mod stdout_thread;

use self::{
    action::Action, button::Button, incoming::Incoming, modifiers::Modifiers,
    stdin_thread::StdinThread, stdout_thread::StdoutThread,
};
use crate::rpc::{self, Request};
use rmpv::Value;
use std::{
    ffi::OsStr,
    io::{self, ErrorKind},
    process::{Child, Command, Stdio},
    sync::{mpsc, Arc, RwLock},
};

#[derive(Debug, Clone)]
pub struct Neovim {
    tx: mpsc::Sender<rpc::Message>,
    incoming: Arc<RwLock<Incoming>>,
    next_msgid: u64,
}

impl Neovim {
    pub fn new(
        args: impl IntoIterator<Item = impl AsRef<OsStr>>,
    ) -> io::Result<(Neovim, StdoutThread, StdinThread, Child)> {
        use io::Error;
        let mut child = Command::new("nvim")
            .args(args)
            .arg("--embed")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
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
            StdoutThread::new(incoming, stdout),
            StdinThread::new(rx, stdin),
            child,
        ))
    }

    pub fn send_response(&self, response: rpc::Response) {
        self.incoming
            .write()
            .expect("The Neovim stdin thread closed unexpectedly")
            .push_response(response, &self.tx);
    }

    fn call(&mut self, method: &str, args: Vec<Value>) -> u64 {
        log::info!("Calling method {method}");
        let msgid = self.next_msgid;
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

        self.next_msgid += 1;
        msgid
    }

    // TODO: Proper public API
    pub fn ui_attach(&mut self) {
        let extensions = ["ext_multigrid", "ext_cmdline", "ext_messages"];
        let extensions = Value::Map(
            extensions
                .into_iter()
                .map(|arg| (arg.into(), true.into()))
                .collect(),
        );
        let attach_args = vec![80u64.into(), 10u64.into(), extensions];
        self.call("nvim_ui_attach", attach_args);
    }

    pub fn input(&mut self, input: String) {
        log::info!("Sending Neovim input: {input}");
        let args = vec![input.into()];
        self.call("nvim_input", args);
    }

    pub fn input_mouse(
        &mut self,
        button: Button,
        action: Action,
        modifiers: Modifiers,
        grid: u32,
        row: u32,
        col: u32,
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

    pub fn ui_try_resize_grid(&mut self, grid: u32, width: u32, height: u32) {
        let args: Vec<_> = [grid, width, height]
            .into_iter()
            .map(|n| n.into())
            .collect();
        self.call("nvim_ui_try_resize_grid", args);
    }

    pub fn ui_set_focus(&mut self, focus: bool) {
        self.call("nvim_ui_set_focus", vec![focus.into()]);
    }

    pub fn exec_lua(&mut self, lua: String, args: Vec<Value>) {
        self.call("nvim_exec_lua", vec![lua.into(), args.into()]);
    }
}
