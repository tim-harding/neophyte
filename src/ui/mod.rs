mod cmdline;
pub mod grid;
mod messages;
pub mod options;
pub mod packed_char;
pub mod window;

use self::{
    cmdline::Cmdline, grid::Grid, messages::Messages, options::Options, window::WindowOffset,
};
use crate::{
    event::{
        mode_info_set::ModeInfo, Anchor, CmdlineBlockAppend, CmdlineBlockShow, CmdlinePos,
        DefaultColorsSet, Event, GridClear, GridCursorGoto, GridDestroy, GridLine, GridResize,
        GridScroll, HlAttrDefine, HlGroupSet, ModeChange, ModeInfoSet, MsgHistoryShow, MsgRuler,
        MsgSetPos, MsgShowcmd, MsgShowmode, PopupmenuSelect, PopupmenuShow, TablineUpdate,
        WinClose, WinExternalPos, WinFloatPos, WinHide, WinPos, WinViewport,
    },
    ui::window::{FloatingWindow, NormalWindow, Window},
    util::vec2::Vec2,
};
use std::{collections::HashMap, fmt::Debug};

/// Manages updates to the UI state from UI events
#[derive(Clone)]
pub struct Ui {
    /// UI grids, ordered by ID
    pub grids: Vec<Grid>,
    /// Grids that have been deleted since the last flush
    pub deleted_grids: Vec<u64>,
    /// The order in which grids should be drawn, ordered from bottom to top
    pub draw_order: Vec<u64>,
    /// The index into self.draw_order at which floating windows begin
    pub float_windows_start: usize,
    /// Cursor information
    pub cursor: CursorInfo,
    /// Whether the mouse is enabled
    pub mouse: bool,
    /// UI highlights, indexed by their ID
    pub highlights: Vec<HlAttrDefine>,
    /// A lookup from highlight names to highlight IDs
    pub highlight_groups: HashMap<String, u64>,
    /// Whether the highlights changed since the last flush
    pub did_highlights_change: bool,
    /// The ID of the current mode
    pub current_mode: u64,
    /// Information about Vim modes, indexed by ID
    pub modes: Vec<ModeInfo>,
    /// UI options set by the option_set event
    pub options: Options,
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
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            grids: vec![],
            deleted_grids: vec![],
            draw_order: vec![1],
            float_windows_start: 0,
            cursor: Default::default(),
            mouse: false,
            highlights: vec![],
            highlight_groups: HashMap::new(),
            did_highlights_change: false,
            current_mode: 0,
            modes: vec![],
            options: Default::default(),
            default_colors: Default::default(),
            messages: Default::default(),
            cmdline: Default::default(),
            popupmenu: None,
            tabline: None,
        }
    }
}

impl Ui {
    pub fn new() -> Self {
        Self::default()
    }

    /// Index of the grid with the given id, or else the index where the
    /// grid should be inserted
    pub fn grid_index(&self, id: u64) -> Result<usize, usize> {
        self.grids.binary_search_by(|probe| probe.id.cmp(&id))
    }

    /// Grid with the given ID
    pub fn grid_mut(&mut self, id: u64) -> Option<&mut Grid> {
        self.grid_index(id)
            .map(|i| self.grids.get_mut(i).unwrap())
            .ok()
    }

    /// Grid with the given ID
    pub fn grid(&self, id: u64) -> Option<&Grid> {
        self.grid_index(id).map(|i| self.grids.get(i).unwrap()).ok()
    }

    /// Get the grid with the given ID or create it if it does not exist
    fn get_or_create_grid(&mut self, id: u64) -> &mut Grid {
        match self.grid_index(id) {
            Ok(i) => self.grids.get_mut(i).unwrap(),
            Err(i) => {
                self.grids.insert(i, Grid::new(id));
                self.grids.get_mut(i).unwrap()
            }
        }
    }

    /// Reset dirty flags
    pub fn clear_dirty(&mut self) {
        self.did_highlights_change = false;
        self.deleted_grids.clear();
        for grid in self.grids.iter_mut() {
            grid.clear_dirty();
        }
    }

