mod grid;
mod window;

use self::window::Window;
use crate::{
    event::{Event, GlobalEvent, GridLine, GridScroll, WinPos},
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
                grid.resize(event.width, event.height);
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
            Event::HlAttrDefine(_) => {}
            Event::ModeChange(_) => {}
            Event::ModeInfoSet(_) => {}
            Event::HlGroupSet(_) => {}
            Event::GridCursorGoto(_) => {}
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
                if rows > 0 {
                    for y in (top..bot).rev() {
                        let dst_y = ((y as i64) - rows) as u64;
                        for x in left..right {
                            let c = grid.get(x, y);
                            grid.set(x, dst_y, c);
                        }
                    }
                } else {
                    for y in top..bot {
                        let dst_y = ((y as i64) - rows) as u64;
                        for x in left..right {
                            let c = grid.get(x, y);
                            grid.set(x, dst_y, c);
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
                let row = grid.row_mut(row);
                let row = &mut row[col_start as usize..];
                let mut dst = row.iter_mut();
                for cell in cells {
                    let c = cell.text.chars().into_iter().next().unwrap();
                    if let Some(repeat) = cell.repeat {
                        for _ in 0..repeat {
                            let dst = dst.next().unwrap();
                            *dst = c;
                        }
                    } else {
                        let dst = dst.next().unwrap();
                        *dst = c;
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
                    log::info!("flush");
                    let mut outer_grid = self.grid(1).clone();
                    let width = outer_grid.width() as usize;
                    let buffer = outer_grid.cells_mut();
                    for window in self.windows.values() {
                        let start_y = window.start.y as usize;
                        let start_x = window.start.x as usize;
                        // Invariant: Should not be possible to create a window
                        // without the corresponding grid
                        let grid = self.grids.get(&window.grid).unwrap();
                        for (y, row) in grid.rows().enumerate() {
                            for (x, col) in row.into_iter().enumerate() {
                                let y = y + start_y as usize;
                                let x = x + start_x as usize;
                                buffer[y * width + x] = *col;
                            }
                        }
                    }
                    println!("{outer_grid:?}");
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
