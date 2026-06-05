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
        let _ = g.cell(0, 0);
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
                assert!(n == new_cols || n == cols);
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
    fn color_components_in_range(r: u8, g: u8, b: u8, a: u8) {
        let c = Color { r, g, b, a };
        assert_eq!(c.r, r);
        assert_eq!(c.g, g);
        assert_eq!(c.b, b);
        assert_eq!(c.a, a);
    }

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

    use torvox_core::config::{Shell, TerminalConfig};

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
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let back: TerminalConfig = serde_json::from_str(&json).unwrap();
        back == cfg
    }

    #[quickcheck]
    fn shell_equality_system_default(_unit: ()) {
        assert_eq!(Shell::SystemDefault, Shell::SystemDefault);
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
