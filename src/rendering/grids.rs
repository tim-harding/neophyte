use super::{grid::Text, scrolling_grids::ScrollingGrids};
use crate::{
    event::{rgb::Rgb, HlAttrDefine},
    text::{cache::FontCache, fonts::Fonts},
    ui::grid::Grid as UiGrid,
    util::vec2::Vec2,
};
use std::collections::HashMap;
use swash::shape::ShapeContext;

pub struct Grid {
    pub text: Text,
    pub scrolling: ScrollingGrids,
}

impl Grid {
    pub fn new(text: Text, scrolling: ScrollingGrids) -> Self {
        Self { text, scrolling }
    }

    pub fn offset(&self, cell_height: f32) -> Vec2<i32> {
        self.text.offset() + self.scrolling.offset(cell_height)
    }
}

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

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        ui_grid: &UiGrid,
        position: Vec2<f64>,
        highlights: &[HlAttrDefine],
        default_fg: Rgb,
        default_bg: Rgb,
        fonts: &Fonts,
        font_cache: &mut FontCache,
        shape_context: &mut ShapeContext,
    ) {
        let grid = self
            .grids
            .entry(ui_grid.id)
            // TODO: Does scrolling grids really need to be initialized with
            // contents?
            .or_insert_with(|| {
                Grid::new(
                    Text::new(ui_grid.contents().size.try_cast().unwrap()),
                    ScrollingGrids::new(ui_grid.contents().clone()),
                )
            });

        if ui_grid.scroll_delta != 0 {
            grid.scrolling
                .push(ui_grid.contents().clone(), ui_grid.scroll_delta);
        } else {
            grid.scrolling.replace_last(ui_grid.contents().clone());
        }

        if ui_grid.dirty.contents() {
            grid.text.update_contents(
                device,
                queue,
                grid.scrolling.size().try_cast().unwrap(),
                grid.scrolling.rows(),
                &self.bind_group_layout,
                highlights,
                default_fg,
                default_bg,
                fonts,
                font_cache,
                shape_context,
            );
        }

        if ui_grid.dirty.window() {
            grid.text.update_window(position, fonts.cell_size().cast());
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
