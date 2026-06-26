// @Selection modes, IMPL_CORE_006, impl, [REQ_CORE_006]
// @need-ids: REQ_CORE_006
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum SelectionMode {
    #[default]
    Char,
    Word,
    Line,
    Block,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct SelectionAnchor {
    pub row: u32,
    pub col: u32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct Selection {
    pub start: SelectionAnchor,
    pub end: SelectionAnchor,
    pub mode: SelectionMode,
}

impl Selection {
    pub fn new(start: SelectionAnchor, end: SelectionAnchor, mode: SelectionMode) -> Self {
        Self { start, end, mode }
    }

    pub fn is_ordered(&self) -> bool {
        if self.start.row < self.end.row {
            return true;
        }
        self.start.row == self.end.row && self.start.col <= self.end.col
    }

    pub fn ordered(&self) -> (SelectionAnchor, SelectionAnchor) {
        if self.is_ordered() {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        }
    }

    pub fn contains(&self, row: u32, col: u32) -> bool {
        let (lo, hi) = self.ordered();
        match self.mode {
            SelectionMode::Char | SelectionMode::Word => {
                if row < lo.row || row > hi.row {
                    return false;
                }
                if row == lo.row && row == hi.row {
                    col >= lo.col && col <= hi.col
                } else if row == lo.row {
                    col >= lo.col
                } else if row == hi.row {
                    col <= hi.col
                } else {
                    true
                }
            }
            SelectionMode::Line => row >= lo.row && row <= hi.row,
            SelectionMode::Block => {
                row >= lo.row && row <= hi.row && col >= lo.col && col <= hi.col
            }
        }
    }

    /// Extract selected text from a grid.
    pub fn text(&self, grid: &crate::grid::Grid) -> alloc::string::String {
        use core::fmt::Write;
        let (lo, hi) = self.ordered();
        let mut result = alloc::string::String::new();
        match self.mode {
            SelectionMode::Char | SelectionMode::Word => {
                for row in lo.row..=hi.row {
                    if let Some(cells) = grid.row_cells(row) {
                        let start_col = if row == lo.row { lo.col } else { 0 };
                        let end_col = if row == hi.row {
                            hi.col
                        } else {
                            cells.len() as u32 - 1
                        };
                        let mut row_str = alloc::string::String::new();
                        for col in start_col..=end_col {
                            if let Some(cell) = cells.get(col as usize)
                                && cell.char != '\0'
                            {
                                let _ = row_str.write_char(cell.char);
                            }
                        }
                        if row < hi.row {
                            let _ = row_str.write_char('\n');
                        }
                        let trimmed = row_str.trim_end();
                        let _ = result.write_str(trimmed);
                        if row < hi.row && !trimmed.is_empty() {
                            let _ = result.write_char('\n');
                        }
                    }
                }
            }
            SelectionMode::Line => {
                for row in lo.row..=hi.row {
                    if let Some(cells) = grid.row_cells(row) {
                        let text: alloc::string::String = cells
                            .iter()
                            .map(|c| if c.char == '\0' { ' ' } else { c.char })
                            .collect();
                        let _ = result.write_str(text.trim_end());
                        if row < hi.row {
                            let _ = result.write_char('\n');
                        }
                    }
                }
            }
            SelectionMode::Block => {
                for row in lo.row..=hi.row {
                    if let Some(cells) = grid.row_cells(row) {
                        for col in lo.col..=hi.col {
                            if let Some(cell) = cells.get(col as usize) {
                                let _ = result.write_char(if cell.char == '\0' {
                                    ' '
                                } else {
                                    cell.char
                                });
                            }
                        }
                        if row < hi.row {
                            let _ = result.write_char('\n');
                        }
                    }
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_ordered_same_row() {
        let s = Selection::new(
            SelectionAnchor { row: 5, col: 10 },
            SelectionAnchor { row: 5, col: 3 },
            SelectionMode::Char,
        );
        assert!(!s.is_ordered());
        let (lo, hi) = s.ordered();
        assert_eq!(lo.col, 3);
        assert_eq!(hi.col, 10);
    }

    #[test]
    fn selection_ordered_different_rows() {
        let s = Selection::new(
            SelectionAnchor { row: 3, col: 0 },
            SelectionAnchor { row: 5, col: 10 },
            SelectionMode::Char,
        );
        assert!(s.is_ordered());
    }

    #[test]
    fn char_selection_contains() {
        let s = Selection::new(
            SelectionAnchor { row: 2, col: 5 },
            SelectionAnchor { row: 4, col: 10 },
            SelectionMode::Char,
        );
        assert!(s.contains(2, 5));
        assert!(s.contains(2, 6));
        assert!(s.contains(3, 0));
        assert!(s.contains(4, 10));
        assert!(!s.contains(2, 4));
        assert!(!s.contains(4, 11));
        assert!(!s.contains(1, 0));
        assert!(!s.contains(5, 0));
    }

    #[test]
    fn line_selection_contains() {
        let s = Selection::new(
            SelectionAnchor { row: 2, col: 5 },
            SelectionAnchor { row: 4, col: 10 },
            SelectionMode::Line,
        );
        assert!(s.contains(2, 0));
        assert!(s.contains(3, 0));
        assert!(s.contains(4, 79));
        assert!(!s.contains(1, 0));
        assert!(!s.contains(5, 0));
    }

    #[test]
    fn block_selection_contains() {
        let s = Selection::new(
            SelectionAnchor { row: 2, col: 5 },
            SelectionAnchor { row: 4, col: 10 },
            SelectionMode::Block,
        );
        assert!(s.contains(2, 5));
        assert!(s.contains(3, 7));
        assert!(s.contains(4, 10));
        assert!(!s.contains(2, 4));
        assert!(!s.contains(3, 11));
    }

    #[test]
    fn selection_mode_default_is_char() {
        assert_eq!(SelectionMode::default(), SelectionMode::Char);
    }

    #[test]
    fn selection_all_modes_distinct() {
        assert_ne!(SelectionMode::Char, SelectionMode::Word);
        assert_ne!(SelectionMode::Char, SelectionMode::Line);
        assert_ne!(SelectionMode::Char, SelectionMode::Block);
        assert_ne!(SelectionMode::Word, SelectionMode::Line);
        assert_ne!(SelectionMode::Word, SelectionMode::Block);
        assert_ne!(SelectionMode::Line, SelectionMode::Block);
    }

    #[test]
    fn selection_word_mode_same_as_char() {
        // Word mode uses the same containment logic as char mode
        let s = Selection::new(
            SelectionAnchor { row: 1, col: 2 },
            SelectionAnchor { row: 3, col: 4 },
            SelectionMode::Word,
        );
        assert!(s.contains(2, 0));
        assert!(!s.contains(4, 0));
    }

    #[test]
    fn selection_line_mode_full_row() {
        let s = Selection::new(
            SelectionAnchor { row: 0, col: 50 },
            SelectionAnchor { row: 0, col: 10 },
            SelectionMode::Line,
        );
        let (lo, hi) = s.ordered();
        assert_eq!(lo.row, 0);
        assert_eq!(hi.row, 0);
        assert!(s.contains(0, 0));
        assert!(s.contains(0, 1000));
    }

    #[test]
    fn selection_block_outside_rows() {
        let s = Selection::new(
            SelectionAnchor { row: 2, col: 5 },
            SelectionAnchor { row: 4, col: 10 },
            SelectionMode::Block,
        );
        assert!(!s.contains(1, 7));
        assert!(!s.contains(5, 7));
    }

    #[test]
    fn selection_block_inside_cols_outside_rows() {
        let s = Selection::new(
            SelectionAnchor { row: 2, col: 5 },
            SelectionAnchor { row: 4, col: 10 },
            SelectionMode::Block,
        );
        assert!(!s.contains(0, 7));
        assert!(!s.contains(10, 7));
    }

    #[test]
    fn selection_anchor_equality() {
        let a = SelectionAnchor { row: 1, col: 2 };
        let b = SelectionAnchor { row: 1, col: 2 };
        assert_eq!(a, b);
        let c = SelectionAnchor { row: 1, col: 3 };
        assert_ne!(a, c);
    }

    #[test]
    fn selection_anchor_copy() {
        let a = SelectionAnchor { row: 1, col: 2 };
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn selection_serde_json_roundtrip() {
        let s = Selection::new(
            SelectionAnchor { row: 1, col: 2 },
            SelectionAnchor { row: 3, col: 4 },
            SelectionMode::Line,
        );
        let json = serde_json::to_string(&s).unwrap();
        let back: Selection = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn selection_ordered_when_start_equals_end() {
        let s = Selection::new(
            SelectionAnchor { row: 5, col: 5 },
            SelectionAnchor { row: 5, col: 5 },
            SelectionMode::Char,
        );
        assert!(s.is_ordered());
        let (lo, hi) = s.ordered();
        assert_eq!(lo, hi);
    }

    #[test]
    fn selection_char_contains_single_cell() {
        let s = Selection::new(
            SelectionAnchor { row: 5, col: 5 },
            SelectionAnchor { row: 5, col: 5 },
            SelectionMode::Char,
        );
        assert!(s.contains(5, 5));
        assert!(!s.contains(5, 4));
        assert!(!s.contains(5, 6));
    }

    #[test]
    fn selection_mode_serde() {
        for mode in [
            SelectionMode::Char,
            SelectionMode::Word,
            SelectionMode::Line,
            SelectionMode::Block,
        ] {
            let json = serde_json::to_string(&mode).unwrap();
            let back: SelectionMode = serde_json::from_str(&json).unwrap();
            assert_eq!(mode, back);
        }
    }

    #[test]
    fn selection_is_ordered_just_equals_col() {
        // same row, end col equal to start col
        let s = Selection::new(
            SelectionAnchor { row: 5, col: 10 },
            SelectionAnchor { row: 5, col: 10 },
            SelectionMode::Char,
        );
        assert!(s.is_ordered());
    }

    #[test]
    fn selection_ordered_end_before_start_swaps() {
        let s = Selection::new(
            SelectionAnchor { row: 5, col: 10 },
            SelectionAnchor { row: 3, col: 5 },
            SelectionMode::Char,
        );
        let (lo, hi) = s.ordered();
        assert_eq!(lo.row, 3);
        assert_eq!(hi.row, 5);
    }

    #[test]
    fn selection_block_contains_middle_row() {
        let s = Selection::new(
            SelectionAnchor { row: 2, col: 5 },
            SelectionAnchor { row: 4, col: 10 },
            SelectionMode::Block,
        );
        assert!(s.contains(3, 7));
        assert!(!s.contains(3, 3));
    }

    fn make_grid_with_text(lines: &[&str]) -> crate::grid::Grid {
        use crate::cell::Cell;
        let rows = lines.len() as u32;
        let cols = lines.iter().map(|l| l.len()).max().unwrap_or(1) as u32;
        let mut grid = crate::grid::Grid::new(rows, cols);
        for (row_idx, line) in lines.iter().enumerate() {
            for (col_idx, ch) in line.chars().enumerate() {
                if let Some(cell) = grid.cell_mut(row_idx as u32, col_idx as u32) {
                    *cell = Cell {
                        char: ch,
                        ..Default::default()
                    };
                }
            }
        }
        grid
    }

    #[test]
    fn char_text_extraction_single_line() {
        let grid = make_grid_with_text(&["Hello, World!"]);
        let s = Selection::new(
            SelectionAnchor { row: 0, col: 0 },
            SelectionAnchor { row: 0, col: 4 },
            SelectionMode::Char,
        );
        assert_eq!(s.text(&grid), "Hello");
    }

    #[test]
    fn char_text_extraction_multi_line() {
        let grid = make_grid_with_text(&["First line", "Second line"]);
        let s = Selection::new(
            SelectionAnchor { row: 0, col: 6 },
            SelectionAnchor { row: 1, col: 5 },
            SelectionMode::Char,
        );
        let result = s.text(&grid);
        // Space at end of "line " is part of the selected area (col 9 <= hi.col=9 on row 0)
        assert!(
            result.starts_with("line"),
            "should extract 'line' from first row, got: {result:?}"
        );
        assert!(
            result.ends_with("Second"),
            "should end with 'Second' from second row, got: {result:?}"
        );
        assert!(
            result.contains('\n'),
            "should have newline between rows, got: {result:?}"
        );
    }

    #[test]
    fn word_text_extraction() {
        let grid = make_grid_with_text(&["Hello World"]);
        let s = Selection::new(
            SelectionAnchor { row: 0, col: 0 },
            SelectionAnchor { row: 0, col: 4 },
            SelectionMode::Word,
        );
        assert_eq!(s.text(&grid), "Hello");
    }

    #[test]
    fn line_text_extraction() {
        let grid = make_grid_with_text(&["Hello", "World", "Test"]);
        let s = Selection::new(
            SelectionAnchor { row: 1, col: 2 },
            SelectionAnchor { row: 2, col: 1 },
            SelectionMode::Line,
        );
        let result = s.text(&grid);
        assert_eq!(result, "World\nTest");
    }

    #[test]
    fn block_text_extraction() {
        let grid = make_grid_with_text(&["ABCDEFGHIJ", "0123456789"]);
        let s = Selection::new(
            SelectionAnchor { row: 0, col: 2 },
            SelectionAnchor { row: 1, col: 5 },
            SelectionMode::Block,
        );
        assert_eq!(s.text(&grid), "CDEF\n2345");
    }

    #[test]
    fn block_text_extraction_single_cell() {
        let grid = make_grid_with_text(&["ABCD"]);
        let s = Selection::new(
            SelectionAnchor { row: 0, col: 1 },
            SelectionAnchor { row: 0, col: 1 },
            SelectionMode::Block,
        );
        assert_eq!(s.text(&grid), "B");
    }

    #[test]
    fn text_extraction_reversed_selection() {
        let grid = make_grid_with_text(&["ABC"]);
        let s = Selection::new(
            SelectionAnchor { row: 0, col: 3 },
            SelectionAnchor { row: 0, col: 0 },
            SelectionMode::Char,
        );
        assert_eq!(s.text(&grid), "ABC");
    }
}
