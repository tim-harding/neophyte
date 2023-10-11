use super::Ui;
use crate::event::Event;
use std::{
    mem,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex, RwLock, RwLockReadGuard,
    },
    thread::{self, JoinHandle},
};

pub fn new() -> (Handle, Thread) {
    let (tx, rx) = mpsc::channel();
    let locks = Arc::new(Locks::new());
    let handle = Handle {
        locks: locks.clone(),
        tx,
    };
    let thread = Thread { locks, rx };
    (handle, thread)
}

#[derive(Clone)]
pub struct Handle {
    locks: Arc<Locks>,
    tx: Sender<Message>,
}

impl Handle {
    pub fn get(&self) -> RwLockReadGuard<Ui> {
        self.locks.back.read().unwrap()
    }

    pub fn process(&self, event: Event) {
        self.tx.send(Message::Event(event)).unwrap()
    }

    pub fn swap(&self) {
        self.tx.send(Message::Swap).unwrap()
    }
}

pub struct Thread {
    locks: Arc<Locks>,
    rx: Receiver<Message>,
}

impl Thread {
    pub fn run(self) -> JoinHandle<()> {
        let Self { locks, rx } = self;
        thread::spawn(move || {
            while let Ok(message) = rx.recv() {
                match message {
                    Message::Event(event) => {
                        locks.front.lock().unwrap().process(event);
                    }
                    Message::Swap => locks.swap(),
                }
            }
        })
    }
}

#[derive(Default)]
struct Locks {
    front: Mutex<Ui>,
    back: RwLock<Ui>,
}

impl Locks {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn swap(&self) {
        if let (Ok(mut front), Ok(mut back)) = (self.front.lock(), self.back.write()) {
            mem::swap(&mut *front, &mut *back);
        }
    }
}

enum Message {
    Event(Event),
    Swap,
}
