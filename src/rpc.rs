use crate::util::{Parse, Values};
use rmpv::{decode::read_value, encode::write_value, Value};
use std::{
    io::{self, Write},
    process::{ChildStdin, ChildStdout},
};

macro_rules! value_vec {
    ($($e:expr), *) => {{
        Value::from(vec![
        $(
            Value::from($e),
        )*
        ])
    }}
}

#[derive(Debug, PartialEq, Clone)]
pub enum Message {
    Request(Request),
    Response(Response),
    Notification(Notification),
}

impl Parse for Message {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        let msg_type: u64 = iter.next()?;
        Some(match msg_type {
            0 => Request {
                msgid: iter.next()?,
                method: iter.next()?,
                params: iter.next()?,
            }
            .into(),

            1 => Response {
                msgid: iter.next()?,
                error: iter.next()?,
                result: iter.next()?,
            }
            .into(),

            2 => Notification {
                method: iter.next()?,
                params: iter.next()?,
            }
            .into(),

            _ => return None,
        })
    }
}

impl From<Message> for Value {
    fn from(message: Message) -> Self {
        match message {
            Message::Request(request) => request.into(),
            Message::Response(response) => response.into(),
            Message::Notification(notification) => notification.into(),
        }
    }
}

impl From<Request> for Message {
    fn from(request: Request) -> Self {
        Self::Request(request)
    }
}

impl From<Response> for Message {
    fn from(response: Response) -> Self {
        Self::Response(response)
    }
}

impl From<Notification> for Message {
    fn from(notification: Notification) -> Self {
        Self::Notification(notification)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Request {
    pub msgid: u64,
    pub method: String,
    pub params: Vec<Value>,
}

impl From<Request> for Value {
    fn from(request: Request) -> Self {
        let Request {
            msgid,
            method,
            params,
        } = request;
        value_vec!(0, msgid, method, params)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Response {
    pub msgid: u64,
    pub error: Value,
    pub result: Value,
}

impl From<Response> for Value {
    fn from(response: Response) -> Self {
        let Response {
            msgid,
            error,
            result,
        } = response;
        value_vec!(1, msgid, error, result)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Notification {
    pub method: String,
    pub params: Vec<Value>,
}

impl From<Notification> for Value {
    fn from(notification: Notification) -> Self {
        let Notification { method, params } = notification;
        value_vec!(2, method, params)
    }
}

pub fn decode(reader: &mut ChildStdout) -> Result<Message, DecodeError> {
    Message::parse(read_value(reader)?).ok_or(DecodeError::Parse)
}

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("msgpack encode error: {0}")]
    Rmpv(#[from] rmpv::decode::Error),
    #[error("Failed to parse RPC message")]
    Parse,
}

pub fn encode(writer: &mut ChildStdin, msg: Message) -> Result<(), EncodeError> {
    let value = msg.into();
    write_value(writer, &value)?;
    writer.flush()?;
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum EncodeError {
    #[error("msgpack encode error: {0}")]
    Rmpv(#[from] rmpv::encode::Error),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}
