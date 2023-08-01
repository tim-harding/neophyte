mod grid;

use crate::event::{Event, GlobalEvent, GridLine};
use grid::Grid;
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct Ui {
    title: String,
    icon: String,
    grids: HashMap<u64, Grid>,
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
                grid.resize(event.width as usize, event.height as usize);
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
            Event::GridScroll(_) => {}
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
                    let c = cell.text.chars().into_iter().next().unwrap();
                    if let Some(repeat) = cell.repeat {
                        for _ in 0..repeat {
                            if let Some(dst) = dst.next() {
                                *dst = c;
                            }
                        }
                    } else {
                        if let Some(dst) = dst.next() {
                            *dst = c;
                        }
                    }
                }
            }
            Event::WinViewport(_) => {}
            Event::TablineUpdate(_) => {}
            Event::MsgShowmode(_) => {}
            Event::MsgShowcmd(_) => {}
            Event::CmdlineShow(_) => {}
            Event::WinPos(_) => {}
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
                    let grid = self.grid(1);
                    println!("{grid}");
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
