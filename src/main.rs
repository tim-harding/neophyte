mod event;
mod nvim;
mod rendering;
mod ui;
mod util;

use event::Event;
use nvim::spawn_neovim;
use tokio::{runtime::Builder, sync::mpsc};
use ui::Ui;

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
    let (nvim, io_handle) = spawn_neovim(80, 10, tx).await.unwrap();
    tokio::spawn(async move {
        for c in 'a'..'z' {
            let s = format!("o{c}<esc>");
            nvim.input(s.as_str()).await.unwrap();
        }
        for _ in 0..26 {
            nvim.input("<up>").await.unwrap();
        }
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
