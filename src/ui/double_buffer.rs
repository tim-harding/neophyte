use super::Ui;
use std::{
    mem,
    sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard},
};

#[derive(Clone, Default)]
pub struct DoubleBuffer {
    locks: Arc<Locks>,
}

impl DoubleBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read(&self) -> RwLockReadGuard<Ui> {
        self.locks.back.read().unwrap()
    }

    pub fn write(&self) -> MutexGuard<Ui> {
        self.locks.front.lock().unwrap()
    }

    pub fn swap(&self) {
        self.locks.swap()
    }
}

#[derive(Default)]
struct Locks {
    front: Mutex<Ui>,
    back: RwLock<Ui>,
}

impl Locks {
    pub fn swap(&self) {
        if let (Ok(mut front), Ok(mut back)) = (self.front.lock(), self.back.write()) {
            mem::swap(&mut *front, &mut *back);
        }
    }
}
