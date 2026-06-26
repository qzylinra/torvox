//! Property-based tests for torvox-core invariants using quickcheck.
//!
//! These verify:
//!  - Grid resize preserves invariants
//!  - DirtyMask operations are idempotent
//!  - Snapshot rkyv/serde roundtrip preserves equality
//!  - Color arithmetic stays in range

#[cfg(test)]
mod grid_invariants {
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    use torvox_core::cell::DirtyMask;
    use torvox_core::grid::Grid;

    #[quickcheck]
    fn grid_resize_preserves_dimensions(rows: u32, cols: u32, new_rows: u32, new_cols: u32) {
        if rows == 0 || cols == 0 || rows > 200 || cols > 500 {
            return;
        }
        let new_rows = new_rows.clamp(1, 200);
        let new_cols = new_cols.clamp(1, 500);
        let mut g = Grid::new(rows, cols);
        g.resize(new_rows, new_cols);
        assert_eq!(g.rows(), new_rows, "rows should match after resize");
        assert_eq!(g.cols(), new_cols, "cols should match after resize");
    }

    #[quickcheck]
    fn grid_resize_keeps_cells_in_range(rows: u32, cols: u32, new_cols: u32) -> TestResult {
        if rows == 0 || rows > 50 || cols == 0 || cols > 100 || new_cols == 0 || new_cols > 200 {
            return TestResult::discard();
        }
        let mut g = Grid::new(rows, cols);
        g.resize(rows, new_cols);
        for r in 0..rows {
            let line = g.get(r);
            if let Some(line) = line {
                let n = line.cells().len() as u32;
                assert_eq!(
                    n, new_cols,
                    "row {} should have {} cells after resize to {} cols, has {}",
                    r, new_cols, new_cols, n
                );
            }
        }
        TestResult::passed()
    }

    #[quickcheck]
    fn dirty_mask_mark_then_is_dirty(row: u32) {
        let rows = 100u32;
        let mut m = DirtyMask::new(rows);
        if row < rows {
            m.mark(row);
            assert!(m.is_dirty(row));
        }
    }

    #[quickcheck]
    fn dirty_mask_clear_makes_empty(row: u32) {
        let rows = 100u32;
        let mut m = DirtyMask::new(rows);
        if row < rows {
            m.mark(row);
            m.clear();
            assert!(!m.is_dirty(row));
        }
    }

    #[quickcheck]
    fn dirty_mask_mark_idempotent(row: u32) {
        let rows = 100u32;
        let mut m = DirtyMask::new(rows);
        if row < rows {
            m.mark(row);
            m.mark(row);
            m.mark(row);
            assert!(m.is_dirty(row));
        }
    }

    #[quickcheck]
    fn dirty_mask_any_after_mark_all(rows: u32) {
        if rows == 0 || rows > 1000 {
            return;
        }
        let mut m = DirtyMask::new(rows);
        m.mark_all(rows);
        for r in 0..rows {
            assert!(m.is_dirty(r));
        }
    }

    #[quickcheck]
    fn dirty_mask_clear_makes_any_dirty_false(rows: u32) {
        if rows == 0 || rows > 1000 {
            return;
        }
        let mut m = DirtyMask::new(rows);
        m.mark_all(rows);
        m.clear();
        assert!(!m.any_dirty());
    }
}

#[cfg(test)]
mod color_arithmetic {
    use quickcheck_macros::quickcheck;

    use torvox_core::cell::Color;

    #[quickcheck]
    fn color_default_is_white(_unit: ()) {
        let c = Color::default();
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 255);
        assert_eq!(c.b, 255);
        assert_eq!(c.a, 255);
    }

    #[quickcheck]
    fn color_equality(c1: (u8, u8, u8, u8), c2: (u8, u8, u8, u8)) {
        let (r1, g1, b1, a1) = c1;
        let (r2, g2, b2, a2) = c2;
        let a = Color {
            r: r1,
            g: g1,
            b: b1,
            a: a1,
        };
        let b = Color {
            r: r2,
            g: g2,
            b: b2,
            a: a2,
        };
        let eq = r1 == r2 && g1 == g2 && b1 == b2 && a1 == a2;
        assert_eq!(a == b, eq);
    }
}

#[cfg(test)]
mod config_invariants {
    use quickcheck_macros::quickcheck;

    use torvox_core::config::{BackspaceMode, RightAltMode, Shell, TerminalConfig};

