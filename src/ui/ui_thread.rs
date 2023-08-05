use super::Ui;
use crate::{
    event::{self, Event},
    session::Notification,
};
use std::sync::mpsc::Receiver;

pub fn ui_thread(rx: Receiver<Notification>) {
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
