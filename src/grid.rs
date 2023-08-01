#![allow(unused)]

use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct Grid {
    cells: Vec<char>,
}

#[derive(Debug, Default, Clone)]
pub struct Grids {
    grids: HashMap<u64, Grid>,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            cells: vec![' '; width * height],
        }
    }
}
