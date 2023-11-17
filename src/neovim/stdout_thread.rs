use std::{
    io::ErrorKind,
    process::ChildStdout,
    sync::{Arc, RwLock},
};

use rmpv::Value;

use crate::rpc::{self, decode, DecodeError, Message};

use super::Incoming;

pub struct StdoutThread {
    incoming: Arc<RwLock<Incoming>>,
    stdout: ChildStdout,
}

impl StdoutThread {
    pub fn new(incoming: Arc<RwLock<Incoming>>, stdout: ChildStdout) -> Self {
        Self { incoming, stdout }
    }

    pub fn start<H>(mut self, mut handler: H)
    where
        H: StdoutHandler,
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
                    handler.handle_shutdown();
                    return;
                }
            };

            match msg {
                Message::Request(request) => {
                    log::info!("RPC Request: {}, {:?}", request.method, request.params);
                    self.incoming.write().unwrap().push_request(request.msgid);
                    handler.handle_request(request);
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

                Message::Notification(notification) => handler.handle_notification(notification),
            };
        }
    }
}

pub trait StdoutHandler {
    fn handle_notification(&mut self, notification: rpc::Notification);
    fn handle_request(&mut self, request: rpc::Request);
    fn handle_shutdown(&mut self);
}
