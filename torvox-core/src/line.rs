use alloc::boxed::Box;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::cell::Cell;

#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LineAttr {
    #[default]
    Normal,
    DoubleWidth,
    DoubleHeightTop,
    DoubleHeightBottom,
}

/// Terminal line: fixed-capacity `Box<[Cell]>` for stable address + small
/// inline `attr`. `Box<[Cell]>` avoids the capacity/len overhead of `Vec` and
/// is the natural choice for a line whose size is known at construction.
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Line {
    cells: Box<[Cell]>,
    pub attr: LineAttr,
}

impl Line {
    pub fn new(cols: u32) -> Self {
        let cells: Box<[Cell]> = (0..cols).map(|_| Cell::default()).collect();
        Self {
            cells,
            attr: LineAttr::Normal,
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
        let new_len = new_cols as usize;
        if new_len == self.cells.len() {
            return;
        }
        let mut new_cells = Vec::with_capacity(new_len);
        new_cells.resize(new_len, Cell::default());
        let copy_len = new_len.min(self.cells.len());
        new_cells[..copy_len].clone_from_slice(&self.cells[..copy_len]);
        self.cells = new_cells.into_boxed_slice();
    }

    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_new_len() {
        let l = Line::new(80);
        assert_eq!(l.len(), 80);
        assert!(!l.is_empty());
    }

    #[test]
    fn line_new_empty() {
        let l = Line::new(0);
        assert_eq!(l.len(), 0);
        assert!(l.is_empty());
    }

    #[test]
    fn line_get_valid() {
        let l = Line::new(5);
        assert!(l.get(0).is_some());
        assert!(l.get(4).is_some());
    }

    #[test]
    fn line_get_out_of_bounds() {
        let l = Line::new(5);
        assert!(l.get(5).is_none());
    }

    #[test]
    fn line_default_cells_are_spaces() {
        let l = Line::new(10);
        for c in l.cells() {
            assert_eq!(c.char, ' ');
        }
    }

    #[test]
    fn line_set_cell() {
        let mut l = Line::new(5);
        l.get_mut(2).unwrap().char = 'A';
        assert_eq!(l.get(2).unwrap().char, 'A');
    }

    #[test]
    fn line_resize_grow() {
        let mut l = Line::new(5);
        l.resize(10);
        assert_eq!(l.len(), 10);
    }

    #[test]
    fn line_resize_shrink() {
        let mut l = Line::new(10);
        l.resize(5);
        assert_eq!(l.len(), 5);
    }

    #[test]
    fn line_resize_new_cells_default() {
        let mut l = Line::new(2);
        l.resize(3);
        assert_eq!(l.get(2).unwrap().char, ' ');
    }

    #[test]
    fn line_resize_preserves_existing_cells() {
        let mut l = Line::new(3);
        l.get_mut(0).unwrap().char = 'A';
        l.get_mut(1).unwrap().char = 'B';
        l.resize(5);
        assert_eq!(l.get(0).unwrap().char, 'A');
        assert_eq!(l.get(1).unwrap().char, 'B');
        assert_eq!(l.get(2).unwrap().char, ' ');
        assert_eq!(l.get(3).unwrap().char, ' ');
    }

    #[test]
    fn line_resize_shrink_keeps_prefix() {
        let mut l = Line::new(5);
        l.get_mut(0).unwrap().char = 'A';
        l.get_mut(1).unwrap().char = 'B';
        l.resize(2);
        assert_eq!(l.len(), 2);
        assert_eq!(l.get(0).unwrap().char, 'A');
        assert_eq!(l.get(1).unwrap().char, 'B');
    }
}
