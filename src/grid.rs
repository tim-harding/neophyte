use crate::event::Event;
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
            Event::GridClear(_) => {}
            Event::GridDestroy(_) => {}
            Event::DefaultColorsSet(_) => {}
            Event::HlAttrDefine(_) => {}
            Event::ModeChange(_) => {}
            Event::ModeInfoSet(_) => {}
            Event::HlGroupSet(_) => {}
            Event::GridCursorGoto(_) => {}
            Event::GridScroll(_) => {}
            Event::GridLine(_) => {}
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
            Event::GlobalEvent(_) => {}
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Grid {
    cells: Vec<char>,
    width: u64,
    height: u64,
}

impl Grid {
    pub fn resize(&mut self, width: u64, height: u64) {
        let mut new = vec![' '; (width * height) as usize];
        for y in 0..height.min(self.height) {
            for x in 0..width.min(self.width) {
                new[(y * width + x) as usize] = self.cells[(y * self.width + x) as usize];
            }
        }
        self.cells = new;
    }
}
