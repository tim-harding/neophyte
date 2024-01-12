use crate::rpc::{encode, Message};
use std::{io::BufWriter, process::ChildStdin, sync::mpsc::Receiver};

pub struct StdinThread {
    rx: Receiver<Message>,
    stdin: ChildStdin,
}

impl StdinThread {
    pub fn new(rx: Receiver<Message>, stdin: ChildStdin) -> Self {
        Self { rx, stdin }
    }

    pub fn start(self) {
        let Self { rx, stdin } = self;
        let mut stdin = BufWriter::new(stdin);
        loop {
            match rx.recv() {
                Ok(msg) => match encode(&mut stdin, msg) {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("{e}");
                        return;
                    }
                },

                Err(e) => {
                    log::error!("{e}");
                    return;
                }
            }
        }
    }
}
