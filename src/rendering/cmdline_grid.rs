use crate::{
    ui::cmdline::{Cmdline, Mode},
    util::vec2::Vec2,
};

use super::grid::Text;

pub struct CmdlineGrid {
    grid: Text,
}

impl CmdlineGrid {
    pub fn new() -> Self {
        Self {
            grid: Text::new(Vec2::new(0, 0)),
        }
    }

    pub fn update(&mut self, cmdline: Cmdline) {
        if let Some(mode) = cmdline.mode {
            match mode {
                Mode::Normal { levels } => {
                    // TODO: Handle multiple levels
                    let first = levels.first().unwrap();
                }
                Mode::Block {
                    previous_lines: _,
                    current_line: _,
                } => todo!(),
            }
        }
    }
}
