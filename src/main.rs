mod event;
mod image;
mod rendering;
mod rpc;
mod session;
mod ui;
mod util;

use session::{Neovim, Notification};
use std::{sync::mpsc, thread};
use ui::ui_thread;

fn main() {
    env_logger::builder().format_timestamp(None).init();

    if std::env::var("RUN_NVIM").is_ok() {
        let (tx, rx) = mpsc::channel::<Notification>();
        let mut neovim = Neovim::new(tx).unwrap();
        neovim.ui_attach();
        neovim.input("ithings<esc>".to_string());
        thread::spawn(move || {
            ui_thread(rx);
        });
    }

    if std::env::var("RUN_GPU").is_ok() {
        pollster::block_on(rendering::run());
    }

    image::render();
}
