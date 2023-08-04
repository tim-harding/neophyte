mod event;
mod model;
mod nvim;
mod rendering;
mod session;
mod ui;
mod util;

use event::Event;
use nvim::spawn_neovim;
use std::{sync::mpsc, thread};
use ui::Ui;

fn main() {
    env_logger::builder().format_timestamp(None).init();
    let (tx, rx) = mpsc::channel::<Vec<Event>>();
    let mut session = spawn_neovim(tx);
    thread::spawn(move || {
        let mut ui = Ui::new();
        while let Ok(events) = rx.recv() {
            for event in events {
                ui.process(event);
            }
        }
    });
    session.input(":w<cr><esc>:messages<cr>".to_string());
    loop {}
}