    /// Update the UI with the given event
    pub fn process(&mut self, event: Event) {
        log::info!("{event:?}");
        match event {
            Event::OptionSet(event) => self.options.event(event),
            Event::DefaultColorsSet(event) => {
                self.did_highlights_change = true;
                self.default_colors = event;
            }
            Event::HlAttrDefine(event) => {
                self.did_highlights_change = true;
                let i = event.id as usize;
                if i > self.highlights.len() {
                    self.highlights.resize(i * 2, HlAttrDefine::default());
                }
                self.highlights.insert(i, event);
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
                    .resize(Vec2::new(width, height));
            }
            Event::GridClear(GridClear { grid }) => {
                self.grid_mut(grid).unwrap().contents_mut().clear()
            }
            Event::GridDestroy(GridDestroy { grid }) => self.delete_grid(grid),
            Event::GridCursorGoto(GridCursorGoto { grid, row, column }) => {
                self.cursor.pos = Vec2::new(column, row);
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
                    .unwrap()
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
                    .unwrap()
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
                self.show(grid);
                self.float_windows_start += 1;
                *self.grid_mut(grid).unwrap().window_mut() = Window::Normal(NormalWindow {
                    start: Vec2::new(start_col, start_row),
                    size: Vec2::new(width, height),
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
            }) => {
                self.show(grid);
                *self.grid_mut(grid).unwrap().window_mut() = Window::Floating(FloatingWindow {
                    anchor,
                    focusable,
                    anchor_grid,
                    anchor_pos: Vec2::new(anchor_col, anchor_row),
                });
            }
            Event::WinExternalPos(WinExternalPos { grid, win: _ }) => {
                *self.grid_mut(grid).unwrap().window_mut() = Window::External;
            }
            Event::WinHide(WinHide { grid }) => self.hide(grid),
            Event::WinClose(WinClose { grid }) => {
                self.hide(grid);
                *self.grid_mut(grid).unwrap().window_mut() = Window::None;
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
                self.grid_mut(grid).unwrap().scroll_delta = scroll_delta;
            }
            Event::WinExtmark(_) => {}

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

            Event::MsgHistoryShow(MsgHistoryShow { entries }) => self.messages.history = entries,
            Event::MsgRuler(MsgRuler { content }) => self.messages.ruler = content,
            Event::MsgSetPos(MsgSetPos {
                grid,
                row,
                scrolled: _,
                sep_char: _,
            }) => {
                self.show(grid);
                *self.get_or_create_grid(grid).window_mut() = Window::Floating(FloatingWindow {
                    anchor: Anchor::Nw,
                    anchor_grid: 1,
                    anchor_pos: Vec2::new(0.0, row as f64),
                    focusable: false,
                });
            }
            Event::MsgShow(event) => self.messages.show(event),
            Event::MsgShowmode(MsgShowmode { content }) => self.messages.showmode = content,
            Event::MsgShowcmd(MsgShowcmd { content }) => self.messages.showcmd = content,
            Event::MsgClear => self.messages.show.clear(),
            Event::MsgHistoryClear => self.messages.history.clear(),

            Event::TablineUpdate(event) => self.tabline = Some(event),

            Event::MouseOn => self.mouse = true,
            Event::MouseOff => self.mouse = false,
            Event::BusyStart => self.cursor.enabled = false,
            Event::BusyStop => self.cursor.enabled = true,
            Event::Flush => {}

            Event::Suspend
            | Event::SetTitle(_)
            | Event::SetIcon(_)
            | Event::UpdateMenu
            | Event::Bell
            | Event::VisualBell => {}
        }
    }

    /// Move the given grid to the top of the draw order
    fn show(&mut self, grid: u64) {
        if let Some(i) = self.draw_order.iter().position(|&r| r == grid) {
            self.draw_order.remove(i);
        }
        self.draw_order.push(grid);
    }

    /// Remove the given grid from the draw order
    fn hide(&mut self, grid: u64) {
        if let Some(i) = self.draw_order.iter().position(|&r| r == grid) {
            self.draw_order.remove(i);
        }
    }

    /// Delete the given grid
    fn delete_grid(&mut self, grid: u64) {
        if let Ok(i) = self.grids.binary_search_by(|probe| probe.id.cmp(&grid)) {
            self.grids.remove(i);
        }
        if let Some(i) = self.draw_order.iter().position(|&r| r == grid) {
            self.draw_order.remove(i);
        }
        self.deleted_grids.push(grid);
    }

    /// Get the position of the grid, accounting for anchor grids and other
    /// windowing details
    pub fn position(&self, grid: u64) -> Vec2<f64> {
        if let Ok(index) = self.grid_index(grid) {
            let grid = &self.grids[index];
            let WindowOffset {
                offset,
                anchor_grid,
            } = grid.window().offset(grid.contents().size);
            let position = if let Some(anchor_grid) = anchor_grid {
                self.position(anchor_grid) + offset
            } else {
                offset
            };
            position.map(|x| x.max(0.))
        } else {
            Vec2::default()
        }
    }

    /// The grid under the cursor, accounting for anchor grids and other
    /// windowing details
    pub fn grid_under_cursor(
        &self,
        cursor: Vec2<u64>,
        cell_size: Vec2<u64>,
    ) -> Option<GridUnderCursor> {
        // TODO: Can this be derived from CursorInfo instead?
        let cursor: Vec2<f64> = cursor.cast_as();
        let cell_size: Vec2<f64> = cell_size.cast_as();
        for &grid_id in self.draw_order.iter().rev() {
            let grid = self.grid(grid_id).unwrap();
            let size: Vec2<f64> = grid.contents().size.cast_as();
            let start = self.position(grid_id) * cell_size;
            let end = start + size * cell_size;
            if cursor.x > start.x && cursor.y > start.y && cursor.x < end.x && cursor.y < end.y {
                let position = (cursor - start) / cell_size;
                let position: Vec2<i64> = position.cast_as();
                let Ok(position) = position.try_cast() else {
                    continue;
                };
                return Some(GridUnderCursor {
                    grid: grid_id,
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
    pub grid: u64,
    /// The position of the cursor in grid cells relative to the grid
    pub position: Vec2<u64>,
}

#[derive(Debug, Copy, Clone)]
pub struct CursorInfo {
    /// The position of the cursor in grid cells
    pub pos: Vec2<u64>,
    /// The grid the cursor is on
    pub grid: u64,
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
