use async_trait::async_trait;
use nvim_rs::{compat::tokio::Compat, Handler, Neovim, UiAttachOptions, Value};
use std::process::Stdio;
use tokio::process::{ChildStdin, Command};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

mod rendering;

#[derive(Clone)]
struct NeovimHandler {}

#[async_trait]
impl Handler for NeovimHandler {
    type Writer = Compat<ChildStdin>;

    async fn handle_request(
        &self,
        name: String,
        _args: Vec<Value>,
        _neovim: Neovim<Self::Writer>,
    ) -> Result<Value, Value> {
        println!("Request: {name}");
        Ok(Value::Nil)
    }

    async fn handle_notify(&self, name: String, args: Vec<Value>, _neovim: Neovim<Self::Writer>) {
        println!("Notify: {name}");
        for arg in args {
            match Event::try_from(arg) {
                Ok(event) => println!("{event:?}"),
                Err(e) => match e {
                    EventParseError::UnknownEvent(name) => eprintln!("Unknown event: {name}"),
                    _ => eprintln!("{e}"),
                },
            }
        }
        println!();
    }
}

#[tokio::main]
async fn main() {
    let mut child = Command::new("nvim")
        .arg("--embed")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let handler = NeovimHandler {};
    let reader = child.stdout.take().unwrap();
    let writer = child.stdin.take().unwrap();
    let (neovim, io) = Neovim::new(reader.compat(), writer.compat_write(), handler);
    let io_handle = tokio::spawn(io);

    let mut options = UiAttachOptions::new();
    options.set_linegrid_external(true);
    neovim.ui_attach(512, 512, &options).await.unwrap();

    tokio::spawn(async move {
        neovim.input("iThings<esc>").await.unwrap();
    });

    match io_handle.await {
        Err(join_error) => eprintln!("Error joining IO loop: '{}'", join_error),
        Ok(Err(error)) => {
            if !error.is_channel_closed() {
                eprintln!("Error: '{}'", error);
            }
        }
        Ok(Ok(())) => {}
    };
}

#[derive(Debug, Clone, Copy)]
enum Event {
    GridResize(GridResize),
}

impl TryFrom<Value> for Event {
    type Error = EventParseError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Array(array) => {
                let mut iter = array.into_iter();
                let event_name = iter.next().ok_or(EventParseError::Malformed)?;
                match event_name {
                    Value::String(s) => match s.as_str() {
                        Some(s) => match s {
                            "grid_resize" => Ok(Self::GridResize(GridResize::try_from(
                                iter.next().ok_or(EventParseError::Malformed)?,
                            )?)),
                            _ => Err(EventParseError::UnknownEvent(s.to_string())),
                        },
                        None => Err(EventParseError::Malformed),
                    },
                    _ => Err(EventParseError::Malformed),
                }
            }
            _ => Err(EventParseError::Malformed),
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
enum EventParseError {
    #[error("Event is malformed")]
    Malformed,
    #[error("Received an unrecognized event name: {0}")]
    UnknownEvent(String),
    #[error("{0}")]
    GridResize(#[from] GridResizeParseError),
}

#[derive(Debug, Clone, Copy)]
struct GridResize {
    pub grid: u64,
    pub width: u64,
    pub height: u64,
}

impl TryFrom<Value> for GridResize {
    type Error = GridResizeParseError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Array(array) => {
                let mut iter = array.into_iter().map(pos_int).flatten();
                let grid = iter.next().ok_or(GridResizeParseError)?;
                let width = iter.next().ok_or(GridResizeParseError)?;
                let height = iter.next().ok_or(GridResizeParseError)?;
                Ok(Self {
                    grid,
                    width,
                    height,
                })
            }
            _ => Err(GridResizeParseError),
        }
    }
}

fn pos_int(value: Value) -> Option<u64> {
    match value {
        Value::Integer(n) => n.as_u64(),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("Failed to parse grid_resize event")]
struct GridResizeParseError;
