mod cmdline;
mod grid;
mod messages;
mod print;
mod window;

use self::{cmdline::Cmdline, messages::Messages, window::Window};
use crate::{
    event::{Event, GlobalEvent, GridLine, GridScroll, HlAttrDefine, WinPos},
    util::Vec2,
};
use grid::Grid;
use std::collections::{hash_map::Entry, HashMap};

pub type Highlights = HashMap<u64, HlAttrDefine>;
pub type HighlightGroups = HashMap<String, u64>;

#[derive(Debug, Default, Clone)]
pub struct Ui {
    title: String,
    icon: String,
    grids: HashMap<u64, Grid>,
    windows: HashMap<u64, Window>,
    cursor: CursorInfo,
    highlights: Highlights,
    highlight_groups: HighlightGroups,
    messages: Messages,
    cmdline: Cmdline,
}

#[derive(Debug, Copy, Clone)]
struct CursorInfo {
    pos: Vec2,
    grid: u64,
    enabled: bool,
}

impl Default for CursorInfo {
    fn default() -> Self {
        Self {
            pos: Default::default(),
            grid: 1,
            enabled: true,
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
            Event::PopupmenuShow(_) => {}
            Event::GridResize(event) => {
                let grid = self.grid(event.grid);
                grid.resize(Vec2::new(event.width, event.height));
            }
            Event::SetTitle(event) => {
                self.title = event.title;
            }
            Event::SetIcon(event) => {
                self.icon = event.icon;
            }
            Event::OptionSet(_) => {}
            Event::GridClear(event) => {
                self.grid(event.grid).clear();
            }
            Event::GridDestroy(event) => {
                self.grids.remove(&event.grid);
            }
            Event::DefaultColorsSet(_) => {}
            Event::HlAttrDefine(event) => {
                self.highlights.insert(event.id, event);
            }
            Event::ModeChange(_) => {}
            Event::ModeInfoSet(_) => {}
            Event::HlGroupSet(event) => {
                self.highlight_groups.insert(event.name, event.hl_id);
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
            Event::WinViewport(_) => {}
            Event::TablineUpdate(_) => {}
            Event::WinPos(event) => {
                let WinPos {
                    grid,
                    win: _,
                    start_row,
                    start_col,
                    width,
                    height,
                } = event;
                match self.windows.entry(grid) {
                    Entry::Occupied(mut entry) => {
                        let window = entry.get_mut();
                        window.start = Vec2::new(start_col, start_row);
                        window.size = Vec2::new(width, height);
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(Window {
                            grid,
                            start: Vec2::new(start_col, start_row),
                            size: Vec2::new(width, height),
                            anchor: None,
                            focusable: true,
                        });
                    }
                }
            }
            Event::WinFloatPos(_) => {}
            Event::WinHide(_) => {}
            Event::WinClose(_) => {}
            Event::WinExternalPos(_) => {}
            Event::WinExtmark(_) => {}
            Event::PopupmenuSelect(_) => {}

            Event::CmdlineShow(event) => {
                self.cmdline.show(event);
            }
            Event::CmdlinePos(event) => {
                self.cmdline.set_cursor_pos(event.pos);
            }
            Event::CmdlineBlockShow(event) => {
                self.cmdline.show_block(event.lines);
            }
            Event::CmdlineBlockAppend(event) => {
                self.cmdline.append_block(event.line);
            }
            Event::CmdlineSpecialChar(event) => {
                self.cmdline.special(event);
            }

            Event::MsgHistoryShow(event) => {
                self.messages.history = event.entries;
            }
            Event::MsgRuler(event) => {
                self.messages.ruler = event.content;
            }
            Event::MsgSetPos(_) => {} // Not used when ui-messages is enabled
            Event::MsgShow(event) => {
                self.messages.show(event);
            }
            Event::MsgShowmode(event) => {
                self.messages.showmode = event.content;
            }
            Event::MsgShowcmd(event) => {
                self.messages.showcmd = event.content;
            }

            Event::GlobalEvent(event) => match event {
                GlobalEvent::MouseOn => {}
                GlobalEvent::MouseOff => {}
                GlobalEvent::BusyStart => {}
                GlobalEvent::BusyStop => {}
                GlobalEvent::Suspend => {}
                GlobalEvent::UpdateMenu => {}
                GlobalEvent::Bell => {}
                GlobalEvent::VisualBell => {}
                GlobalEvent::Flush => {
                    let mut outer_grid = self.grid(1).clone();
                    for window in self.windows.values() {
                        let grid = self.grids.get(&window.grid).unwrap();
                        outer_grid.combine(grid, window.start);
                        if window.grid == self.cursor.grid && self.cursor.enabled {
                            let hl = *self.highlight_groups.get("Cursor").unwrap_or(&0);
                            let pos = window.start + self.cursor.pos;
                            outer_grid.set_hl(pos, hl);
                        }
                    }
                    outer_grid.print_colored(&self.highlights);
                }
                GlobalEvent::PopupmenuHide => {}
                GlobalEvent::MsgClear => {
                    self.messages.show.clear();
                }
                GlobalEvent::CmdlineHide => self.cmdline.hide(),
                GlobalEvent::CmdlineBlockHide => self.cmdline.hide_block(),
                GlobalEvent::MsgHistoryClear => {
                    self.messages.history.clear();
                }
            },
        }
    }
}
