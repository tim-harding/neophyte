pub mod cmdline;
pub mod grid;
pub mod messages;
pub mod options;
pub mod window;

use self::{
    cmdline::Cmdline, grid::Grid, messages::Messages, options::GuiFont, window::WindowOffset,
};
use crate::{
    event::{
        hl_attr_define::Attributes, mode_info_set::ModeInfo, Chdir, CmdlineBlockAppend,
        CmdlineBlockShow, CmdlinePos, DefaultColorsSet, Event, GridClear, GridCursorGoto,
        GridDestroy, GridLine, GridResize, GridScroll, HlGroupSet, ModeChange, ModeInfoSet,
        MsgHistoryShow, MsgRuler, MsgSetPos, MsgShowcmd, MsgShowmode, OptionSet, PopupmenuSelect,
        PopupmenuShow, TablineUpdate, WinClose, WinExternalPos, WinFloatPos, WinHide, WinPos,
        WinViewport,
    },
    ui::window::{FloatingWindow, NormalWindow, Window},
    util::vec2::{CellVec, PixelVec, Vec2},
};
use std::{collections::HashMap, fmt::Debug};

pub type HlId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrawItem {
    pub grid: grid::Id,
    pub z: Option<u32>,
}

impl DrawItem {
    pub const fn new(grid: grid::Id, z: Option<u32>) -> Self {
        Self { grid, z }
    }
}

/// Manages updates to the UI state from UI events
#[derive(Clone, Default)]
pub struct Ui {
    /// UI grids, ordered by ID
    pub grids: Vec<Grid>,
    /// The order in which grids should be drawn, ordered from bottom to top
    pub draw_order: Vec<DrawItem>,
    /// The index into self.draw_order at which floating windows begin
    pub float_windows_start: usize,
    /// Cursor information
    pub cursor: CursorInfo,
    /// Whether the mouse is enabled
    pub mouse: bool,
    /// UI highlights, indexed by their ID
    // TODO: Only store the rgb_attr part
    pub highlights: Vec<Option<Attributes>>,
    /// A lookup from highlight names to highlight IDs
    pub highlight_groups: HashMap<String, HlId>,
    /// Whether the highlights changed since the last flush
    pub did_highlights_change: bool,
    /// The ID of the current mode
    pub current_mode: u32,
    /// Information about Vim modes, indexed by ID
    pub modes: Vec<ModeInfo>,
    /// UI options set by the option_set event
    pub guifont_update: Option<GuiFont>,
    /// Default highlight colors
    pub default_colors: DefaultColorsSet,
    /// Manages ext_hlstate events
    pub messages: Messages,
    /// Manages ext_cmdline events
    pub cmdline: Cmdline,
    /// Manages ext_popupmenu events
    pub popupmenu: Option<PopupmenuShow>,
    /// Manages ext_tabline events
    pub tabline: Option<TablineUpdate>,
    /// Did we receive a flush event?
    pub did_flush: bool,
    pub ignore_next_scroll: bool,
}

impl Ui {
    pub fn new() -> Self {
        Self::default()
    }

    /// Index of the grid with the given id, or else the index where the
    /// grid should be inserted
    pub fn grid_index(&self, id: grid::Id) -> Result<usize, usize> {
        self.grids.binary_search_by(|probe| probe.id.cmp(&id))
    }

    /// Grid with the given ID
    pub fn grid_mut(&mut self, id: grid::Id) -> Option<&mut Grid> {
        self.grid_index(id)
            .map(|i| self.grids.get_mut(i))
            .ok()
            .flatten()
    }

    /// Grid with the given ID
    pub fn grid(&self, id: grid::Id) -> Option<&Grid> {
        self.grid_index(id)
            .map(|i| self.grids.get(i))
            .ok()
            .flatten()
    }

    /// Get the grid with the given ID or create it if it does not exist
    fn get_or_create_grid(&mut self, id: grid::Id) -> &mut Grid {
        match self.grid_index(id) {
            Ok(i) => &mut self.grids[i],
            Err(i) => {
                self.grids.insert(i, Grid::new(id));
                self.show_normal(id);
                &mut self.grids[i]
            }
        }
    }

