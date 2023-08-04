mod cmdline;
mod grid;
mod messages;
mod options;
mod print;

use self::{cmdline::Cmdline, grid::CursorRenderInfo, messages::Messages, options::Options};
use crate::{
    event::{
        mode_info_set::ModeInfo, DefaultColorsSet, Event, GlobalEvent, GridLine, GridScroll,
        HlAttrDefine, PopupmenuShow, WinFloatPos, WinPos,
    },
    ui::grid::{FloatingWindow, Window},
    util::{Vec2, Vec2f},
};
use grid::Grid;
use std::collections::HashMap;

pub type Highlights = HashMap<u64, HlAttrDefine>;
pub type HighlightGroups = HashMap<String, u64>;

#[derive(Debug, Default, Clone)]
pub struct Ui {
    title: String,
    icon: String,
    grids: HashMap<u64, Grid>,
    cursor: CursorInfo,
    #[allow(unused)]
    mouse: bool,
    highlights: Highlights,
    highlight_groups: HighlightGroups,
    messages: Messages,
    cmdline: Cmdline,
    popupmenu: Option<PopupmenuShow>,
    current_mode: u64,
    modes: Vec<ModeInfo>,
    mode_for_hl_id: HashMap<u64, usize>,
    mode_for_langmap: HashMap<u64, usize>,
    options: Options,
    default_colors: DefaultColorsSet,
}

#[derive(Debug, Copy, Clone)]
struct CursorInfo {
    pos: Vec2,
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
        Self::default()
    }

    fn grid(&mut self, grid: u64) -> &mut Grid {
        self.grids.entry(grid).or_default()
    }

    pub fn process(&mut self, event: Event) {
        log::info!("{event:?}");
        match event {
            Event::SetTitle(event) => self.title = event.title,
            Event::SetIcon(event) => self.icon = event.icon,
            Event::OptionSet(event) => self.options.event(event),
            Event::DefaultColorsSet(event) => self.default_colors = event,
            Event::HlAttrDefine(event) => {
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
                let grid = self.grid(event.grid);
                grid.resize(Vec2::new(event.width, event.height));
            }
            Event::GridClear(event) => self.grid(event.grid).clear(),
            Event::GridDestroy(event) => {
                self.grids.remove(&event.grid);
            }
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
                let grid = self.grid(grid);
                grid.scroll(top, bot, left, right, rows);
            }
            Event::GridLine(event) => {
                let GridLine {
                    grid,
                    row,
                    col_start,
                    cells,
                } = event;
                let grid = self.grid(grid);
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
                if let Some(grid) = self.grids.get_mut(&grid) {
                    grid.window = Window::Normal(grid::NormalWindow {
                        start: Vec2::new(start_col, start_row),
                        size: Vec2::new(width, height),
                    });
                }
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
                if let Some(grid) = self.grids.get_mut(&grid) {
                    grid.window = Window::Floating(FloatingWindow {
                        anchor,
                        focusable,
                        anchor_grid,
                        anchor_pos: Vec2f::new(anchor_col, anchor_row),
                    })
                }
            }
            Event::WinExternalPos(event) => {
                if let Some(grid) = self.grids.get_mut(&event.grid) {
                    grid.window = Window::External;
                }
            }
            Event::WinHide(event) => {
                if let Some(grid) = self.grids.get_mut(&event.grid) {
                    grid.show = false;
                }
            }
            Event::WinClose(event) => {
                self.grids.remove(&event.grid);
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
            Event::MsgSetPos(_) => {} // Not used when ui-messages is enabled
            Event::MsgShow(event) => self.messages.show(event),
            Event::MsgShowmode(event) => self.messages.showmode = event.content,
            Event::MsgShowcmd(event) => self.messages.showcmd = event.content,
            Event::GlobalEvent(GlobalEvent::MsgClear) => self.messages.show.clear(),
            Event::GlobalEvent(GlobalEvent::MsgHistoryClear) => self.messages.history.clear(),

            Event::TablineUpdate(_) => {}

            Event::GlobalEvent(GlobalEvent::MouseOn) => self.cursor.enabled = true,
            Event::GlobalEvent(GlobalEvent::MouseOff) => self.cursor.enabled = false,
            Event::GlobalEvent(GlobalEvent::BusyStart) => self.cursor.enabled = false,
            Event::GlobalEvent(GlobalEvent::BusyStop) => self.cursor.enabled = true,
            Event::GlobalEvent(GlobalEvent::Flush) => {
                let mut outer_grid = self.grid(1).clone();
                for (id, grid) in self.grids.iter() {
                    outer_grid.combine(grid, self.cursor_render_info(*id));
                }
                outer_grid.print_colored(&self.highlights);
            }

            Event::GlobalEvent(GlobalEvent::Suspend)
            | Event::GlobalEvent(GlobalEvent::UpdateMenu)
            | Event::GlobalEvent(GlobalEvent::Bell)
            | Event::GlobalEvent(GlobalEvent::VisualBell) => {}
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
}
