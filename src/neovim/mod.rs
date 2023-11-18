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
    io::{self, ErrorKind},
    process::{Command, Stdio},
    sync::{mpsc, Arc, Mutex, RwLock},
};

#[derive(Debug, Clone)]
pub struct Neovim {
    tx: mpsc::Sender<rpc::Message>,
    incoming: Arc<RwLock<Incoming>>,
    next_msgid: Arc<Mutex<u64>>,
}

impl Neovim {
    pub fn new() -> io::Result<(Neovim, StdoutThread, StdinThread)> {
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
            StdoutThread::new(incoming, stdout),
            StdinThread::new(rx, stdin),
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
            // "ext_cmdline",
            // "ext_popupmenu",
            // "ext_tabline",
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