    /// Reset dirty flags
    pub fn clear_dirty(&mut self) {
        self.did_highlights_change = false;
        self.did_flush = false;
        self.guifont_update = None;
        self.ignore_next_scroll = false;
        self.messages.dirty = false;
        for grid in self.grids.iter_mut() {
            grid.clear_dirty();
        }
    }

    /// Update the UI with the given event
    pub fn process(&mut self, event: Event) {
        match event {
            Event::OptionSet(event) => match event {
                OptionSet::Guifont(s) if !s.is_empty() => self.guifont_update = Some(s.into()),
                _ => {}
            },
            Event::DefaultColorsSet(event) => {
                self.did_highlights_change = true;
                self.default_colors = event;
            }
            Event::HlAttrDefine(event) => {
                self.did_highlights_change = true;
                let i = event.id as usize;
                if i > self.highlights.len() {
                    self.highlights.resize(i * 2, None);
                }
                self.highlights.insert(i, Some(event.rgb_attr));
            }
            Event::HlGroupSet(HlGroupSet { name, hl_id }) => {
                self.did_highlights_change = true;
                self.highlight_groups.insert(name, hl_id);
            }
            Event::ModeChange(ModeChange { mode_idx, mode: _ }) => self.current_mode = mode_idx,
            Event::ModeInfoSet(ModeInfoSet {
                cursor_style_enabled,
                mode_info,
            }) => {
                self.cursor.style_enabled = cursor_style_enabled;
                self.modes = mode_info;
            }

            Event::GridResize(GridResize {
                grid,
                width,
                height,
            }) => {
                self.get_or_create_grid(grid)
                    .contents_mut()
                    .resize(CellVec(Vec2::new(width, height)));
            }
            Event::GridClear(GridClear { grid }) => self
                .grid_mut(grid)
                .expect("Tried to clear nonexistent grid")
                .contents_mut()
                .clear(),
            Event::GridDestroy(GridDestroy { grid }) => self.delete_grid(grid),
            Event::GridCursorGoto(GridCursorGoto { grid, row, column }) => {
                self.cursor.pos = CellVec::new(column, row);
                self.cursor.grid = grid;
            }
            Event::GridScroll(GridScroll {
                grid,
                top,
                bot,
                left,
                right,
                rows,
                cols: _,
            }) => {
                self.grid_mut(grid)
                    .expect("Tried to scroll nonexistent grid")
                    .contents_mut()
                    .scroll(top, bot, left, right, rows);
            }
            Event::GridLine(GridLine {
                grid,
                row,
                col_start,
                cells,
            }) => {
                self.grid_mut(grid)
                    .expect("Tried to update a line of a nonexistent grid")
                    .contents_mut()
                    .grid_line(row, col_start, cells);
            }

            Event::WinPos(WinPos {
                grid,
                win: _,
                start_row,
                start_col,
                width,
                height,
            }) => {
                self.show_normal(grid);
                *self
                    .grid_mut(grid)
                    .expect("Tried to update the position of a nonexistent grid")
                    .window_mut() = Window::Normal(NormalWindow {
                    start: CellVec(Vec2::new(start_col, start_row)),
                    size: CellVec(Vec2::new(width, height)),
                });
            }
            Event::WinFloatPos(WinFloatPos {
                grid,
                win: _,
                anchor,
                anchor_grid,
                anchor_row,
                anchor_col,
                focusable,
                zindex,
            }) => {
                self.show_float(DrawItem::new(grid, zindex));
                *self
                    .grid_mut(grid)
                    .expect("Tried to update the position of a nonexistent grid")
                    .window_mut() = Window::Floating(FloatingWindow {
                    anchor,
                    focusable,
                    anchor_grid,
                    anchor_pos: CellVec(Vec2::new(anchor_col, anchor_row)),
                });
            }
            Event::WinExternalPos(WinExternalPos { grid, win: _ }) => {
                *self
                    .grid_mut(grid)
                    .expect("Tried to update the position of a nonexistent grid")
                    .window_mut() = Window::External;
            }
            Event::WinHide(WinHide { grid }) => {
                self.hide(grid);
            }
            Event::WinClose(WinClose { grid }) => {
                self.hide(grid);
                // It seems like we shouldn't be able to receive this event
                // when a grid doesn't exist, but I have had this happen when
                // opening DAP UI.
                if let Some(grid) = self.grid_mut(grid) {
                    *grid.window_mut() = Window::None;
                }
            }
            Event::WinViewport(WinViewport {
                grid,
                scroll_delta,
                win: _,
                topline: _,
                botline: _,
                curline: _,
                curcol: _,
                line_count: _,
            }) => {
                if !self.ignore_next_scroll {
                    self.grid_mut(grid)
                        .expect("Tried to update the viewport of a nonexistent grid")
                        .scroll_delta = scroll_delta;
                }
            }
            Event::WinViewportMargins(_) | Event::WinExtmark(_) => {}

            Event::PopupmenuShow(event) => self.popupmenu = Some(event),
            Event::PopupmenuSelect(PopupmenuSelect { selected }) => {
                if let Some(menu) = &mut self.popupmenu {
                    menu.selected = selected
                }
            }
            Event::PopupmenuHide => self.popupmenu = None,

            Event::CmdlineShow(event) => self.cmdline.show(event),
            Event::CmdlinePos(CmdlinePos { pos, level: _ }) => self.cmdline.set_cursor_pos(pos),
            Event::CmdlineBlockShow(CmdlineBlockShow { lines }) => self.cmdline.show_block(lines),
            Event::CmdlineBlockAppend(CmdlineBlockAppend { line }) => {
                self.cmdline.append_block(line)
            }
            Event::CmdlineSpecialChar(event) => self.cmdline.special(event),
            Event::CmdlineHide => self.cmdline.hide(),
            Event::CmdlineBlockHide => self.cmdline.hide_block(),

            Event::MsgHistoryShow(MsgHistoryShow { entries }) => {
                self.messages.history = entries;
                self.messages.dirty = true;
            }
            Event::MsgRuler(MsgRuler { content }) => self.messages.ruler = content,
            Event::MsgSetPos(MsgSetPos {
                grid,
                row,
                scrolled: _,
                sep_char: _,
            }) => {
                // Message scrollback is a hard-coded z-index
                // https://neovim.io/doc/user/api.html#nvim_open_win()
                self.show_float(DrawItem::new(grid, Some(200)));
                *self.get_or_create_grid(grid).window_mut() = Window::Messages { row };
            }
            Event::MsgShow(event) => {
                self.messages.show(event);
                self.messages.dirty = true;
            }
            Event::MsgShowmode(MsgShowmode { content }) => self.messages.showmode = content,
            Event::MsgShowcmd(MsgShowcmd { content }) => self.messages.showcmd = content,
            Event::MsgClear => {
                self.messages.show.clear();
                self.messages.dirty = true;
            }
            Event::MsgHistoryClear => {
                self.messages.history.clear();
            }

            Event::TablineUpdate(event) => self.tabline = Some(event),
            Event::Chdir(Chdir { path }) => match std::env::set_current_dir(path) {
                Ok(_) => {}
                Err(e) => log::error!("Failed to change directory: {e:?}"),
            },

            Event::MouseOn => self.mouse = true,
            Event::MouseOff => self.mouse = false,
            Event::BusyStart => self.cursor.enabled = false,
            Event::BusyStop => self.cursor.enabled = true,
            Event::Flush => self.did_flush = true,

            Event::Suspend
            | Event::SetTitle(_)
            | Event::SetIcon(_)
            | Event::UpdateMenu
            | Event::Bell
            | Event::VisualBell => {}
        }
    }

