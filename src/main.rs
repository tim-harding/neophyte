mod event;
mod rendering;
mod rpc;
mod session;
mod ui;
mod util;

use event::Event;
use session::{Notification, Session};
use std::sync::mpsc;
use ui::Ui;

fn main() {
    env_logger::builder().format_timestamp(None).init();
    let (tx, rx) = mpsc::channel::<Notification>();
    let mut session = Session::new_child(tx).unwrap();
    session.ui_attach();
    session.input("ithings<esc>".to_string());
    let mut ui = Ui::new();
    while let Ok(Notification { name, instances }) = rx.recv() {
        match name.as_str() {
            "redraw" => {
                for instance in instances {
                    match Event::try_parse(instance.clone()) {
                        Ok(events) => {
                            for event in events {
                                ui.process(event);
                            }
                        }
                        Err(e) => match e {
                            event::Error::UnknownEvent(name) => {
                                log::error!("Unknown event: {name}\n{instance:#?}");
                            }
                            _ => log::error!("{e}"),
                        },
                    }
                }
            }
            _ => log::error!("Unrecognized notification: {name}"),
        }
    }
}