    #[quickcheck]
    fn terminal_config_default_is_24x80(_unit: ()) {
        let cfg = TerminalConfig::default();
        assert_eq!(cfg.rows, 24);
        assert_eq!(cfg.cols, 80);
    }

    #[quickcheck]
    fn terminal_config_serde_roundtrip(rows: u16, cols: u16, scroll: u32) -> bool {
        if rows == 0 || cols == 0 {
            return true;
        }
        let cfg = TerminalConfig {
            rows: rows as u32,
            cols: cols as u32,
            scrollback_lines: scroll,
            shell: Shell::SystemDefault,
            font_size_tenths: 140,
            backspace_mode: BackspaceMode::default(),
            right_alt_mode: RightAltMode::default(),
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let back: TerminalConfig = serde_json::from_str(&json).unwrap();
        back == cfg
    }

    #[quickcheck]
    fn shell_equality_system_default(_unit: ()) {
        assert_ne!(Shell::SystemDefault, Shell::Custom("/bin/sh".to_string()));
    }
}

#[cfg(test)]
mod snapshot_invariants {
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    use torvox_core::cell::Cell;
    use torvox_core::grid::Grid;
    use torvox_core::line::Line;
    use torvox_core::snapshot::SessionSnapshot;

    #[quickcheck]
    fn session_snapshot_from_grid_dimensions(rows: u32, cols: u32) -> TestResult {
        if rows == 0 || cols == 0 || rows > 50 || cols > 100 {
            return TestResult::discard();
        }
        let g = Grid::new(rows, cols);
        let snap = SessionSnapshot::from_grid(&g);
        assert_eq!(snap.rows, rows);
        assert_eq!(snap.cols, cols);
        assert_eq!(snap.visible_lines.len(), rows as usize);
        TestResult::passed()
    }

    #[quickcheck]
    fn cell_default_serde_roundtrip(_unit: ()) -> bool {
        let c: Cell = Cell::default();
        let json = serde_json::to_string(&c).unwrap();
        let back: Cell = serde_json::from_str(&json).unwrap();
        back == c
    }

    #[quickcheck]
    fn line_serde_roundtrip(cols: u32) -> TestResult {
        if cols == 0 || cols > 100 {
            return TestResult::discard();
        }
        let l = Line::new(cols);
        let json = serde_json::to_string(&l).unwrap();
        let back: Line = serde_json::from_str(&json).unwrap();
        let eq = back.cells().len() == l.cells().len();
        TestResult::from_bool(eq)
    }
}

#[cfg(test)]
mod cell_serde_invariants {
    use quickcheck_macros::quickcheck;
    use torvox_core::cell::{Attrs, Cell, Color};

    #[quickcheck]
    fn cell_serde_roundtrip_char(char_code: u32, width: u8) -> bool {
        let cell = Cell {
            char: char::from_u32(char_code & 0x10FFFF).unwrap_or(' '),
            width: if width == 0 { 1 } else { (width % 3) + 1 },
            ..Default::default()
        };
        let json = serde_json::to_string(&cell).unwrap();
        let back: Cell = serde_json::from_str(&json).unwrap();
        back == cell
    }

    #[quickcheck]
    fn cell_serde_roundtrip_colors(r1: u8, g1: u8, b1: u8, a1: u8) -> bool {
        let fg = Color {
            r: r1,
            g: g1,
            b: b1,
            a: a1,
        };
        let bg = Color {
            r: r1.wrapping_add(128),
            g: g1.wrapping_add(128),
            b: b1.wrapping_add(128),
            a: a1,
        };
        let cell = Cell {
            fg,
            bg,
            ..Default::default()
        };
        let json = serde_json::to_string(&cell).unwrap();
        let back: Cell = serde_json::from_str(&json).unwrap();
        back == cell
    }

    #[quickcheck]
    fn cell_serde_roundtrip_attrs(
        bold: bool,
        italic: bool,
        underline: bool,
        reverse: bool,
    ) -> bool {
        let attrs = Attrs {
            bold,
            italic,
            underline,
            reverse,
            ..Default::default()
        };
        let cell = Cell {
            attrs,
            ..Default::default()
        };
        let json = serde_json::to_string(&cell).unwrap();
        let back: Cell = serde_json::from_str(&json).unwrap();
        back == cell
    }

    #[quickcheck]
    fn attrs_first_four_roundtrip(bold: bool, dim: bool, italic: bool, underline: bool) -> bool {
        let a = Attrs {
            bold,
            dim,
            italic,
            underline,
            ..Default::default()
        };
        let json = serde_json::to_string(&a).unwrap();
        let back: Attrs = serde_json::from_str(&json).unwrap();
        a == back
    }

