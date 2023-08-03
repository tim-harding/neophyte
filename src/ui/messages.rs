use super::{print::print_content, Highlights};
use crate::event::{msg_history_show::MsgHistoryEntry, msg_show::ReplaceLast, Content, MsgShow};

type Message = MsgHistoryEntry;

#[derive(Debug, Clone, Default)]
pub struct Messages {
    pub show: Vec<Message>,
    pub history: Vec<Message>,
    pub showmode: Content,
    pub showcmd: Content,
    pub ruler: Content,
}

impl Messages {
    pub fn show(&mut self, event: MsgShow) {
        match event.replace_last {
            ReplaceLast::Replace => {
                self.show.pop();
            }
            ReplaceLast::Keep => {}
        }
        let message = Message {
            kind: event.kind,
            content: event.content,
        };
        self.show.push(message);
    }
}

#[allow(unused)]
pub fn eprint_messages(messages: &[Message], highlights: &Highlights) {
    for message in messages {
        eprint!("msg_show {:?}: ", message.kind);
        print_content(&message.content, highlights);
        eprintln!();
    }
}
