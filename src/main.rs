#[warn(
    clippy::cast_lossless,
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::unnecessary_cast
)]
mod event;
mod event_handler;
mod neovim;
mod neovim_handler;
mod rendering;
mod rpc;
pub mod text;
mod ui;
mod util;

use event_handler::EventHandler;
use neovim::Neovim;
use neovim_handler::NeovimHandler;
use std::thread;
use winit::{
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};

fn main() {
    env_logger::builder().format_timestamp(None).init();

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event()
        .build()
        .expect("Failed to create event loop");
    let window = WindowBuilder::new()
        .with_transparent(true)
        .build(&event_loop)
        .expect("Failed to create window");

    let (mut neovim, stdout_handler, stdin_handler) =
        Neovim::new().expect("Failed to start Neovim");
    neovim.ui_attach();
    let stdin_thread = std::thread::spawn(move || stdin_handler.start());
    let proxy = event_loop.create_proxy();
    let stdout_thread = thread::spawn(move || {
        stdout_handler.start(NeovimHandler::new(proxy));
    });

    let mut handler = EventHandler::new(neovim, window);
    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop
        .run(move |event, window_target| {
            handler.handle(event, window_target);
        })
        .expect("Failed to start render loop");

    stdout_thread
        .join()
        .expect("Failed to join Neovim stdout thread");
    stdin_thread
        .join()
        .expect("Failed to join Neovim stdin thread");
}

#[derive(Debug)]
pub enum UserEvent {
    Notification(rpc::Notification),
    Request(rpc::Request),
    Shutdown,
}
