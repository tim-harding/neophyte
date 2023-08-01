mod event;
mod grid;
mod nvim;
mod rendering;

use event::Event;
use grid::Ui;
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
    env_logger::builder().format_timestamp(None).init();
    let (tx, mut rx) = mpsc::channel::<Vec<Event>>(32);
    let (nvim, io_handle) = spawn_neovim(80, 80, tx).await.unwrap();
    tokio::spawn(async move {
        nvim.input(":things<left><left><cr>").await.unwrap();
    });
    tokio::spawn(async move {
        let mut ui = Ui::new();
        while let Some(events) = rx.recv().await {
            for event in events {
                ui.process(event);
            }
        }
    });
    io_handle.spin().await
}
