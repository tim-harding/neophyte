mod grid;
mod window;

use self::window::Window;
use crate::{
    event::{Event, GlobalEvent, GridLine, GridScroll, HlAttrDefine, WinPos},
    util::Vec2,
};
use grid::Grid;
use std::collections::{hash_map::Entry, HashMap};

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
                let height = grid.height();
                let mut copy = move |src_y, dst_y| {
                    for x in left..right {
                        let (c, highlight) = grid.get(Vec2::new(x, src_y));
                        grid.set(Vec2::new(x, dst_y), c, highlight);
                    }
                };
                // TODO: Skip iterations for lines that won't be copied
                if rows > 0 {
                    for y in top..bot {
                        if let Ok(dst_y) = ((y as i64) - rows).try_into() {
                            copy(y, dst_y);
                        }
                    }
                } else {
                    for y in (top..bot).rev() {
                        let dst_y = ((y as i64) - rows) as u64;
                        if dst_y < height {
                            copy(y, dst_y);
                        }
                    }
                }
            }
            Event::GridLine(event) => {
                // log::info!("{event:?}");
                let GridLine {
                    grid,
                    row,
                    col_start,
                    cells,
                } = event;
                let grid = self.grid(grid);
                let mut row = grid.row_mut(row).skip(col_start as usize);
                let mut highlight = 0;
                for cell in cells {
                    let c = cell.text.chars().into_iter().next().unwrap();
                    if let Some(hl_id) = cell.hl_id {
                        highlight = hl_id;
                    }
                    if let Some(repeat) = cell.repeat {
                        for _ in 0..repeat {
                            let dst = row.next().unwrap();
                            *dst.0 = c;
                            *dst.1 = highlight;
                        }
                    } else {
                        let dst = row.next().unwrap();
                        *dst.0 = c;
                        *dst.1 = highlight;
                    }
                }
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
                    // TODO: Optimize
                    let mut outer_grid = self.grid(1).clone();
                    for window in self.windows.values() {
                        let grid = self.grids.get(&window.grid).unwrap();
                        for (y, row) in grid.rows().enumerate() {
                            for (x, (c, hl)) in row.into_iter().enumerate() {
                                let pos = Vec2::new(x as u64, y as u64);
                                outer_grid.set(window.start + pos, c, hl);
                            }
                        }
                        if window.grid == self.cursor.grid && self.cursor.enabled {
                            let pos = window.start + self.cursor.pos;
                            outer_grid.set(pos, '█', 0);
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
