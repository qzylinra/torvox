use torvox_core::cell::Cell;
use torvox_core::grid::Grid;
use torvox_core::selection::{Selection, SelectionAnchor, SelectionMode};

fn make_grid(lines: &[&str]) -> Grid {
    let rows = lines.len() as u32;
    let cols = lines.iter().map(|l| l.len()).max().unwrap_or(1) as u32;
    let mut grid = Grid::new(rows, cols);
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

fn make_grid_with_nulls(lines: &[&[char]]) -> Grid {
    let rows = lines.len() as u32;
    let cols = lines.iter().map(|l| l.len()).max().unwrap_or(1) as u32;
    let mut grid = Grid::new(rows, cols);
    for (row_idx, line) in lines.iter().enumerate() {
        for (col_idx, ch) in line.iter().enumerate() {
            if let Some(cell) = grid.cell_mut(row_idx as u32, col_idx as u32) {
                *cell = Cell {
                    char: *ch,
                    ..Default::default()
                };
            }
        }
    }
    grid
}

#[test]
fn char_selection_trailing_spaces_not_included() {
    let grid = make_grid(&["Hello     "]);
    let s = Selection::new(
        SelectionAnchor { row: 0, col: 0 },
        SelectionAnchor { row: 0, col: 9 },
        SelectionMode::Char,
    );
    let result = s.text(&grid);
    assert!(
        result.starts_with("Hello"),
        "should start with 'Hello', got: {result:?}"
    );
    assert!(
        result.len() <= 9,
        "trailing spaces at end of last line should be trimmed, got {} chars: {result:?}",
        result.len()
    );
}

#[test]
fn char_selection_single_word_middle() {
    let grid = make_grid(&["Hello World"]);
    let s = Selection::new(
        SelectionAnchor { row: 0, col: 6 },
        SelectionAnchor { row: 0, col: 10 },
        SelectionMode::Char,
    );
    assert_eq!(s.text(&grid), "World");
}

#[test]
fn char_selection_reversed_single_line() {
    let grid = make_grid(&["Hello World"]);
    let s = Selection::new(
        SelectionAnchor { row: 0, col: 10 },
        SelectionAnchor { row: 0, col: 0 },
        SelectionMode::Char,
    );
    assert_eq!(s.text(&grid), "Hello World");
}

#[test]
fn char_selection_multi_line() {
    let grid = make_grid(&["First", "Second", "Third"]);
    let s = Selection::new(
        SelectionAnchor { row: 0, col: 2 },
        SelectionAnchor { row: 2, col: 3 },
        SelectionMode::Char,
    );
    let result = s.text(&grid);
    assert!(
        result.contains("rst"),
        "should contain 'rst', got: {result:?}"
    );
    assert!(
        result.contains("Secon"),
        "should contain 'Secon' (end col exclusive), got: {result:?}"
    );
    assert!(
        result.contains("Thir"),
        "should contain 'Thir', got: {result:?}"
    );
    let lines: Vec<&str> = result.split('\n').collect();
    assert_eq!(lines.len(), 3, "should have 3 lines, got: {result:?}");
}

#[test]
fn char_selection_entire_single_line() {
    let grid = make_grid(&["ABCDE"]);
    let s = Selection::new(
        SelectionAnchor { row: 0, col: 0 },
        SelectionAnchor { row: 0, col: 4 },
        SelectionMode::Char,
    );
    assert_eq!(s.text(&grid), "ABCDE");
}

#[test]
fn char_selection_single_cell() {
    let grid = make_grid(&["ABCDE"]);
    let s = Selection::new(
        SelectionAnchor { row: 0, col: 2 },
        SelectionAnchor { row: 0, col: 2 },
        SelectionMode::Char,
    );
    assert_eq!(s.text(&grid), "C");
}

#[test]
fn char_selection_preserves_internal_spaces() {
    let grid = make_grid(&["A B C D"]);
    let s = Selection::new(
        SelectionAnchor { row: 0, col: 0 },
        SelectionAnchor { row: 0, col: 6 },
        SelectionMode::Char,
    );
    assert_eq!(s.text(&grid), "A B C D");
}

#[test]
fn char_selection_null_chars_dropped() {
    let grid = make_grid_with_nulls(&[&['A', '\0', 'B', '\0', 'C']]);
    let s = Selection::new(
        SelectionAnchor { row: 0, col: 0 },
        SelectionAnchor { row: 0, col: 4 },
        SelectionMode::Char,
    );
    let result = s.text(&grid);
    assert_eq!(result, "ABC", "null chars should be dropped");
}

#[test]
fn line_selection_single_line() {
    let grid = make_grid(&["Hello World"]);
    let s = Selection::new(
        SelectionAnchor { row: 0, col: 5 },
        SelectionAnchor { row: 0, col: 10 },
        SelectionMode::Line,
    );
    assert_eq!(s.text(&grid), "Hello World");
}

#[test]
fn line_selection_multi_line() {
    let grid = make_grid(&["First", "Second", "Third"]);
    let s = Selection::new(
        SelectionAnchor { row: 0, col: 5 },
        SelectionAnchor { row: 1, col: 0 },
        SelectionMode::Line,
    );
    let result = s.text(&grid);
    assert_eq!(result, "First\nSecond");
}

#[test]
fn line_selection_all_three() {
    let grid = make_grid(&["AA", "BB", "CC"]);
    let s = Selection::new(
        SelectionAnchor { row: 0, col: 0 },
        SelectionAnchor { row: 2, col: 0 },
        SelectionMode::Line,
    );
    assert_eq!(s.text(&grid), "AA\nBB\nCC");
}

#[test]
fn line_selection_trims_trailing_spaces() {
    let grid = make_grid_with_nulls(&[&['A', 'B', ' ', ' ', '\0', '\0']]);
    let s = Selection::new(
        SelectionAnchor { row: 0, col: 0 },
        SelectionAnchor { row: 0, col: 5 },
        SelectionMode::Line,
    );
    assert_eq!(s.text(&grid), "AB");
}

#[test]
fn block_selection_single_column() {
    let grid = make_grid(&["ABC", "DEF", "GHI"]);
    let s = Selection::new(
        SelectionAnchor { row: 0, col: 1 },
        SelectionAnchor { row: 2, col: 1 },
        SelectionMode::Block,
    );
    assert_eq!(s.text(&grid), "B\nE\nH");
}

#[test]
fn block_selection_rectangle() {
    let grid = make_grid(&["ABCDEFGHIJ", "0123456789", "abcdefghij"]);
    let s = Selection::new(
        SelectionAnchor { row: 0, col: 2 },
        SelectionAnchor { row: 1, col: 5 },
        SelectionMode::Block,
    );
    assert_eq!(s.text(&grid), "CDEF\n2345");
}

#[test]
fn block_selection_3x3() {
    let grid = make_grid(&["ABCDEF", "GHIJKL", "MNOPQR"]);
    let s = Selection::new(
        SelectionAnchor { row: 0, col: 1 },
        SelectionAnchor { row: 2, col: 3 },
        SelectionMode::Block,
    );
    let result = s.text(&grid);
    assert_eq!(result, "BCD\nHIJ\nNOP", "got: {:?}", result);
}

#[test]
fn block_selection_single_cell() {
    let grid = make_grid(&["ABCD"]);
    let s = Selection::new(
        SelectionAnchor { row: 0, col: 2 },
        SelectionAnchor { row: 0, col: 2 },
        SelectionMode::Block,
    );
    assert_eq!(s.text(&grid), "C");
}

#[test]
fn block_selection_null_chars_become_spaces() {
    let grid = make_grid_with_nulls(&[&['A', '\0', 'C'], &['D', 'E', 'F']]);
    let s = Selection::new(
        SelectionAnchor { row: 0, col: 0 },
        SelectionAnchor { row: 1, col: 1 },
        SelectionMode::Block,
    );
    assert_eq!(s.text(&grid), "A \nDE");
}

#[test]
fn word_selection_same_as_char() {
    let grid = make_grid(&["Hello World"]);
    let s_word = Selection::new(
        SelectionAnchor { row: 0, col: 0 },
        SelectionAnchor { row: 0, col: 4 },
        SelectionMode::Word,
    );
    let s_char = Selection::new(
        SelectionAnchor { row: 0, col: 0 },
        SelectionAnchor { row: 0, col: 4 },
        SelectionMode::Char,
    );
    assert_eq!(s_word.text(&grid), s_char.text(&grid));
}

#[test]
fn selection_reversed_multi_line() {
    let grid = make_grid(&["First", "Second"]);
    let s = Selection::new(
        SelectionAnchor { row: 1, col: 4 },
        SelectionAnchor { row: 0, col: 2 },
        SelectionMode::Char,
    );
    let result = s.text(&grid);
    assert!(
        result.contains("rst"),
        "should contain 'rst', got: {result:?}"
    );
    assert!(
        result.contains("Secon"),
        "should contain 'Secon' (col 4 = 'n', not 'd'), got: {result:?}"
    );
}

#[test]
fn selection_out_of_bounds_row_returns_empty() {
    let grid = make_grid(&["AB"]);
    let s = Selection::new(
        SelectionAnchor { row: 0, col: 0 },
        SelectionAnchor { row: 0, col: 1 },
        SelectionMode::Char,
    );
    let result = s.text(&grid);
    assert_eq!(result, "AB");
}

#[test]
fn selection_col_beyond_line_length() {
    let grid = make_grid(&["AB"]);
    let s = Selection::new(
        SelectionAnchor { row: 0, col: 0 },
        SelectionAnchor { row: 0, col: 100 },
        SelectionMode::Char,
    );
    let result = s.text(&grid);
    assert_eq!(result, "AB");
}

#[test]
fn contains_boundary_left_edge() {
    let s = Selection::new(
        SelectionAnchor { row: 0, col: 5 },
        SelectionAnchor { row: 0, col: 10 },
        SelectionMode::Char,
    );
    assert!(s.contains(0, 5));
    assert!(!s.contains(0, 4));
}

#[test]
fn contains_boundary_right_edge() {
    let s = Selection::new(
        SelectionAnchor { row: 0, col: 5 },
        SelectionAnchor { row: 0, col: 10 },
        SelectionMode::Char,
    );
    assert!(s.contains(0, 10));
    assert!(!s.contains(0, 11));
}

#[test]
fn contains_multi_row_first_row_only_start() {
    let s = Selection::new(
        SelectionAnchor { row: 0, col: 5 },
        SelectionAnchor { row: 2, col: 3 },
        SelectionMode::Char,
    );
    assert!(s.contains(0, 5));
    assert!(s.contains(0, 79));
    assert!(!s.contains(0, 4));
    assert!(s.contains(1, 0));
    assert!(s.contains(2, 3));
    assert!(!s.contains(2, 4));
}

#[test]
fn block_contains_strict_bounds() {
    let s = Selection::new(
        SelectionAnchor { row: 1, col: 2 },
        SelectionAnchor { row: 3, col: 5 },
        SelectionMode::Block,
    );
    assert!(s.contains(1, 2));
    assert!(s.contains(1, 5));
    assert!(s.contains(3, 2));
    assert!(s.contains(3, 5));
    assert!(!s.contains(1, 1));
    assert!(!s.contains(1, 6));
    assert!(!s.contains(0, 3));
    assert!(!s.contains(4, 3));
}
