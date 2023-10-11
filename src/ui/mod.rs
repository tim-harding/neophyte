mod cmdline;
mod double_buffer;
pub mod grid;
mod messages;
mod options;
pub mod packed_char;
pub mod window;

use self::{
    cmdline::Cmdline, grid::DoubleBufferGrid, messages::Messages, options::Options,
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
use std::{collections::HashMap, fmt::Debug};

pub use double_buffer::DoubleBuffer;
pub use options::{FontSize, FontsSetting};

pub type HighlightGroups = HashMap<String, u64>;

pub struct Ui {
    // TODO: Probably should privatize
    pub grids: Vec<DoubleBufferGrid>,
    pub draw_order: Vec<u64>,
    pub float_windows_start: usize,
    pub cursor: CursorInfo,
    pub mouse: bool,
    pub highlights: Vec<HlAttrDefine>,
    pub highlight_groups: HighlightGroups,
    pub did_highlights_change: bool,
    pub messages: Messages,
    pub cmdline: Cmdline,
    pub popupmenu: Option<PopupmenuShow>,
    pub current_mode: u64,
    pub modes: Vec<ModeInfo>,
    pub options: Options,
    pub default_colors: DefaultColorsSet,
    pub tabline: Option<TablineUpdate>,
    pub did_flush: bool,
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            grids: Default::default(),
            draw_order: vec![1],
            float_windows_start: 0,
            cursor: Default::default(),
            mouse: Default::default(),
            highlights: Default::default(),
            highlight_groups: Default::default(),
            did_highlights_change: false,
            messages: Default::default(),
            cmdline: Default::default(),
            popupmenu: Default::default(),
            current_mode: Default::default(),
            modes: Default::default(),
            options: Default::default(),
            default_colors: Default::default(),
            tabline: Default::default(),
            did_flush: false,
        }
    }
}

impl Ui {
    pub fn grid_index(&self, id: u64) -> Result<usize, usize> {
        self.grids.binary_search_by(|probe| probe.id.cmp(&id))
    }

    pub fn grid_mut(&mut self, id: u64) -> Option<&mut DoubleBufferGrid> {
        self.grid_index(id)
            .map(|i| self.grids.get_mut(i).unwrap())
            .ok()
    }

    pub fn grid(&self, id: u64) -> Option<&DoubleBufferGrid> {
        self.grid_index(id).map(|i| self.grids.get(i).unwrap()).ok()
    }

    fn get_or_create_grid(&mut self, id: u64) -> &mut DoubleBufferGrid {
        match self.grid_index(id) {
            Ok(i) => self.grids.get_mut(i).unwrap(),
            Err(i) => {
                self.grids.insert(i, DoubleBufferGrid::new(id));
                self.grids.get_mut(i).unwrap()
            }
        }
    }

    pub fn process(&mut self, event: Event) {
        log::info!("{event:?}");
        if self.did_flush {
            self.did_flush = false;
            self.did_highlights_change = false;
            for grid in self.grids.iter_mut() {
                grid.flush();
            }
        }

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
                    .current_mut()
                    .resize(Vec2::new(width, height));
            }
            Event::GridClear(GridClear { grid }) => {
                self.grid_mut(grid).unwrap().current_mut().clear()
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
                    .current_mut()
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
                    .current_mut()
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

            Event::MouseOn => self.cursor.enabled = true,
            Event::MouseOff => self.cursor.enabled = false,
            Event::BusyStart => self.cursor.enabled = false,
            Event::BusyStop => self.cursor.enabled = true,
            Event::Flush => {
                self.did_flush = true;
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

    pub fn position(&self, grid: u64) -> Vec2<f64> {
        if let Ok(index) = self.grid_index(grid) {
            let grid = &self.grids[index];
            let WindowOffset {
                offset,
                anchor_grid,
            } = grid.window().offset(grid.current().size);
            if let Some(anchor_grid) = anchor_grid {
                self.position(anchor_grid) + offset
            } else {
                offset
            }
        } else {
            Vec2::default()
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
            let size: Vec2<f64> = grid.current().size.cast_as();
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
    pub grid: u64,
    pub position: Vec2<u64>,
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
