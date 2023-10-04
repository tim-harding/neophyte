mod cmdline;
pub mod grid;
mod messages;
mod options;
pub mod window;

use self::{
    cmdline::Cmdline, grid::CursorRenderInfo, messages::Messages, options::Options,
    window::WindowOffset,
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
use grid::Grid;
use std::{
    collections::HashMap,
    fmt::{self, Debug, Formatter},
};

pub use options::{FontSize, FontsSetting};

pub type Highlights = HashMap<u64, HlAttrDefine>;
pub type HighlightGroups = HashMap<String, u64>;

#[derive(Clone)]
pub struct Ui {
    pub grids: Vec<Grid>,
    pub draw_order: Vec<u64>,
    pub float_windows_start: usize,
    pub cursor: CursorInfo,
    pub mouse: bool,
    pub highlights: Highlights,
    pub new_highlights: Vec<u64>,
    pub highlight_groups: HighlightGroups,
    pub messages: Messages,
    pub cmdline: Cmdline,
    pub popupmenu: Option<PopupmenuShow>,
    pub current_mode: u64,
    pub modes: Vec<ModeInfo>,
    pub options: Options,
    pub default_colors: DefaultColorsSet,
    pub tabline: Option<TablineUpdate>,
}

#[derive(Debug, Copy, Clone)]
pub struct CursorInfo {
    pub pos: Vec2<u64>,
    pub grid: u64,
    pub enabled: bool,
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

impl Ui {
    pub fn new() -> Self {
        Self {
            grids: Default::default(),
            draw_order: vec![1],
            float_windows_start: 0,
            cursor: Default::default(),
            mouse: Default::default(),
            highlights: Default::default(),
            new_highlights: vec![],
            highlight_groups: Default::default(),
            messages: Default::default(),
            cmdline: Default::default(),
            popupmenu: Default::default(),
            current_mode: Default::default(),
            modes: Default::default(),
            options: Default::default(),
            default_colors: Default::default(),
            tabline: Default::default(),
        }
    }

    pub fn grid_index(&self, id: u64) -> Result<usize, usize> {
        self.grids.binary_search_by(|probe| probe.id.cmp(&id))
    }

    pub fn grid_mut(&mut self, id: u64) -> &mut Grid {
        let i = self.grid_index(id).unwrap();
        self.grids.get_mut(i).unwrap()
    }

    pub fn grid(&self, id: u64) -> Option<&Grid> {
        self.grid_index(id).map(|i| self.grids.get(i).unwrap()).ok()
    }

    fn get_or_create_grid(&mut self, id: u64) -> &mut Grid {
        match self.grid_index(id) {
            Ok(i) => self.grids.get_mut(i).unwrap(),
            Err(i) => {
                self.grids.insert(i, Grid::new(id));
                self.grids.get_mut(i).unwrap()
            }
        }
    }

    pub fn process(&mut self, event: Event) {
        log::info!("{event:?}");
        match event {
            Event::OptionSet(event) => self.options.event(event),
            Event::DefaultColorsSet(event) => self.default_colors = event,
            Event::HlAttrDefine(event) => {
                self.new_highlights.push(event.id);
                self.highlights.insert(event.id, event);
            }
            Event::ModeChange(ModeChange { mode_idx, mode: _ }) => self.current_mode = mode_idx,
            Event::ModeInfoSet(ModeInfoSet {
                cursor_style_enabled,
                mode_info,
            }) => {
                self.cursor.style_enabled = cursor_style_enabled;
                self.modes = mode_info;
            }
            Event::HlGroupSet(HlGroupSet { name, hl_id }) => {
                self.highlight_groups.insert(name, hl_id);
            }

            Event::GridResize(GridResize {
                grid,
                width,
                height,
            }) => {
                let grid = self.get_or_create_grid(grid);
                grid.resize(Vec2::new(width, height));
            }
            Event::GridClear(GridClear { grid }) => self.grid_mut(grid).clear(),
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
                let grid = self.grid_mut(grid);
                grid.scroll(top, bot, left, right, rows);
            }
            Event::GridLine(GridLine {
                grid,
                row,
                col_start,
                cells,
            }) => {
                let grid = self.grid_mut(grid);
                grid.grid_line(row, col_start, cells);
            }

            Event::WinPos(WinPos {
                grid,
                win: _,
                start_row,
                start_col,
                width,
                height,
            }) => {
                if let Some(i) = self.draw_order.iter().position(|&r| r == grid) {
                    self.draw_order.remove(i);
                }
                self.draw_order.push(grid);
                self.float_windows_start += 1;
                let grid = self.grid_mut(grid);
                grid.window = Window::Normal(NormalWindow {
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
                let grid = self.grid_mut(grid);
                grid.window = Window::Floating(FloatingWindow {
                    anchor,
                    focusable,
                    anchor_grid,
                    anchor_pos: Vec2::new(anchor_col, anchor_row),
                })
            }
            Event::WinExternalPos(WinExternalPos { grid, win: _ }) => {
                let grid = self.grid_mut(grid);
                grid.window = Window::External;
            }
            Event::WinHide(WinHide { grid }) => self.hide(grid),
            Event::WinClose(WinClose { grid }) => {
                self.hide(grid);
                let grid = self.grid_mut(grid);
                grid.window = Window::None;
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
                let grid = self.grid_mut(grid);
                grid.scroll_delta = scroll_delta;
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
                let grid = self.get_or_create_grid(grid);
                grid.window = Window::Floating(FloatingWindow {
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

            Event::MouseOn => self.cursor.enabled = true,
            Event::MouseOff => self.cursor.enabled = false,
            Event::BusyStart => self.cursor.enabled = false,
            Event::BusyStop => self.cursor.enabled = true,
            Event::Flush => {
                self.new_highlights.clear();
                for grid in self.grids.iter_mut() {
                    grid.dirty = false;
                }
            }

            Event::Suspend
            | Event::SetTitle(_)
            | Event::SetIcon(_)
            | Event::UpdateMenu
            | Event::Bell
            | Event::VisualBell => {}
        }
    }

    fn show(&mut self, grid: u64) {
        if let Some(i) = self.draw_order.iter().position(|&r| r == grid) {
            self.draw_order.remove(i);
        }
        self.draw_order.push(grid);
    }

    fn hide(&mut self, grid: u64) {
        if let Some(i) = self.draw_order.iter().position(|&r| r == grid) {
            self.draw_order.remove(i);
        }
    }

    fn delete_grid(&mut self, grid: u64) {
        if let Ok(i) = self.grids.binary_search_by(|probe| probe.id.cmp(&grid)) {
            self.grids.remove(i);
        }
        if let Some(i) = self.draw_order.iter().position(|&r| r == grid) {
            self.draw_order.remove(i);
        }
    }

    pub fn composite(&self) -> Grid {
        let mut outer_grid = self.grids.get(0).unwrap_or(&Grid::default()).clone();
        for &grid in self.draw_order.iter() {
            let position = self.position(grid).cast_as();
            let grid = self.grid(grid).unwrap();
            outer_grid.combine(grid.clone(), self.cursor_render_info(grid.id), position);
        }
        outer_grid
    }

    pub fn position(&self, grid: u64) -> Vec2<f64> {
        if let Ok(index) = self.grid_index(grid) {
            let grid = &self.grids[index];
            let WindowOffset {
                offset,
                anchor_grid,
            } = grid.offset();
            if let Some(anchor_grid) = anchor_grid {
                self.position(anchor_grid) + offset
            } else {
                offset
            }
        } else {
            Vec2::default()
        }
    }

    fn cursor_render_info(&self, grid: u64) -> Option<CursorRenderInfo> {
        if self.cursor.enabled && grid == self.cursor.grid {
            self.highlight_groups
                .get("Cursor")
                .map(|&hl| CursorRenderInfo {
                    hl,
                    pos: self.cursor.pos,
                })
        } else {
            None
        }
    }

    pub fn grid_under_cursor(
        &self,
        cursor: Vec2<u64>,
        cell_size: Vec2<u64>,
    ) -> Option<GridUnderCursor> {
        let cursor: Vec2<f64> = cursor.cast_as();
        let cell_size: Vec2<f64> = cell_size.cast_as();
        for &grid_id in self.draw_order.iter().rev() {
            let grid = self.grid(grid_id).unwrap();
            let size: Vec2<f64> = grid.size.cast_as();
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

impl Debug for Ui {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let grid = self.composite();
        write!(f, "{:?}", grid)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridUnderCursor {
    pub grid: u64,
    pub position: Vec2<u64>,
}