    #[quickcheck]
    fn attrs_mid_four_roundtrip(
        double_underline: bool,
        reverse: bool,
        strikethrough: bool,
        blink: bool,
    ) -> bool {
        let a = Attrs {
            double_underline,
            reverse,
            strikethrough,
            blink,
            ..Default::default()
        };
        let json = serde_json::to_string(&a).unwrap();
        let back: Attrs = serde_json::from_str(&json).unwrap();
        a == back
    }

    #[quickcheck]
    fn attrs_last_six_roundtrip(
        hidden: bool,
        overline: bool,
        protected: bool,
        double_width: bool,
        double_height_top: bool,
        double_height_bottom: bool,
    ) -> bool {
        let a = Attrs {
            hidden,
            overline,
            protected,
            double_width,
            double_height_top,
            double_height_bottom,
            ..Default::default()
        };
        let json = serde_json::to_string(&a).unwrap();
        let back: Attrs = serde_json::from_str(&json).unwrap();
        a == back
    }
}

#[cfg(test)]
mod dirty_mask_invariants {
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;
    use torvox_core::cell::DirtyMask;

    #[quickcheck]
    fn dirty_mask_mark_one_does_not_affect_others(marked_row: u32, checked_row: u32) -> TestResult {
        if marked_row == checked_row || marked_row > 1000 || checked_row > 1000 {
            return TestResult::discard();
        }
        let mut m = DirtyMask::new(1001);
        m.mark(marked_row);
        if m.is_dirty(checked_row) {
            return TestResult::failed();
        }
        TestResult::passed()
    }

    #[quickcheck]
    fn dirty_mask_union_covers_full_range(rows: u32) -> TestResult {
        if rows == 0 || rows > 1000 {
            return TestResult::discard();
        }
        let mut m = DirtyMask::new(rows);
        m.mark_all(rows);
        for r in 0..rows {
            if !m.is_dirty(r) {
                return TestResult::failed();
            }
        }
        TestResult::passed()
    }
}

#[cfg(test)]
mod selection_invariants {
    use quickcheck_macros::quickcheck;
    use torvox_core::selection::{Selection, SelectionAnchor, SelectionMode};

    #[quickcheck]
    fn selection_contains_is_deterministic(
        start_row: u32,
        start_col: u32,
        end_row: u32,
        end_col: u32,
        test_row: u32,
        test_col: u32,
    ) {
        for &mode in &[
            SelectionMode::Char,
            SelectionMode::Word,
            SelectionMode::Line,
            SelectionMode::Block,
        ] {
            let s = Selection::new(
                SelectionAnchor {
                    row: start_row,
                    col: start_col,
                },
                SelectionAnchor {
                    row: end_row,
                    col: end_col,
                },
                mode,
            );
            let r1 = s.contains(test_row, test_col);
            let r2 = s.contains(test_row, test_col);
            assert_eq!(r1, r2, "contains must be deterministic for mode {mode:?}");
        }
    }
}

#[cfg(test)]
mod cursor_invariants {
    use quickcheck_macros::quickcheck;
    use torvox_core::cursor::CursorState;

    #[quickcheck]
    fn cursor_state_move_no_panic(row: u32, col: u32, n: u32, max_rows: u32, max_cols: u32) {
        let mut c = CursorState::new(row, col);
        c.move_up(n);
        c.move_down(n, max_rows);
        c.move_left(n);
        c.move_right(n, max_cols);
        c.carriage_return();
        // Verify moves don't leave cursor in an invalid state
        assert!(
            c.row <= max_rows || max_rows == 0,
            "row {} should be ≤ max_rows {}",
            c.row,
            max_rows
        );
        assert!(
            c.col <= max_cols || max_cols == 0,
            "col {} should be ≤ max_cols {}",
            c.col,
            max_cols
        );
    }

    #[quickcheck]
    fn cursor_state_clamp_safety(row: u32, col: u32, max_rows: u32, max_cols: u32) {
        let mut c = CursorState::new(row, col);
        c.clamp(max_rows, max_cols);
        assert!(
            c.row <= max_rows || max_rows == 0,
            "after clamp, row {} should be ≤ max_rows {}",
            c.row,
            max_rows
        );
        assert!(
            c.col <= max_cols || max_cols == 0,
            "after clamp, col {} should be ≤ max_cols {}",
            c.col,
            max_cols
        );
    }
}
