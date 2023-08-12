use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub struct DoubleBuffer<T> {
    read: RwLock<T>,
    write: RwLock<T>,
}

impl<T> DoubleBuffer<T>
where
    T: Clone,
{
    pub fn new(initial: T) -> Self {
        Self {
            read: RwLock::new(initial.clone()),
            write: RwLock::new(initial),
        }
    }

    pub fn swap(&self) {
        let clone = self.write.read().unwrap().clone();
        *self.read.write().unwrap() = clone;
    }
}

impl<T> DoubleBuffer<T> {
    pub fn read(&self) -> RwLockReadGuard<T> {
        self.read.read().unwrap()
    }

    pub fn write(&self) -> RwLockWriteGuard<T> {
        self.write.write().unwrap()
    }
}

impl<T> Default for DoubleBuffer<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            read: Default::default(),
            write: Default::default(),
        }
    }
}
