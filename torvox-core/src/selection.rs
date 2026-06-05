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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct SelectionAnchor {
    pub row: u32,
    pub col: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
        // 词模式使用与字符模式相同的包含逻辑
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
}
