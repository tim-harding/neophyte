use super::Ui;
use std::sync::{Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Default)]
pub struct UiHandle {
    needs_render_update: Mutex<bool>,
    ui: RwLock<Ui>,
}

impl UiHandle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read(&self) -> RwLockReadGuard<'_, Ui> {
        self.ui.read().unwrap()
    }

    pub fn read_and_consume_needs_render_update(&self) -> (RwLockReadGuard<'_, Ui>, bool) {
        let ui_lock = self.ui.read().unwrap();
        let mut needs_render_update_lock = self.needs_render_update.lock().unwrap();
        let needs_render_update = *needs_render_update_lock;
        *needs_render_update_lock = false;
        (ui_lock, needs_render_update)
    }

    pub fn write_and_get_needs_render_update(&self) -> (RwLockWriteGuard<'_, Ui>, bool) {
        let lock = self.ui.write().unwrap();
        let needs_render_update = *self.needs_render_update.lock().unwrap();
        (lock, needs_render_update)
    }

    pub fn end_write_and_set_needs_render_update(&self, _lock: RwLockWriteGuard<'_, Ui>) {
        *self.needs_render_update.lock().unwrap() = true;
    }
}
