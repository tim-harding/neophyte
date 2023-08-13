use super::StateConstant;
use crate::util::vec2::Vec2;
use std::sync::{Arc, RwLock};
use winit::dpi::PhysicalSize;

#[derive(Clone)]
pub struct StateSurfaceConfig {
    config: Arc<RwLock<wgpu::SurfaceConfiguration>>,
}

impl StateSurfaceConfig {
    pub fn new(config: wgpu::SurfaceConfiguration) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
        }
    }

    pub fn resize(&self, new_size: PhysicalSize<u32>, constant: &StateConstant) {
        if new_size.width > 0 && new_size.height > 0 {
            {
                let mut lock = self.config.write().unwrap();
                lock.width = new_size.width;
                lock.height = new_size.height;
            }
            let lock = self.config.read().unwrap();
            constant.surface.configure(&constant.device, &*lock);
        }
    }

    pub fn size(&self) -> Vec2<u32> {
        let lock = self.config.read().unwrap();
        Vec2::new(lock.width, lock.height)
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.config.read().unwrap().format
    }
}
