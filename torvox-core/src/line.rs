use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::cell::Cell;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Line {
    cells: Vec<Cell>,
}

impl Line {
    pub fn new(cols: u32) -> Self {
        Self {
            cells: alloc::vec![Cell::default(); cols as usize],
        }
    }

    pub fn len(&self) -> u32 {
        self.cells.len() as u32
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    pub fn get(&self, col: u32) -> Option<&Cell> {
        self.cells.get(col as usize)
    }

    pub fn get_mut(&mut self, col: u32) -> Option<&mut Cell> {
        self.cells.get_mut(col as usize)
    }

    pub fn resize(&mut self, new_cols: u32) {
        self.cells.resize(new_cols as usize, Cell::default());
    }
}
