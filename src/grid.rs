use crate::event::{Event, GlobalEvent, GridLine};
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

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

#[derive(Debug, Default, Clone)]
pub struct Grid {
    cells: Vec<char>,
    width: usize,
    height: usize,
}

impl Grid {
    pub fn resize(&mut self, width: usize, height: usize) {
        // TODO: Resize in place
        let mut new = vec![' '; (width * height) as usize];
        for y in 0..height.min(self.height) {
            for x in 0..width.min(self.width) {
                new[(y * width + x) as usize] = self.cells[(y * self.width + x) as usize];
            }
        }
        self.width = width;
        self.height = height;
        self.cells = new;
    }

    pub fn clear(&mut self) {
        for cell in self.cells.iter_mut() {
            *cell = ' ';
        }
    }

    pub fn row(&self, i: usize) -> &[char] {
        let start = i * self.width;
        &self.cells[start..start + self.width]
    }

    pub fn row_mut(&mut self, i: usize) -> &mut [char] {
        let start = i * self.width;
        &mut self.cells[start..start + self.width]
    }
}

impl Display for Grid {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for y in 0..self.height {
            let row = self.row(y);
            for cell in row {
                write!(f, "{cell}")?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}