    /// Move the given grid to the top of the draw order
    fn show_float(&mut self, draw_item: DrawItem) {
        self.hide(draw_item.grid);
        // Default float value is 50
        // https://neovim.io/doc/user/api.html#nvim_open_win()
        let z_of = |item: DrawItem| item.z.unwrap_or(50);
        let z = z_of(draw_item);
        let insert_position = self
            .draw_order
            .iter()
            .enumerate()
            .skip(self.float_windows_start)
            .rev()
            .find_map(|(i, item)| (z >= z_of(*item)).then_some(i + 1))
            .unwrap_or(self.float_windows_start);
        self.draw_order.insert(insert_position, draw_item);
    }

    /// Move the given grid to the top of the draw order
    fn show_normal(&mut self, grid: grid::Id) {
        self.hide(grid);
        self.draw_order
            .insert(self.float_windows_start, DrawItem::new(grid, None));
        self.float_windows_start += 1;
    }

    /// Remove the given grid from the draw order
    fn hide(&mut self, grid: grid::Id) {
        if let Some(i) = self.draw_order.iter().position(|&r| r.grid == grid) {
            self.draw_order.remove(i);
            if i < self.float_windows_start {
                self.float_windows_start -= 1;
            }
        }
    }

    /// Delete the given grid
    fn delete_grid(&mut self, grid: grid::Id) {
        if let Ok(i) = self.grids.binary_search_by(|probe| probe.id.cmp(&grid)) {
            self.grids.remove(i);
        }
        self.hide(grid);
    }

