use crate::event::Event;
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct Grids {
    grids: HashMap<u64, Grid>,
}

impl Grids {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn process(&mut self, event: Event) {
        println!("{event:?}");
    }
}

#[derive(Debug, Default, Clone)]
pub struct Grid {
    cells: Vec<char>,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            cells: vec![' '; width * height],
        }
    }
}
