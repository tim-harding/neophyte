use super::{scrolling_grids::ScrollingGrids, text::Text};
use crate::{
    event::rgb::Rgb,
    text::{cache::FontCache, fonts::Fonts},
    ui::{self, Ui},
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
        ui: &Ui,
        fonts: &Fonts,
        font_cache: &mut FontCache,
        shape_context: &mut ShapeContext,
    ) {
        self.grids.retain(|id, _| ui.grid(*id).is_some());

        let fg = ui.default_colors.rgb_fg.unwrap_or(Rgb::WHITE);
        let bg = ui.default_colors.rgb_bg.unwrap_or(Rgb::BLACK);

        for ui_grid in ui.grids.iter() {
            let grid = self.grids.entry(ui_grid.id).or_insert_with(|| {
                Grid::new(
                    Text::new(ui_grid.contents().size.try_cast().unwrap()),
                    // TODO: Does these need to be initialized with data?
                    // We might just fill it anyway down below.
                    ScrollingGrids::new(ui_grid.contents().clone()),
                )
            });

            let window_position = ui.position(ui_grid.id);
            if ui_grid.dirty.contents() {
                if ui_grid.scroll_delta != 0 {
                    grid.scrolling
                        .push(ui_grid.contents().clone(), ui_grid.scroll_delta);
                } else {
                    grid.scrolling.replace(ui_grid.contents().clone());
                }

                grid.text.update_contents(
                    device,
                    queue,
                    Some(grid.scrolling.size().try_cast().unwrap()),
                    grid.scrolling.rows(),
                    &self.bind_group_layout,
                    &ui.highlights,
                    fg,
                    bg,
                    fonts,
                    font_cache,
                    shape_context,
                );
            }

            grid.text.update_window(window_position);
        }

        self.draw_order.clear();
        self.draw_order
            .extend(ui.draw_order.iter().map(|draw_item| draw_item.grid));
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
