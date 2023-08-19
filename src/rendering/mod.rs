mod font;
mod grid;
mod highlights;
mod read;
mod shared;
mod surface_config;
mod write;
mod texture;

use self::{read::ReadState, surface_config::SurfaceConfig, write::WriteState};
use crate::{
    session::Neovim,
    text::{
        cache::FontCache,
        fonts::{FontStyle, Fonts},
    },
    ui::Ui,
};
use std::sync::{mpsc::Receiver, Arc, RwLock};
use winit::{dpi::PhysicalSize, window::Window};

pub enum RenderEvent {
    Flush(Ui),
    Resized(PhysicalSize<u32>),
    Redraw,
}

pub fn render_loop(window: Arc<Window>, neovim: Neovim, rx: Receiver<RenderEvent>) {
    let (state, mut write_state) = {
        let window = window.clone();
        pollster::block_on(async { init(window.clone()).await })
    };
    let mut fonts = Fonts::new();
    let window = window.clone();
    let state = state.clone();
    let mut font_cache = FontCache::new();
    while let Ok(event) = rx.recv() {
        match event {
            RenderEvent::Flush(ui) => {
                state.update(ui, &mut write_state, &mut fonts, &mut font_cache);
                window.request_redraw();
            }
            RenderEvent::Resized(size) => {
                // TODO: Factor this stuff out. Duplicate logic from grid
                // rendering.
                state.resize(size);
                let metrics = fonts
                    .with_style(FontStyle::Regular)
                    .unwrap()
                    .as_ref()
                    .metrics(&[]);
                let scale_factor = fonts.size() as f32 / metrics.average_width;
                let em_px = (metrics.units_per_em as f32 * scale_factor).ceil() as u32;
                let descent_px = (metrics.descent as f32 * scale_factor).ceil() as u32;
                let cell_height_px = em_px + descent_px;
                neovim.ui_try_resize_grid(
                    1,
                    (size.width / fonts.size()) as u64,
                    (size.height / cell_height_px) as u64,
                )
            }

            RenderEvent::Redraw => match state.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => state.rebuild_swap_chain(),
                Err(wgpu::SurfaceError::OutOfMemory) => panic!("Out of memory"),
                Err(e) => eprintln!("{e:?}"),
            },
        }
    }
}

#[derive(Clone)]
pub struct State {
    surface_config: SurfaceConfig,
    constant: Arc<ConstantState>,
    read: Arc<RwLock<Option<ReadState>>>,
}

pub struct ConstantState {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface,
    pub grid: grid::Constant,
    pub font: font::Constant,
    pub highlights: highlights::Constant,
}

impl State {
    pub fn update(
        &self,
        ui: Ui,
        write: &mut WriteState,
        fonts: &mut Fonts,
        font_cache: &mut FontCache,
    ) {
        let updates = write.updates(ui, &self.constant, &self.surface_config, fonts, font_cache);
        let mut read = self.read.write().unwrap();
        match read.as_mut() {
            Some(read) => read.apply_updates(updates),
            None => *read = ReadState::from_updates(updates),
        }
    }

    pub fn resize(&self, size: PhysicalSize<u32>) {
        self.surface_config.resize(size, self.constant.as_ref());
    }

    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let read = self.read.read().unwrap();
        if let Some(read) = read.as_ref() {
            read.render(self.constant.as_ref())
        } else {
            Ok(())
        }
    }

    pub fn rebuild_swap_chain(&self) {
        let size = self.surface_config.size();
        let size = PhysicalSize::new(size.x, size.y);
        self.surface_config.resize(size, self.constant.as_ref())
    }
}

pub async fn init(window: Arc<Window>) -> (State, WriteState) {
    let size = window.inner_size();

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        dx12_shader_compiler: Default::default(),
    });

    let surface = unsafe { instance.create_surface(window.as_ref()) }.unwrap();

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::TEXTURE_BINDING_ARRAY
                    | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
                    | wgpu::Features::UNIFORM_BUFFER_AND_STORAGE_TEXTURE_ARRAY_NON_UNIFORM_INDEXING
                    | wgpu::Features::PUSH_CONSTANTS,
                limits: adapter.limits(),
            },
            None,
        )
        .await
        .unwrap();

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps
        .formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(surface_caps.formats[0]);

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: surface_caps.present_modes[0], // Vsync
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
    };
    surface.configure(&device, &config);

    let (highlights_write, highlights_constant) = highlights::init(&device);
    let (grid_write, grid_constant) = grid::init(&device, surface_format, &highlights_constant);
    let (font_write, font_constant) = font::new(&device);

    (
        State {
            surface_config: SurfaceConfig::new(config),
            constant: Arc::new(ConstantState {
                device,
                queue,
                surface,
                grid: grid_constant,
                font: font_constant,
                highlights: highlights_constant,
            }),
            read: Arc::new(RwLock::new(None)),
        },
        WriteState {
            grid: grid_write,
            font: font_write,
            highlights: highlights_write,
        },
    )
}
