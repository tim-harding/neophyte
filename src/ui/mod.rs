mod grid;
mod window;

use crate::{
    event::{Anchor, Event, GlobalEvent, GridLine, WinPos},
    util::Vec2,
};
use grid::Grid;
use std::collections::{hash_map::Entry, HashMap};

use self::window::Window;

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

    pub fn process(&mut self, event: Event) -> Result<(), Error> {
        match event {
            Event::MsgHistoryShow(_) => Ok(()),
            Event::CmdlineSpecialChar(_) => Ok(()),
            Event::PopupmenuShow(_) => Ok(()),
            Event::CmdlinePos(_) => Ok(()),
            Event::GridResize(event) => {
                log::info!("{event:?}");
                let grid = self.grid(event.grid);
                grid.resize(event.width as usize, event.height as usize);
                Ok(())
            }
            Event::SetTitle(event) => {
                log::info!("{event:?}");
                self.title = event.title;
                Ok(())
            }
            Event::SetIcon(event) => {
                log::info!("{event:?}");
                self.icon = event.icon;
                Ok(())
            }
            Event::OptionSet(_) => Ok(()),
            Event::GridClear(event) => {
                log::info!("{event:?}");
                self.grid(event.grid).clear();
                Ok(())
            }
            Event::GridDestroy(event) => {
                log::info!("{event:?}");
                self.grids.remove(&event.grid);
                Ok(())
            }
            Event::DefaultColorsSet(_) => Ok(()),
            Event::HlAttrDefine(_) => Ok(()),
            Event::ModeChange(_) => Ok(()),
            Event::ModeInfoSet(_) => Ok(()),
            Event::HlGroupSet(_) => Ok(()),
            Event::GridCursorGoto(_) => Ok(()),
            Event::GridScroll(_) => Ok(()),
            Event::GridLine(event) => {
                // log::info!("{event:?}");
                let GridLine {
                    grid,
                    row,
                    col_start,
                    cells,
                } = event;
                let grid = self.grid(grid);
                let row = grid.row_mut(row as usize);
                let row = &mut row[col_start as usize..];
                let mut dst = row.iter_mut();
                for cell in cells {
                    let c = cell
                        .text
                        .chars()
                        .into_iter()
                        .next()
                        .ok_or(Error::GridLineMissingChar)?;
                    if let Some(repeat) = cell.repeat {
                        for _ in 0..repeat {
                            let dst = dst.next().ok_or(Error::GridLineOverflowed)?;
                            *dst = c;
                        }
                    } else {
                        let dst = dst.next().ok_or(Error::GridLineOverflowed)?;
                        *dst = c;
                    }
                }
                Ok(())
            }
            Event::WinViewport(_) => Ok(()),
            Event::TablineUpdate(_) => Ok(()),
            Event::MsgShowmode(_) => Ok(()),
            Event::MsgShowcmd(_) => Ok(()),
            Event::CmdlineShow(_) => Ok(()),
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
                Ok(())
            }
            Event::WinFloatPos(_) => Ok(()),
            Event::MsgRuler(_) => Ok(()),
            Event::WinHide(_) => Ok(()),
            Event::WinClose(_) => Ok(()),
            Event::WinExternalPos(_) => Ok(()),
            Event::MsgSetPos(_) => Ok(()),
            Event::MsgShow(_) => Ok(()),
            Event::WinExtmark(_) => Ok(()),
            Event::PopupmenuSelect(_) => Ok(()),
            Event::CmdlineBlockShow(_) => Ok(()),
            Event::CmdlineBlockAppend(_) => Ok(()),
            Event::GlobalEvent(event) => match event {
                GlobalEvent::MouseOn => Ok(()),
                GlobalEvent::MouseOff => Ok(()),
                GlobalEvent::BusyStart => Ok(()),
                GlobalEvent::BusyStop => Ok(()),
                GlobalEvent::Suspend => Ok(()),
                GlobalEvent::UpdateMenu => Ok(()),
                GlobalEvent::Bell => Ok(()),
                GlobalEvent::VisualBell => Ok(()),
                GlobalEvent::Flush => {
                    log::info!("flush");
                    let mut outer_grid = self.grid(1).clone();
                    let width = outer_grid.width();
                    let buffer = outer_grid.cells_mut();
                    for window in self.windows.values() {
                        // Invariant: Should not be possible to create a window
                        // without the corresponding grid
                        let grid = self.grids.get(&window.grid).unwrap();
                        for (y, row) in grid.rows().enumerate() {
                            for (x, col) in row.into_iter().enumerate() {
                                let y = y + window.start.y as usize;
                                let x = x + window.start.x as usize;
                                buffer[y * width + x] = *col;
                            }
                        }
                    }
                    println!("{outer_grid}");
                    Ok(())
                }
                GlobalEvent::CmdlineHide => Ok(()),
                GlobalEvent::CmdlineBlockHide => Ok(()),
                GlobalEvent::PopupmenuHide => Ok(()),
                GlobalEvent::MsgClear => Ok(()),
                GlobalEvent::MsgHistoryClear => Ok(()),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum Error {
    #[error("A grid_line event had a cell with empty text")]
    GridLineMissingChar,
    #[error("A grid_line event overflowed the bounds of the grid")]
    GridLineOverflowed,
}
