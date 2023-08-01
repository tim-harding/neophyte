mod event;
mod rendering;

use crate::event::Event;
use async_trait::async_trait;
use nvim_rs::{compat::tokio::Compat, Handler, Neovim, UiAttachOptions, Value};
use std::process::Stdio;
use tokio::{
    process::{ChildStdin, Command},
    runtime::Builder,
};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

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
            match Event::try_from(arg.clone()) {
                Ok(event) => println!("{event:?}"),
                Err(e) => match e {
                    event::Error::UnknownEvent(name) => {
                        eprintln!("Unknown event: {name}\n{arg:#?}");
                    }
                    _ => eprintln!("{e}"),
                },
            }
        }
        println!();
    }
}

fn main() {
    let rt = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime");

    rt.block_on(async_main());
}

async fn async_main() {
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

    tokio::spawn(async move {
        neovim.input(":things<left><left><cr>").await.unwrap();
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
