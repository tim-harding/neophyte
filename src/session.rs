use crate::rpc::{decode, encode, RpcMessage};
use rmpv::Value;
use std::{
    io::{self, Error, ErrorKind},
    process::{Command, Stdio},
    sync::mpsc,
    thread,
};

pub struct Neovim {
    stdin_tx: mpsc::Sender<RpcMessage>,
    msgid: u64,
}

impl Neovim {
    pub fn new(notification_tx: mpsc::Sender<Notification>) -> io::Result<Neovim> {
        let mut child = Command::new("nvim")
            .arg("--embed")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        let mut stdout = child
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

        let stdout_tx = tx.clone();
        thread::spawn(move || loop {
            let msg = match decode(&mut stdout) {
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
                    match stdout_tx.send(response) {
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

                RpcMessage::Notification { method, params } => {
                    match notification_tx.send(Notification {
                        name: method,
                        instances: params,
                    }) {
                        Ok(_) => {}
                        Err(e) => {
                            log::error!("{e}");
                            return;
                        }
                    }
                }
            };
        });

        Ok(Neovim {
            stdin_tx: tx,
            msgid: 0,
        })
    }

    pub fn call(&mut self, method: &str, args: Vec<Value>) -> u64 {
        let msgid = self.msgid;
        self.msgid += 1;

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

    pub fn ui_attach(&mut self) {
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

    pub fn input(&mut self, input: String) {
        let args = vec![input.into()].into();
        self.call("nvim_input", args);
    }

    pub fn ui_try_resize_grid(&mut self, grid: u64, width: u64, height: u64) {
        let args: Vec<_> = [grid, width, height]
            .into_iter()
            .map(|n| n.into())
            .collect();
        self.call("nvim_ui_try_resize_grid", args);
    }
}

pub struct Notification {
    pub name: String,
    pub instances: Vec<Value>,
}
