use super::UserEvent;
use crate::{neovim::stdout_thread::StdoutHandler, rpc};
use winit::event_loop::EventLoopProxy;

pub struct NeovimHandler {
    proxy: EventLoopProxy<UserEvent>,
}

impl NeovimHandler {
    pub fn new(proxy: EventLoopProxy<UserEvent>) -> Self {
        Self { proxy }
    }
}

// Explicitly ignoring errors here because if we close the app through Neophyte
// instead of Neovim, the main thread will have already dropped the event loop.
impl StdoutHandler for NeovimHandler {
    fn handle_notification(&mut self, notification: rpc::Notification) {
        let _ = self.proxy.send_event(UserEvent::Notification(notification));
    }

    fn handle_request(&mut self, request: rpc::Request) {
        let _ = self.proxy.send_event(UserEvent::Request(request));
    }

    fn handle_shutdown(&mut self) {
        let _ = self.proxy.send_event(UserEvent::Shutdown);
    }
}
