mod grid;
mod window;

use self::window::Window;
use crate::{
    event::{Event, GlobalEvent, GridLine, GridScroll, HlAttrDefine, WinPos},
    util::Vec2,
};
use grid::Grid;
use std::collections::{hash_map::Entry, HashMap};
use termcolor::{ColorChoice, StandardStream, WriteColor};

// TODO: Audit unwrap uses

#[derive(Debug, Default, Clone)]
pub struct Ui {
    title: String,
    icon: String,
    grids: HashMap<u64, Grid>,
    windows: HashMap<u64, Window>,
    cursor: CursorInfo,
    highlights: HashMap<u64, HlAttrDefine>,
    /// Lookup into the highlights map. Useful for a UI who want to render its
    /// own elements with consistent highlighting. For instance a UI using
    /// ui-popupmenu events, might use the hl-Pmenu family of builtin highlights
    highlight_groups: HashMap<String, u64>,
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
        match event {
            Event::MsgHistoryShow(_) => {}
            Event::CmdlineSpecialChar(_) => {}
            Event::PopupmenuShow(_) => {}
            Event::CmdlinePos(_) => {}
            Event::GridResize(event) => {
                log::info!("{event:?}");
                let grid = self.grid(event.grid);
                grid.resize(Vec2::new(event.width, event.height));
            }
            Event::SetTitle(event) => {
                log::info!("{event:?}");
                self.title = event.title;
            }
            Event::SetIcon(event) => {
                log::info!("{event:?}");
                self.icon = event.icon;
            }
            Event::OptionSet(_) => {}
            Event::GridClear(event) => {
                log::info!("{event:?}");
                self.grid(event.grid).clear();
            }
            Event::GridDestroy(event) => {
                log::info!("{event:?}");
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
                log::info!("{event:?}");
                self.cursor.pos = Vec2::new(event.column, event.row);
                self.cursor.grid = event.grid;
            }
            Event::GridScroll(event) => {
                log::info!("{event:?}");
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
            Event::MsgShowmode(_) => {}
            Event::MsgShowcmd(_) => {}
            Event::CmdlineShow(_) => {}
            Event::WinPos(event) => {
                log::info!("{event:?}");
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
            Event::MsgRuler(_) => {}
            Event::WinHide(_) => {}
            Event::WinClose(_) => {}
            Event::WinExternalPos(_) => {}
            Event::MsgSetPos(_) => {}
            Event::MsgShow(_) => {}
            Event::WinExtmark(_) => {}
            Event::PopupmenuSelect(_) => {}
            Event::CmdlineBlockShow(_) => {}
            Event::CmdlineBlockAppend(_) => {}
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
                GlobalEvent::CmdlineHide => {}
                GlobalEvent::CmdlineBlockHide => {}
                GlobalEvent::PopupmenuHide => {}
                GlobalEvent::MsgClear => {}
                GlobalEvent::MsgHistoryClear => {}
            },
        }
    }
}
