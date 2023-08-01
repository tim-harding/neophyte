use crate::event::{self, Event};
use async_trait::async_trait;
use nvim_rs::create::tokio::new_child_cmd;
use nvim_rs::{compat::tokio::Compat, Handler, Neovim, Value};
use nvim_rs::{error::LoopError, UiAttachOptions};
use tokio::process::ChildStdin;
use tokio::process::Command;
use tokio::task::JoinHandle;

pub type Writer = Compat<ChildStdin>;
pub type Nvim = Neovim<Writer>;

pub async fn spawn_neovim() -> std::io::Result<(Nvim, IoHandle)> {
    let handler = NeovimHandler {};
    let (neovim, io_handle, _child) =
        new_child_cmd(Command::new("nvim").arg("--embed"), handler).await?;

    let mut options = UiAttachOptions::new();
    options.set_cmdline_external(true);
    options.set_hlstate_external(true);
    options.set_linegrid_external(true);
    options.set_messages_external(true);
    options.set_multigrid_external(true);
    options.set_popupmenu_external(true);
    options.set_tabline_external(true);

    // By default, the grid size is handled by Nvim and set to the outer grid
    // size (i.e. the size of the window frame in Nvim) whenever the split is
    // created. Once a UI sets a grid size, Nvim does not handle the size for
    // that grid and the UI must change the grid size whenever the outer size
    // is changed. To delegate grid-size handling back to Nvim, request the
    // size (0, 0).
    neovim.ui_attach(10, 10, &options).await.unwrap();

    Ok((neovim, IoHandle(io_handle)))
}

pub struct IoHandle(JoinHandle<Result<(), Box<LoopError>>>);

impl IoHandle {
    pub async fn spin(self) {
        match self.0.await {
            Err(join_error) => eprintln!("Error joining IO loop: '{}'", join_error),
            Ok(Err(error)) => {
                if !error.is_channel_closed() {
                    eprintln!("Error: '{}'", error);
                }
            }
            Ok(Ok(())) => {}
        };
    }
}

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
        match name.as_str() {
            "redraw" => {
                for arg in args {
                    match Event::try_parse(arg.clone()) {
                        Ok(event) => println!("{event:?}"),
                        Err(e) => match e {
                            event::Error::UnknownEvent(name) => {
                                eprintln!("Unknown event: {name}\n{arg:#?}");
                            }
                            _ => eprintln!("{e}"),
                        },
                    }
                }
            }
            _ => eprintln!("Unrecognized notification: {name}"),
        }
    }
}
