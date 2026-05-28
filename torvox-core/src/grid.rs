use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::cell::DirtyMask;
use crate::line::Line;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grid {
    lines: Vec<Line>,
    dirty: DirtyMask,
    rows: u32,
    cols: u32,
}

impl Grid {
    pub fn new(rows: u32, cols: u32) -> Self {
        let lines = (0..rows).map(|_| Line::new(cols)).collect();
        let mut dirty = DirtyMask::new(rows);
        dirty.mark_all(rows);
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
        self.dirty.mark(row);
        self.lines.get_mut(row as usize)
    }

    pub fn dirty(&self) -> &DirtyMask {
        &self.dirty
    }

    pub fn mark_clean(&mut self) {
        self.dirty.clear();
    }

    pub fn resize(&mut self, new_rows: u32, new_cols: u32) {
        self.lines.resize(new_rows as usize, Line::new(new_cols));
        for line in &mut self.lines {
            line.resize(new_cols);
        }
        self.dirty.resize(new_rows);
        self.dirty.mark_all(new_rows);
        self.rows = new_rows;
        self.cols = new_cols;
    }

    pub fn cell_mut(&mut self, row: u32, col: u32) -> Option<&mut crate::cell::Cell> {
        self.dirty.mark(row);
        self.lines
            .get_mut(row as usize)
            .and_then(|line| line.get_mut(col))
    }

    pub fn cell(&self, row: u32, col: u32) -> Option<&crate::cell::Cell> {
        self.lines.get(row as usize).and_then(|line| line.get(col))
    }

    pub fn mark_row_dirty(&mut self, row: u32) {
        self.dirty.mark(row);
    }

    pub fn mark_rows_dirty(&mut self, start: u32, end: u32) {
        for row in start..end {
            self.dirty.mark(row);
        }
    }

    pub fn scroll_up(&mut self, top: u32, bottom: u32, cols: u32) {
        if top >= bottom || bottom > self.rows {
            return;
        }
        let count = (bottom - top) as usize;
        if count <= 1 {
            return;
        }
        self.lines.remove(top as usize);
        self.lines.insert(bottom as usize - 1, Line::new(cols));
        for row in top..bottom {
            self.dirty.mark(row);
        }
    }

    pub fn scroll_down(&mut self, top: u32, bottom: u32, cols: u32) {
        if top >= bottom || bottom > self.rows {
            return;
        }
        self.lines.remove(bottom as usize - 1);
        self.lines.insert(top as usize, Line::new(cols));
        for row in top..bottom {
            self.dirty.mark(row);
        }
    }

    pub fn insert_lines(&mut self, at: u32, count: u32, bottom: u32, cols: u32) {
        if at >= bottom || count == 0 {
            return;
        }
        let actual = count.min(bottom - at);
        for _ in 0..actual {
            self.lines.remove(bottom as usize - 1);
            self.lines.insert(at as usize, Line::new(cols));
        }
        for row in at..bottom {
            self.dirty.mark(row);
        }
    }

    pub fn delete_lines(&mut self, at: u32, count: u32, bottom: u32, cols: u32) {
        if at >= bottom || count == 0 {
            return;
        }
        let actual = count.min(bottom - at);
        for _ in 0..actual {
            self.lines.remove(at as usize);
            self.lines.insert(bottom as usize - 1, Line::new(cols));
        }
        for row in at..bottom {
            self.dirty.mark(row);
        }
    }

    pub fn clear_cells(&mut self, row: u32, start_col: u32, end_col: u32) {
        if let Some(line) = self.lines.get_mut(row as usize) {
            for col in start_col..end_col.min(line.len()) {
                if let Some(cell) = line.get_mut(col) {
                    *cell = crate::cell::Cell::default();
                }
            }
            self.dirty.mark(row);
        }
    }

    pub fn fill_cells(&mut self, row: u32, ch: char, start_col: u32, end_col: u32) {
        if let Some(line) = self.lines.get_mut(row as usize) {
            for col in start_col..end_col.min(line.len()) {
                if let Some(cell) = line.get_mut(col) {
                    cell.char = ch;
                }
            }
            self.dirty.mark(row);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_new_dimensions() {
        let g = Grid::new(24, 80);
        assert_eq!(g.rows(), 24);
        assert_eq!(g.cols(), 80);
    }

    #[test]
    fn grid_new_all_dirty() {
        let g = Grid::new(3, 5);
        for row in 0..3 {
            assert!(g.dirty().is_dirty(row));
        }
    }

    #[test]
    fn grid_mark_clean() {
        let mut g = Grid::new(3, 5);
        g.mark_clean();
        assert!(!g.dirty().any_dirty());
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
        assert!(g.dirty().is_dirty(1));
        assert!(!g.dirty().is_dirty(0));
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
        assert!(g.dirty().any_dirty());
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

    #[test]
    fn dirty_mask_bit_ops() {
        let mut m = DirtyMask::new(24);
        assert!(!m.any_dirty());
        m.mark(0);
        m.mark(3);
        assert!(m.is_dirty(0));
        assert!(m.is_dirty(3));
        assert!(!m.is_dirty(1));
        m.clear();
        assert!(!m.any_dirty());
        m.mark_all(5);
        for i in 0..5 {
            assert!(m.is_dirty(i));
        }
    }
}
