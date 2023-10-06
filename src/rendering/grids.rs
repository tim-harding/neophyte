use super::grid::{self, Grid};
use crate::{
    text::{cache::FontCache, fonts::Fonts},
    ui::Ui,
};
use swash::shape::ShapeContext;

pub struct Grids {
    grids: Vec<Grid>,
    draw_order_cache: Vec<usize>,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl Grids {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            grids: vec![],
            draw_order_cache: vec![],
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
        ui: &Ui,
        fonts: &mut Fonts,
        font_cache: &mut FontCache,
        shape_context: &mut ShapeContext,
    ) {
        let mut i = 0;
        while let Some(grid) = self.grids.get(i) {
            if ui.grid_index(grid.id()).is_ok() {
                i += 1;
            } else {
                self.grids.remove(i);
            }
        }

        for ui_grid in ui.grids.iter() {
            let index = match self
                .grids
                .binary_search_by(|probe| probe.id().cmp(&ui_grid.id))
            {
                Ok(index) => index,
                Err(index) => {
                    self.grids.insert(index, Grid::new(ui_grid.id));
                    index
                }
            };
            let grid = &mut self.grids[index];

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
                    &ui.highlights,
                    fonts,
                    font_cache,
                    shape_context,
                    ui_grid.current(),
                );
            }

            if ui_grid.is_window_dirty() {
                grid.update_window(
                    ui.position(ui_grid.id),
                    fonts.metrics().into_pixels().cell_size().cast(),
                );
            }
        }

        self.draw_order_cache.clear();
        for &id in ui.draw_order.iter().rev() {
            let i = self
                .grids
                .binary_search_by(|probe| probe.id().cmp(&{ id }))
                .unwrap();
            self.draw_order_cache.push(i);
        }
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn front_to_back(&self) -> impl Iterator<Item = (f32, &Grid)> {
        let len = self.draw_order_cache.len() as f32;
        self.draw_order_cache
            .iter()
            .enumerate()
            .map(move |(i, &grid_i)| (i as f32 / len, &self.grids[grid_i]))
    }

    pub fn iter(&self) -> impl Iterator<Item = &Grid> {
        self.grids.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Grid> {
        self.grids.iter_mut()
    }
}
