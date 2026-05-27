use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::cell::DirtyLine;
use crate::line::Line;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grid {
    lines: Vec<Line>,
    dirty: Vec<DirtyLine>,
    rows: u32,
    cols: u32,
}

impl Grid {
    pub fn new(rows: u32, cols: u32) -> Self {
        let lines = (0..rows).map(|_| Line::new(cols)).collect();
        let dirty = alloc::vec![DirtyLine::Dirty(0); rows as usize];
        Self {
            lines,
            dirty,
            rows,
            cols,
        }
    }

    pub fn rows(&self) -> u32 {
        self.rows
    }

    pub fn cols(&self) -> u32 {
        self.cols
    }

    pub fn get(&self, row: u32) -> Option<&Line> {
        self.lines.get(row as usize)
    }

    pub fn get_mut(&mut self, row: u32) -> Option<&mut Line> {
        self.dirty[row as usize] = DirtyLine::Dirty(row);
        self.lines.get_mut(row as usize)
    }

    pub fn dirty(&self) -> &[DirtyLine] {
        &self.dirty
    }

    pub fn mark_clean(&mut self) {
        for d in &mut self.dirty {
            *d = DirtyLine::Clean;
        }
    }

    pub fn resize(&mut self, new_rows: u32, new_cols: u32) {
        self.lines.resize(new_rows as usize, Line::new(new_cols));
        for line in &mut self.lines {
            line.resize(new_cols);
        }
        self.dirty = alloc::vec![DirtyLine::Dirty(0); new_rows as usize];
        self.rows = new_rows;
        self.cols = new_cols;
    }
}
