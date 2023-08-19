use crate::rpc::{decode, encode, RpcMessage};
use rmpv::Value;
use std::{
    io::{self, Error, ErrorKind},
    process::{ChildStdout, Command, Stdio},
    sync::{mpsc, Arc, Mutex},
    thread,
};

#[derive(Debug, Clone)]
pub struct Neovim {
    stdin_tx: mpsc::Sender<RpcMessage>,
    msgid: Arc<Mutex<u64>>,
}

impl Neovim {
    pub fn new() -> io::Result<(Neovim, Handler)> {
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
        let args = vec![input.into()].into();
        self.call("nvim_input", args);
    }

    pub fn ui_try_resize_grid(&self, grid: u64, width: u64, height: u64) {
        let args: Vec<_> = [grid, width, height]
            .into_iter()
            .map(|n| n.into())
            .collect();
        self.call("nvim_ui_try_resize_grid", args);
    }
}

pub struct Handler {
    stdin_tx: mpsc::Sender<RpcMessage>,
    stdout: ChildStdout,
}

impl Handler {
    pub fn start<F>(mut self, mut notification_handler: F)
    where
        F: FnMut(String, Vec<Value>),
    {
        loop {
            let msg = match decode(&mut self.stdout) {
                Ok(msg) => msg,
                Err(e) => {
                    log::error!("{e}");
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