    /// Get the position of the grid, accounting for anchor grids and other
    /// windowing details
    pub fn position(&self, grid: grid::Id) -> Option<CellVec<f32>> {
        if grid == 1 {
            return Some(CellVec::new(0., 0.));
        }
        if let Ok(index) = self.grid_index(grid) {
            let grid = &self.grids[index];
            if grid.window() == &Window::None {
                return None;
            }

            let WindowOffset {
                offset,
                anchor_grid,
            } = grid.window().offset(grid.contents().size);

            let position = if let Some(anchor_grid) = anchor_grid {
                self.position(anchor_grid)? + offset
            } else {
                offset
            };

            match grid.window() {
                Window::Floating(_) => {
                    let base_grid_size = self.grids[0].contents().size;
                    let grid_max = position + grid.contents().size.cast_as();
                    let overflow = (grid_max - base_grid_size.cast_as()).map(|x| x.max(0.));
                    Some((position - overflow).map(|x| x.max(0.)))
                }
                _ => Some(position),
            }
        } else {
            None
        }
    }

    /// The grid under the cursor, accounting for anchor grids and other
    /// windowing details
    pub fn grid_under_cursor(
        &self,
        cursor: PixelVec<u32>,
        cell_size: Vec2<u32>,
    ) -> Option<GridUnderCursor> {
        // TODO: Can this be derived from CursorInfo instead?
        let cursor = cursor.cast_as::<f32>();
        let cell_size = cell_size.cast_as::<f32>();
        for &draw_item in self.draw_order.iter().rev() {
            let grid = self.grid(draw_item.grid).unwrap();
            let size: CellVec<f32> = grid.contents().size.cast_as();
            let start = self.position(draw_item.grid)?.into_pixels(cell_size);
            let end = start + size.into_pixels(cell_size);
            if cursor.0.x > start.0.x
                && cursor.0.y > start.0.y
                && cursor.0.x < end.0.x
                && cursor.0.y < end.0.y
            {
                let position = (cursor - start).into_cells(cell_size);
                let position = position.cast_as::<i64>();
                let Ok(position) = position.try_cast() else {
                    continue;
                };
                return Some(GridUnderCursor {
                    grid: draw_item.grid,
                    position,
                });
            }
        }
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridUnderCursor {
    /// The ID of the grid
    pub grid: grid::Id,
    /// The position of the cursor in grid cells relative to the grid
    pub position: CellVec<u32>,
}

#[derive(Debug, Copy, Clone)]
pub struct CursorInfo {
    /// The position of the cursor in grid cells
    pub pos: CellVec<u16>,
    /// The grid the cursor is on
    pub grid: grid::Id,
    /// Whether the cursor should be rendered
    pub enabled: bool,
    /// Whether the UI should set the cursor style
    pub style_enabled: bool,
}

impl Default for CursorInfo {
    fn default() -> Self {
        Self {
            pos: Default::default(),
            grid: 1,
            enabled: true,
            style_enabled: false,
        }
    }
}
