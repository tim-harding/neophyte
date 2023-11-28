use super::{scrolling_grids::ScrollingGrids, text::Text};
use crate::{
    event::{hl_attr_define::Attributes, rgb::Rgb},
    text::{cache::FontCache, fonts::Fonts},
    ui::{self, grid::Grid as UiGrid},
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
    grids: HashMap<ui::grid::Id, Grid>,
    draw_order: Vec<ui::grid::Id>,
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
        position: Vec2<f32>,
        highlights: &[Option<Attributes>],
        default_fg: Rgb,
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
                    // TODO: Uniform type sizes
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
                // TODO: Uniform type sizes
                Some(grid.scrolling.size().try_cast().unwrap()),
                grid.scrolling.rows(),
                &self.bind_group_layout,
                highlights,
                default_fg,
                fonts,
                font_cache,
                shape_context,
            );
        }

        if ui_grid.dirty.window() {
            grid.text
                .update_window(position, fonts.cell_size().cast_as());
        }
    }

    pub fn remove_grid(&mut self, id: ui::grid::Id) {
        self.grids.remove(&id);
    }

    pub fn set_draw_order(&mut self, draw_order: Vec<ui::grid::Id>) {
        self.draw_order = draw_order;
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn grid_count(&self) -> usize {
        self.draw_order.len()
    }

    pub fn front_to_back(&self) -> impl Iterator<Item = (usize, &Grid)> {
        self.draw_order
            .iter()
            .rev()
            .enumerate()
            .map(move |(i, &grid_id)| (i, self.grids.get(&grid_id).unwrap()))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Grid> {
        self.grids.values_mut()
    }
}
