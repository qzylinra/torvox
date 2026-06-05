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

/// 终端行：固定容量的 `Box<[Cell]>`，提供稳定地址和较小的内联 `attr`。
/// `Box<[Cell]>` 避免了 `Vec` 的容量/长度开销，是构建时大小已知的行的自然选择。
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

    #[test]
    fn line_default_attr_is_normal() {
        let l = Line::new(5);
        assert_eq!(l.attr, LineAttr::Normal);
    }

    #[test]
    fn line_set_attr() {
        let mut l = Line::new(5);
        l.attr = LineAttr::DoubleWidth;
        assert_eq!(l.attr, LineAttr::DoubleWidth);
    }

    #[test]
    fn line_resize_same_size_no_op() {
        let mut l = Line::new(5);
        l.get_mut(0).unwrap().char = 'A';
        l.resize(5);
        assert_eq!(l.len(), 5);
        assert_eq!(l.get(0).unwrap().char, 'A');
    }

    #[test]
    fn line_resize_to_zero() {
        let mut l = Line::new(5);
        l.resize(0);
        assert_eq!(l.len(), 0);
        assert!(l.is_empty());
    }

    #[test]
    fn line_resize_from_zero() {
        let mut l = Line::new(0);
        l.resize(5);
        assert_eq!(l.len(), 5);
    }

    #[test]
    fn line_get_mut_modifies() {
        let mut l = Line::new(3);
        let c = l.get_mut(1).unwrap();
        c.char = 'X';
        c.fg = crate::cell::Color::new(1, 2, 3);
        assert_eq!(l.get(1).unwrap().char, 'X');
        assert_eq!(l.get(1).unwrap().fg, crate::cell::Color::new(1, 2, 3));
    }

    #[test]
    fn line_cells_returns_full_slice() {
        let l = Line::new(5);
        let cells: &[crate::cell::Cell] = l.cells();
        assert_eq!(cells.len(), 5);
    }

    #[test]
    fn line_resize_shrink_drops_suffix() {
        let mut l = Line::new(5);
        l.get_mut(3).unwrap().char = 'X';
        l.get_mut(4).unwrap().char = 'Y';
        l.resize(2);
        assert_eq!(l.len(), 2);
    }

    #[test]
    fn line_serde_json_roundtrip() {
        let mut l = Line::new(3);
        l.get_mut(0).unwrap().char = 'A';
        l.get_mut(1).unwrap().char = 'B';
        l.attr = LineAttr::DoubleWidth;
        let json = serde_json::to_string(&l).unwrap();
        let back: Line = serde_json::from_str(&json).unwrap();
        assert_eq!(l, back);
    }

    #[test]
    fn line_equality() {
        let l1 = Line::new(3);
        let l2 = Line::new(3);
        assert_eq!(l1, l2);
        let mut l3 = Line::new(3);
        l3.get_mut(0).unwrap().char = 'X';
        assert_ne!(l1, l3);
    }

    #[test]
    fn line_clone() {
        let mut l = Line::new(3);
        l.get_mut(0).unwrap().char = 'A';
        let l2 = l.clone();
        assert_eq!(l, l2);
    }

    #[test]
    fn line_attr_variants() {
        let mut l = Line::new(1);
        l.attr = LineAttr::Normal;
        assert_eq!(l.attr, LineAttr::Normal);
        l.attr = LineAttr::DoubleWidth;
        assert_eq!(l.attr, LineAttr::DoubleWidth);
        l.attr = LineAttr::DoubleHeightTop;
        assert_eq!(l.attr, LineAttr::DoubleHeightTop);
        l.attr = LineAttr::DoubleHeightBottom;
        assert_eq!(l.attr, LineAttr::DoubleHeightBottom);
    }
}
