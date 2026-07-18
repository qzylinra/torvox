//! Selection computation for the bridge layer.
//!
//! The bridge's selection boltffi methods all share the same shape: take a
//! terminal snapshot, read cell codepoints through a `cell_at` closure, expand a
//! `Selection` to its full extent, and build a GPU `SelectionRange`. This module
//! owns that pure computation so it can be unit-tested without a `TorvoxBridge`,
//! an `AndroidSurface`, or a GPU context — improving locality (selection math in
//! one place) and leverage (snapshot-driven tests).

use torvox_core::selection::{Selection, SelectionAnchor, SelectionMode};
use torvox_renderer::gpu::SelectionRange;
use torvox_terminal::ghostty_terminal::CellSnapshot;

/// Build a closure that reads the primary codepoint of a grid cell.
///
/// Lives here so the two call sites (`expand_and_set_selection`,
/// `set_selection_endpoint`) share one definition instead of re-deriving it.
pub(crate) fn cell_at_fn(
    cells: &[CellSnapshot],
    cols: u32,
) -> impl Fn(u32, u32) -> Option<char> + '_ {
    move |row, col| {
        let idx = (row * cols + col) as usize;
        cells.get(idx).and_then(|c| char::from_u32(c.codepoint))
    }
}

/// Expand a selection anchored at `anchor` to its full extent for `mode`.
///
/// Pure: given a cell buffer and a starting anchor, returns the ordered
/// (start, end) anchors. No surface, session, or GPU access required.
pub(crate) fn expand_at(
    cells: &[CellSnapshot],
    cols: u32,
    anchor: SelectionAnchor,
    mode: SelectionMode,
) -> (SelectionAnchor, SelectionAnchor) {
    let cell_at = cell_at_fn(cells, cols);
    Selection::new(anchor, anchor, mode)
        .expand(cell_at)
        .ordered()
}

/// Construct a GPU `SelectionRange` from two ordered anchors.
pub(crate) fn range_from(
    start: SelectionAnchor,
    end: SelectionAnchor,
    mode: SelectionMode,
    origin: Option<(i32, i32)>,
) -> SelectionRange {
    SelectionRange {
        start_row: start.row as i32,
        start_col: start.col as i32,
        end_row: end.row as i32,
        end_col: end.col as i32,
        active: true,
        mode,
        origin,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use torvox_core::selection::SelectionMode;

    fn snapshot_with(text: &str) -> (Vec<CellSnapshot>, u32) {
        let cols = text.chars().count() as u32;
        let cells = text
            .chars()
            .map(|c| CellSnapshot {
                codepoint: c as u32,
                ..Default::default()
            })
            .collect();
        (cells, cols)
    }

    #[test]
    fn expand_word_stops_at_space() {
        let (cells, cols) = snapshot_with("hello world");
        let anchor = SelectionAnchor { row: 0, col: 2 };
        let (start, end) = expand_at(&cells, cols, anchor, SelectionMode::Word);
        assert_eq!(start, SelectionAnchor { row: 0, col: 0 });
        assert_eq!(end, SelectionAnchor { row: 0, col: 4 });
    }

    #[test]
    fn expand_char_is_single_cell() {
        let (cells, cols) = snapshot_with("abc");
        let anchor = SelectionAnchor { row: 0, col: 1 };
        let (start, end) = expand_at(&cells, cols, anchor, SelectionMode::Char);
        assert_eq!(start, anchor);
        assert_eq!(end, anchor);
    }

    #[test]
    fn range_from_maps_anchors_to_i32() {
        let range = range_from(
            SelectionAnchor { row: 1, col: 2 },
            SelectionAnchor { row: 3, col: 4 },
            SelectionMode::Line,
            Some((0, 0)),
        );
        assert_eq!(range.start_row, 1);
        assert_eq!(range.start_col, 2);
        assert_eq!(range.end_row, 3);
        assert_eq!(range.end_col, 4);
        assert_eq!(range.mode, SelectionMode::Line);
        assert_eq!(range.origin, Some((0, 0)));
    }
}
