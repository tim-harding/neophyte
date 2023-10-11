use super::Ui;
use std::{
    mem,
    sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard},
};

#[derive(Clone, Default)]
pub struct DoubleBuffer {
    front: Arc<Mutex<Ui>>,
    back: Arc<RwLock<Ui>>,
}

impl DoubleBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read(&self) -> RwLockReadGuard<Ui> {
        self.back.read().unwrap()
    }

    pub fn write(&self) -> MutexGuard<Ui> {
        self.front.lock().unwrap()
    }

    pub fn back(&self) -> Arc<RwLock<Ui>> {
        self.back.clone()
    }

    pub fn swap(&self) {
        if let (Ok(mut front), Ok(mut back)) = (self.front.lock(), self.back.write()) {
            mem::swap(&mut *front, &mut *back);
        }
    }
}
