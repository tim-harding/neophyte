mod cmdline;
pub mod grid;
mod messages;
mod options;
mod print;

use self::{cmdline::Cmdline, grid::CursorRenderInfo, messages::Messages, options::Options};
use crate::{
    event::{
        mode_info_set::ModeInfo, Anchor, DefaultColorsSet, Event, GlobalEvent, GridLine,
        GridScroll, HlAttrDefine, PopupmenuShow, TablineUpdate, WinFloatPos, WinPos,
    },
    ui::grid::{FloatingWindow, Window},
    util::vec2::Vec2,
};
use grid::Grid;
use std::collections::HashMap;

pub type Highlights = HashMap<u64, HlAttrDefine>;
pub type HighlightGroups = HashMap<String, u64>;

#[derive(Debug, Clone)]
pub struct Ui {
    pub title: String,
    pub icon: String,
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
    pub mode_for_hl_id: HashMap<u64, usize>,
    pub mode_for_langmap: HashMap<u64, usize>,
    pub options: Options,
    pub default_colors: DefaultColorsSet,
    pub tabline: Option<TablineUpdate>,
}

#[derive(Debug, Copy, Clone)]
pub struct CursorInfo {
    pos: Vec2<u64>,
    grid: u64,
    enabled: bool,
    style_enabled: bool,
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
            title: Default::default(),
            icon: Default::default(),
            grids: Default::default(),
            draw_order: vec![],
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
            mode_for_hl_id: Default::default(),
            mode_for_langmap: Default::default(),
            options: Default::default(),
            default_colors: Default::default(),
            tabline: Default::default(),
        }
    }

    fn grid_mut(&mut self, id: u64) -> &mut Grid {
        let i = self
            .grids
            .binary_search_by(|probe| probe.id.cmp(&id))
            .unwrap();
        self.grids.get_mut(i).unwrap()
    }

    pub fn grid(&self, id: u64) -> &Grid {
        let i = self
            .grids
            .binary_search_by(|probe| probe.id.cmp(&id))
            .unwrap();
        self.grids.get(i).unwrap()
    }

    fn get_or_create_grid(&mut self, id: u64) -> &mut Grid {
        match self.grids.binary_search_by(|probe| probe.id.cmp(&id)) {
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
            Event::SetTitle(event) => self.title = event.title,
            Event::SetIcon(event) => self.icon = event.icon,
            Event::OptionSet(event) => self.options.event(event),
            Event::DefaultColorsSet(event) => self.default_colors = event,
            Event::HlAttrDefine(event) => {
                self.new_highlights.push(event.id);
                self.highlights.insert(event.id, event);
            }
            Event::ModeChange(event) => self.current_mode = event.mode_idx,
            Event::ModeInfoSet(event) => {
                self.cursor.style_enabled = event.cursor_style_enabled;
                // TODO: Factor out the mode management and provide a getter for
                // the highlight attribute. Some rules require having options
                // from Neovim.
                for (i, info) in event.mode_info.iter().enumerate() {
                    if let Some(attr_id) = info.attr_id {
                        self.mode_for_hl_id.insert(attr_id, i);
                    }
                    if let Some(attr_id_lm) = info.attr_id_lm {
                        self.mode_for_langmap.insert(attr_id_lm, i);
                    }
                }
                self.modes = event.mode_info;
            }
            Event::HlGroupSet(event) => {
                self.highlight_groups.insert(event.name, event.hl_id);
            }

            Event::GridResize(event) => {
                let grid = self.get_or_create_grid(event.grid);
                grid.resize(Vec2::new(event.width, event.height));
            }
            Event::GridClear(event) => self.grid_mut(event.grid).clear(),
            Event::GridDestroy(event) => self.delete_grid(event.grid),
            Event::GridCursorGoto(event) => {
                self.cursor.pos = Vec2::new(event.column, event.row);
                self.cursor.grid = event.grid;
            }
            Event::GridScroll(event) => {
                let GridScroll {
                    grid,
                    top,
                    bot,
                    left,
                    right,
                    rows,
                    cols: _,
                } = event;
                let grid = self.grid_mut(grid);
                grid.scroll(top, bot, left, right, rows);
            }
            Event::GridLine(event) => {
                let GridLine {
                    grid,
                    row,
                    col_start,
                    cells,
                } = event;
                let grid = self.grid_mut(grid);
                grid.grid_line(row, col_start, cells);
            }

            Event::WinPos(event) => {
                let WinPos {
                    grid,
                    win: _,
                    start_row,
                    start_col,
                    width,
                    height,
                } = event;
                match self.draw_order.iter().position(|&r| r == grid) {
                    Some(i) => {
                        self.draw_order.remove(i);
                    }
                    None => {}
                }
                self.draw_order.push(grid);
                self.float_windows_start += 1;
                let grid = self.grid_mut(grid);
                grid.window = Window::Normal(grid::NormalWindow {
                    start: Vec2::new(start_col, start_row),
                    size: Vec2::new(width, height),
                });
            }
            Event::WinFloatPos(event) => {
                let WinFloatPos {
                    grid,
                    win: _,
                    anchor,
                    anchor_grid,
                    anchor_row,
                    anchor_col,
                    focusable,
                } = event;
                match self.draw_order.iter().position(|&r| r == grid) {
                    Some(i) => {
                        self.draw_order.remove(i);
                    }
                    None => {}
                }
                self.draw_order.push(grid);
                let grid = self.grid_mut(grid);
                grid.window = Window::Floating(FloatingWindow {
                    anchor,
                    focusable,
                    anchor_grid,
                    anchor_pos: Vec2::new(anchor_col, anchor_row),
                })
            }
            Event::WinExternalPos(event) => {
                let grid = self.grid_mut(event.grid);
                grid.window = Window::External;
            }
            Event::WinHide(event) => {
                if let Some(i) = self.draw_order.iter().position(|&r| r == event.grid) {
                    self.draw_order.remove(i);
                }
                let grid = self.grid_mut(event.grid);
                grid.show = false;
            }
            Event::WinClose(event) => {
                if let Some(i) = self.draw_order.iter().position(|&r| r == event.grid) {
                    self.draw_order.remove(i);
                }
                let grid = self.grid_mut(event.grid);
                grid.window = Window::None;
            }
            Event::WinViewport(_) => {} // For smooth scrolling
            Event::WinExtmark(_) => {}  // Ignored

            Event::PopupmenuShow(event) => self.popupmenu = Some(event),
            Event::PopupmenuSelect(event) => {
                if let Some(menu) = &mut self.popupmenu {
                    menu.selected = event.selected
                }
            }
            Event::GlobalEvent(GlobalEvent::PopupmenuHide) => self.popupmenu = None,

            Event::CmdlineShow(event) => self.cmdline.show(event),
            Event::CmdlinePos(event) => self.cmdline.set_cursor_pos(event.pos),
            Event::CmdlineBlockShow(event) => self.cmdline.show_block(event.lines),
            Event::CmdlineBlockAppend(event) => self.cmdline.append_block(event.line),
            Event::CmdlineSpecialChar(event) => self.cmdline.special(event),
            Event::GlobalEvent(GlobalEvent::CmdlineHide) => self.cmdline.hide(),
            Event::GlobalEvent(GlobalEvent::CmdlineBlockHide) => self.cmdline.hide_block(),

            Event::MsgHistoryShow(event) => self.messages.history = event.entries,
            Event::MsgRuler(event) => self.messages.ruler = event.content,
            Event::MsgSetPos(event) => {
                let grid = self.get_or_create_grid(event.grid);
                grid.window = Window::Floating(FloatingWindow {
                    anchor: Anchor::Nw,
                    anchor_grid: 1,
                    anchor_pos: Vec2::new(0.0, event.row as f64),
                    focusable: false,
                });
            }
            Event::MsgShow(event) => self.messages.show(event),
            Event::MsgShowmode(event) => self.messages.showmode = event.content,
            Event::MsgShowcmd(event) => self.messages.showcmd = event.content,
            Event::GlobalEvent(GlobalEvent::MsgClear) => self.messages.show.clear(),
            Event::GlobalEvent(GlobalEvent::MsgHistoryClear) => self.messages.history.clear(),

            Event::TablineUpdate(event) => self.tabline = Some(event),

            Event::GlobalEvent(GlobalEvent::MouseOn) => self.cursor.enabled = true,
            Event::GlobalEvent(GlobalEvent::MouseOff) => self.cursor.enabled = false,
            Event::GlobalEvent(GlobalEvent::BusyStart) => self.cursor.enabled = false,
            Event::GlobalEvent(GlobalEvent::BusyStop) => self.cursor.enabled = true,
            Event::GlobalEvent(GlobalEvent::Flush) => self.new_highlights.clear(),

            Event::GlobalEvent(GlobalEvent::Suspend)
            | Event::GlobalEvent(GlobalEvent::UpdateMenu)
            | Event::GlobalEvent(GlobalEvent::Bell)
            | Event::GlobalEvent(GlobalEvent::VisualBell) => {}
        }
    }

    fn delete_grid(&mut self, grid: u64) {
        match self.grids.binary_search_by(|probe| probe.id.cmp(&grid)) {
            Ok(i) => {
                self.grids.remove(i);
            }
            Err(_) => {}
        }
        match self.draw_order.iter().position(|&r| r == grid) {
            Some(i) => {
                self.draw_order.remove(i);
            }
            None => {}
        }
    }

    pub fn composite(&self) -> Grid {
        let mut outer_grid = self.grids.get(0).unwrap_or(&Grid::default()).clone();
        for grid in self.draw_order.iter() {
            let grid = self.grid(*grid);
            outer_grid.combine(grid.clone(), self.cursor_render_info(grid.id));
        }
        outer_grid
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
}
