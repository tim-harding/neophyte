mod event;
mod rendering;
mod rpc;
mod session;
pub mod text;
mod ui;
mod util;

use session::Neovim;
use std::{sync::mpsc, thread};
use ui::ui_thread;

fn main() {
    env_logger::builder().format_timestamp(None).init();
    let (notification_tx, notification_rx) = mpsc::channel();
    let (grid_tx, grid_rx) = mpsc::channel();
    let mut neovim = Neovim::new(notification_tx).unwrap();
    neovim.ui_attach();
    thread::spawn(move || {
        ui_thread(notification_rx, grid_tx);
    });
    pollster::block_on(rendering::run(grid_rx, neovim));
}
