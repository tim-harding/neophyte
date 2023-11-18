use crate::event::{CmdlineShow, CmdlineSpecialChar, Content};

#[derive(Debug, Clone, Default)]
pub struct Cmdline {
    pub mode: Option<Mode>,
}

impl Cmdline {
    pub fn show(&mut self, event: CmdlineShow) {
        let i = event.level as usize;
        let level = event.into();
        match &mut self.mode {
            Some(Mode::Normal { levels }) => {
                if i > levels.len() {
                    levels.push(level);
                } else {
                    levels[i - 1] = level;
                }
            }
            Some(Mode::Block {
                previous_lines: _,
                current_line,
            }) => {
                *current_line = level;
            }
            None => {
                self.mode = Some(Mode::Normal {
                    levels: vec![level],
                })
            }
        }
    }

    pub fn hide(&mut self) {
        if let Some(Mode::Normal { levels }) = &mut self.mode {
            match levels.len() {
                0 | 1 => {
                    self.mode = None;
                }
                _ => {
                    levels.pop();
                }
            }
        }
    }

    pub fn hide_block(&mut self) {
        self.mode = None;
    }

    pub fn show_block(&mut self, lines: Vec<Content>) {
        self.mode = Some(Mode::Block {
            previous_lines: lines,
            // Will be filled in by a cmdline_show event before next flush
            current_line: Default::default(),
        });
    }

    pub fn append_block(&mut self, line: Content) {
        if let Some(Mode::Block {
            previous_lines,
            current_line,
        }) = &mut self.mode
        {
            previous_lines.push(line);
            // Will be filled in by a cmdline_show event before next flush
            *current_line = Default::default();
        }
    }

    pub fn set_cursor_pos(&mut self, pos: u32) {
        match &mut self.mode {
            Some(Mode::Normal { levels }) => {
                if let Some(level) = levels.last_mut() {
                    level.cursor_pos = pos;
                }
            }
            Some(Mode::Block {
                previous_lines: _,
                current_line,
            }) => {
                current_line.cursor_pos = pos;
            }
            None => {}
        }
    }

    pub fn special(&mut self, event: CmdlineSpecialChar) {
        let special = Some(Special::new(event.c, event.shift));
        match &mut self.mode {
            Some(Mode::Normal { levels }) => {
                if let Some(level) = levels.last_mut() {
                    level.special = special;
                }
            }
            Some(Mode::Block {
                previous_lines: _,
                current_line,
            }) => {
                current_line.special = special;
            }
            None => {}
        }
    }
}

#[derive(Debug, Clone)]
pub enum Mode {
    Normal {
        levels: Vec<Prompt>,
    },
    Block {
        previous_lines: Vec<Content>,
        current_line: Prompt,
    },
}

#[derive(Debug, Clone, Default)]
pub struct Prompt {
    pub content_lines: Vec<Content>,
    pub cursor_pos: u32,
    pub first_char: Option<char>,
    pub prompt: String,
    pub special: Option<Special>,
    pub indent: u32,
}

impl From<CmdlineShow> for Prompt {
    fn from(value: CmdlineShow) -> Self {
        Self {
            content_lines: vec![value.content],
            cursor_pos: value.pos,
            first_char: value.firstc.chars().next(),
            prompt: value.prompt,
            special: None,
            indent: value.indent,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Special {
    pub c: char,
    pub shift: bool,
}

impl Special {
    pub fn new(c: char, shift: bool) -> Self {
        Self { c, shift }
    }
}
