mod event;
mod grid;
mod nvim;
mod rendering;

use event::Event;
use grid::Grids;
use nvim::spawn_neovim;
use tokio::{runtime::Builder, sync::mpsc};

fn main() {
    let rt = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime");

    rt.block_on(async_main());
}

async fn async_main() {
    let (tx, mut rx) = mpsc::channel::<Vec<Event>>(32);
    let (nvim, io_handle) = spawn_neovim(tx).await.unwrap();
    tokio::spawn(async move {
        nvim.input(":things<left><left><cr>").await.unwrap();
    });
    tokio::spawn(async move {
        let mut grids = Grids::new();
        while let Some(events) = rx.recv().await {
            for event in events {
                grids.process(event);
            }
        }
    });
    io_handle.spin().await
}
