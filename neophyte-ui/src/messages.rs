use neophyte_ui_event::{
    Content, MsgShow, msg_history_show::MsgHistoryEntry, msg_show::ReplaceLast,
};

type Message = MsgHistoryEntry;

#[derive(Debug, Clone, Default)]
pub struct Messages {
    pub dirty: bool,
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
