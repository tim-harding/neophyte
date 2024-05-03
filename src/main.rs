mod assets;
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
use flexi_logger::Logger;
use neovim::Neovim;
use neovim_handler::NeovimHandler;
use std::{env, process::Output, thread};
use winit::event_loop::{ControlFlow, EventLoop};

const HELP_TEXT: &str = "\
A WebGPU-rendered Neovim GUI

SYNOPSIS 
    neophyte [OPTIONS] [-- NVIM_COMMAND]

DESCRIPTION
    Opens the GUI with the given options and Neovim command. 
    All the arguments following the two dashes (--) specify the Neovim command to run. 
    If two dashes are not given, the default command `nvim` is run instead.

OPTIONS
    -t, --transparent  
        Enable window transparency
    -h, --help
        Show this help text

EXAMPLES
    1. Run Neophyte with defaults. The following commands are equivalent.
        neophyte
        neophyte -- nvim

    2. Run Neophyte with a transparent window and a clean Neovim configuration.
        neophyte --transparent -- nvim --clean
";

fn main() {
    match Logger::try_with_env_or_str("neophyte=warn")
        .unwrap()
        .start()
    {
        Ok(_logger) => {}
        Err(e) => eprintln!("Failed to start logging: {e}"),
    }

    let mut transparent = false;
    let mut args = env::args().skip(1);
    for arg in &mut args {
        match arg.as_str() {
            "--" => break,
            "--transparent" | "-t" => transparent = true,
            "--help" | "-h" => {
                print!("{}", HELP_TEXT);
                return;
            }
            other => log::warn!("Unrecognized argument: {}", other),
        }
    }

    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .build()
        .expect("Failed to create event loop");

    let (mut neovim, stdout_handler, stdin_handler, child) =
        Neovim::new(args).expect("Failed to start Neovim");
    neovim.ui_attach();
    let stdin_thread = std::thread::spawn(move || stdin_handler.start());
    let proxy = event_loop.create_proxy();
    let stdout_thread = thread::spawn(move || {
        stdout_handler.start(NeovimHandler::new(proxy));
    });

    {
        let mut handler = EventHandler::new(neovim, transparent);
        event_loop.set_control_flow(ControlFlow::Wait);
        event_loop
            .run_app(&mut handler)
            .expect("Failed to start event loop");
    } // Dropping handler drops channels for faster shutdown

    match child.wait_with_output() {
        Ok(output) => {
            let Output {
                status,
                stdout: _,
                stderr,
            } = output;
            let stderr = match String::from_utf8(stderr) {
                Ok(stderr) => stderr,
                Err(_) => {
                    log::error!("Unable to get Neovim stderr as a string");
                    String::new()
                }
            };
            log::info!("Neovim exited with {status} and stderr: {stderr}");
        }
        Err(e) => log::error!("{e}"),
    }

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
