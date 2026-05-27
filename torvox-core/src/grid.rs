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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::DirtyLine;

    #[test]
    fn grid_new_dimensions() {
        let g = Grid::new(24, 80);
        assert_eq!(g.rows(), 24);
        assert_eq!(g.cols(), 80);
    }

    #[test]
    fn grid_new_all_dirty() {
        let g = Grid::new(3, 5);
        for d in g.dirty() {
            assert!(*d != DirtyLine::Clean);
        }
    }

    #[test]
    fn grid_mark_clean() {
        let mut g = Grid::new(3, 5);
        g.mark_clean();
        for d in g.dirty() {
            assert_eq!(*d, DirtyLine::Clean);
        }
    }

    #[test]
    fn grid_get_valid_row() {
        let g = Grid::new(24, 80);
        assert!(g.get(0).is_some());
        assert!(g.get(23).is_some());
    }

    #[test]
    fn grid_get_out_of_bounds() {
        let g = Grid::new(24, 80);
        assert!(g.get(24).is_none());
    }

    #[test]
    fn grid_get_mut_marks_dirty() {
        let mut g = Grid::new(3, 5);
        g.mark_clean();
        let _ = g.get_mut(1);
        assert_eq!(g.dirty()[1], DirtyLine::Dirty(1));
        assert_eq!(g.dirty()[0], DirtyLine::Clean);
    }

    #[test]
    fn grid_resize() {
        let mut g = Grid::new(24, 80);
        g.resize(30, 100);
        assert_eq!(g.rows(), 30);
        assert_eq!(g.cols(), 100);
    }

    #[test]
    fn grid_resize_marks_all_dirty() {
        let mut g = Grid::new(3, 5);
        g.mark_clean();
        g.resize(5, 10);
        for d in g.dirty() {
            assert!(*d != DirtyLine::Clean);
        }
    }

    #[test]
    fn grid_cell_default() {
        let g = Grid::new(1, 1);
        let cell = g.get(0).unwrap().get(0).unwrap();
        assert_eq!(cell.char, ' ');
    }

    #[test]
    fn grid_set_cell() {
        let mut g = Grid::new(1, 1);
        g.get_mut(0).unwrap().get_mut(0).unwrap().char = 'X';
        assert_eq!(g.get(0).unwrap().get(0).unwrap().char, 'X');
    }

    #[test]
    fn grid_serde_roundtrip() {
        let g = Grid::new(4, 10);
        let bytes = postcard::to_allocvec(&g).unwrap();
        let decoded: Grid = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(decoded.rows(), 4);
        assert_eq!(decoded.cols(), 10);
    }
}
