use crate::event::{self, Event};
use async_trait::async_trait;
use nvim_rs::{
    compat::tokio::Compat, create::tokio::new_child_cmd, error::LoopError, Handler, Neovim,
    UiAttachOptions, Value,
};
use tokio::{
    process::{ChildStdin, Command},
    sync::mpsc,
    task::JoinHandle,
};

pub type Writer = Compat<ChildStdin>;
pub type Nvim = Neovim<Writer>;

pub async fn spawn_neovim(
    width: u64,
    height: u64,
    tx: mpsc::Sender<Vec<Event>>,
) -> std::io::Result<(Nvim, IoHandle)> {
    let handler = NeovimHandler::new(tx);
    let (neovim, io_handle, _child) =
        new_child_cmd(Command::new("nvim").arg("--embed"), handler).await?;

    let mut options = UiAttachOptions::new();
    options.set_cmdline_external(true);
    options.set_hlstate_external(true);
    options.set_linegrid_external(true);
    // options.set_messages_external(true);
    options.set_multigrid_external(true);
    options.set_popupmenu_external(true);
    options.set_tabline_external(true);

    neovim
        .ui_attach(
            width.try_into().unwrap(),
            height.try_into().unwrap(),
            &options,
        )
        .await
        .unwrap();

    Ok((neovim, IoHandle(io_handle)))
}

pub struct IoHandle(JoinHandle<Result<(), Box<LoopError>>>);

impl IoHandle {
    pub async fn spin(self) {
        match self.0.await {
            Err(join_error) => log::error!("Error joining IO loop: '{}'", join_error),
            Ok(Err(error)) => {
                if !error.is_channel_closed() {
                    log::error!("Error: '{}'", error);
                }
            }
            Ok(Ok(())) => {}
        };
    }
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

#[async_trait]
impl Handler for NeovimHandler {
    type Writer = Compat<ChildStdin>;

    async fn handle_request(
        &self,
        name: String,
        _args: Vec<Value>,
        _neovim: Neovim<Self::Writer>,
    ) -> Result<Value, Value> {
        log::info!("Request: {name}");
        Ok(Value::Nil)
    }

    async fn handle_notify(&self, name: String, args: Vec<Value>, _neovim: Neovim<Self::Writer>) {
        match name.as_str() {
            "redraw" => {
                for arg in args {
                    match Event::try_parse(arg.clone()) {
                        Ok(events) => {
                            let _ = self.tx.send(events).await;
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
}
