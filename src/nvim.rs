use crate::{
    event::{self, Event},
    session::{Handler, Session},
};
use rmpv::Value;
use std::sync::mpsc;

pub fn spawn_neovim(tx: mpsc::Sender<Vec<Event>>) -> Session {
    let handler = NeovimHandler::new(tx);
    let mut session = Session::new_child(handler).unwrap();
    session.ui_attach();
    session
}

#[derive(Clone)]
struct NeovimHandler {
    tx: mpsc::Sender<Vec<Event>>,
}

impl NeovimHandler {
    pub fn new(tx: mpsc::Sender<Vec<Event>>) -> Self {
        Self { tx }
    }
}

impl Handler for NeovimHandler {
    fn handle_notify(&mut self, name: &str, args: Vec<Value>) {
        match name {
            "redraw" => {
                // TODO: Parse on another thread?
                for arg in args {
                    match Event::try_parse(arg.clone()) {
                        Ok(events) => {
                            let _ = self.tx.send(events);
                        }
                        Err(e) => match e {
                            event::Error::UnknownEvent(name) => {
                                log::error!("Unknown event: {name}\n{arg:#?}");
                            }
                            _ => log::error!("{e}"),
                        },
                    }
                }
            }
            _ => log::error!("Unrecognized notification: {name}"),
        }
    }

    fn handle_request(&mut self, name: &str, args: Vec<Value>) -> Result<Value, Value> {
        log::info!("Request: {name}, {args:?}");
        Ok(Value::Nil)
    }

    fn handle_response(&mut self, _msgid: u64, response: Result<Value, Value>) {
        match response {
            Ok(response) => log::info!("{response:?}"),
            Err(error) => log::error!("{error:?}"),
        }
    }

    fn handle_close(&mut self) {
        log::error!("Neovim process closed");
    }
}
