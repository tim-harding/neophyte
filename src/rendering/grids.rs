use super::grid::Grid;
use crate::{
    event::hl_attr_define::Attributes,
    text::{cache::FontCache, fonts::Fonts},
    ui::grid::DoubleBufferGrid,
    util::vec2::Vec2,
};
use std::collections::HashMap;
use swash::shape::ShapeContext;

pub struct Grids {
    grids: HashMap<u64, Grid>,
    draw_order: Vec<u64>,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl Grids {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            grids: HashMap::new(),
            draw_order: vec![],
            bind_group_layout: device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Grid bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            }),
        }
    }

    pub fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        ui_grid: &DoubleBufferGrid,
        position: Vec2<f64>,
        highlights: &[Attributes],
        fonts: &Fonts,
        font_cache: &mut FontCache,
        shape_context: &mut ShapeContext,
    ) {
        let grid = self.grids.entry(ui_grid.id).or_insert(Grid::new());

        if ui_grid.scroll_delta != 0 {
            grid.scrolling_mut().push(
                ui_grid.previous(),
                ui_grid.scroll_delta,
                ui_grid.current().size.y as usize,
            );
        }

        if ui_grid.is_grid_dirty() {
            grid.update_grid(
                device,
                queue,
                &self.bind_group_layout,
                &highlights,
                fonts,
                font_cache,
                shape_context,
                ui_grid.current(),
            );
        }

        if ui_grid.is_window_dirty() {
            grid.update_window(position, fonts.metrics().into_pixels().cell_size().cast());
        }
    }

    pub fn remove_grid(&mut self, id: u64) {
        self.grids.remove(&id);
    }

    pub fn set_draw_order(&mut self, draw_order: Vec<u64>) {
        self.draw_order = draw_order;
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn front_to_back(&self) -> impl Iterator<Item = (f32, &Grid)> {
        let len = self.draw_order.len() as f32;
        self.draw_order
            .iter()
            .rev()
            .enumerate()
            .map(move |(i, &grid_id)| (i as f32 / len, self.grids.get(&grid_id).unwrap()))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Grid> {
        self.grids.values_mut()
    }
}
