mod event;
mod grid;
mod nvim;
mod rendering;

use nvim::spawn_neovim;
use tokio::runtime::Builder;

fn main() {
    let rt = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime");

    rt.block_on(async_main());
}

async fn async_main() {
    let (nvim, io_handle) = spawn_neovim().await.unwrap();
    tokio::spawn(async move {
        nvim.input(":things<left><left><cr>").await.unwrap();
    });
    io_handle.spin().await
}
