use crate::util::{Parse, Values};
use rmpv::{decode::read_value, encode::write_value, Value};
use std::{
    io::{self, Write},
    process::{ChildStdin, ChildStdout},
};

#[derive(Debug, PartialEq, Clone)]
pub enum RpcMessage {
    Request {
        msgid: u64,
        method: String,
        params: Vec<Value>,
    },
    Response {
        msgid: u64,
        error: Value,
        result: Value,
    },
    Notification {
        method: String,
        params: Vec<Value>,
    },
}

impl Parse for RpcMessage {
    fn parse(value: Value) -> Option<Self> {
        let mut iter = Values::new(value)?;
        let msg_type: u64 = iter.next()?;
        Some(match msg_type {
            0 => Self::Request {
                msgid: iter.next()?,
                method: iter.next()?,
                params: iter.next()?,
            },
            1 => Self::Response {
                msgid: iter.next()?,
                error: iter.next()?,
                result: iter.next()?,
            },
            2 => Self::Notification {
                method: iter.next()?,
                params: iter.next()?,
            },
            _ => return None,
        })
    }
}

pub fn decode(reader: &mut ChildStdout) -> Result<RpcMessage, DecodeError> {
    RpcMessage::parse(read_value(reader)?).ok_or(DecodeError::Parse)
}

macro_rules! value_vec {
    ($($e:expr), *) => {{
        let mut vec = Vec::new();
        $(
            vec.push(Value::from($e));
        )*
        Value::from(vec)
    }}
}

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("msgpack encode error: {0}")]
    Rmpv(#[from] rmpv::decode::Error),
    #[error("Failed to parse RPC message")]
    Parse,
}

pub fn encode(writer: &mut ChildStdin, msg: RpcMessage) -> Result<(), EncodeError> {
    let value = match msg {
        RpcMessage::Request {
            msgid,
            method,
            params,
        } => {
            value_vec!(0, msgid, method, params)
        }
        RpcMessage::Response {
            msgid,
            error,
            result,
        } => {
            value_vec!(1, msgid, error, result)
        }
        RpcMessage::Notification { method, params } => {
            value_vec!(2, method, params)
        }
    };

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
