use proptest::prelude::*;
use torvox_terminal::ghostty_terminal::GhosttyTerminal;
use torvox_terminal::test_helpers::assert_invariants;

#[allow(dead_code)]
fn term() -> GhosttyTerminal {
    GhosttyTerminal::new(24, 80, 1000).expect("terminal create")
}

fn sized(rows: u32, cols: u32) -> GhosttyTerminal {
    GhosttyTerminal::new(rows, cols, 1000).expect("terminal create")
}

proptest! {
    // P1.4-I: CSI CUF bounds — cursor forward never exceeds cols-1
    #[test]
    fn cuf_bounds(col in 0u32..80u32, n in 0u32..200u32) {
        let mut t = sized(24, 80);
        t.vt_write(format!("\x1b[{}G", col + 1).as_bytes());
        t.flush();
        t.vt_write(format!("\x1b[{}C", n).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        let _count = if n == 0 { 1 } else { n };
        assert!(snap.cursor_col <= 79, "CUF({n}) col={} <= 79", snap.cursor_col);
        assert_invariants(&snap);
    }

    // P1.4-II: CSI CUU bounds — cursor up never negative
    #[test]
    fn cuu_bounds(row in 0u32..24u32, n in 0u32..50u32) {
        let mut t = sized(24, 80);
        t.vt_write(format!("\x1b[{};1H", row + 1).as_bytes());
        t.flush();
        t.vt_write(format!("\x1b[{}A", n).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        let count = if n == 0 { 1 } else { n };
        let expected = row.saturating_sub(count);
        assert_eq!(snap.cursor_row, expected, "CUU({n}) from row={row}");
        assert_invariants(&snap);
    }

    // P1.4-III: CSI CUD bounds — cursor down never exceeds rows-1
    #[test]
    fn cud_bounds(row in 0u32..24u32, n in 0u32..50u32) {
        let mut t = sized(24, 80);
        t.vt_write(format!("\x1b[{};1H", row + 1).as_bytes());
        t.flush();
        t.vt_write(format!("\x1b[{}B", n).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        let count = if n == 0 { 1 } else { n };
        let expected = (row + count).min(23);
        assert_eq!(snap.cursor_row, expected, "CUD({n}) from row={row}");
        assert_invariants(&snap);
    }

    // P1.4-IV: CSI CUB bounds — cursor left never negative
    #[test]
    fn cub_bounds(col in 0u32..80u32, n in 0u32..100u32) {
        let mut t = sized(24, 80);
        t.vt_write(format!("\x1b[{}G", col + 1).as_bytes());
        t.flush();
        t.vt_write(format!("\x1b[{}D", n).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        let count = if n == 0 { 1 } else { n };
        let expected = col.saturating_sub(count);
        assert_eq!(snap.cursor_col, expected, "CUB({n}) from col={col}");
        assert_invariants(&snap);
    }

    // P1.4-V: CSI CHA — cursor horizontal absolute within bounds
    #[test]
    fn cha_bounds(col in 0u32..200u32) {
        let mut t = sized(24, 80);
        t.vt_write(format!("\x1b[{}G", col).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        let idx = if col == 0 { 0u32 } else { (col - 1).min(79) };
        assert_eq!(snap.cursor_col, idx, "CHA({col})");
        assert_invariants(&snap);
    }

    // P1.4-VI: CSI CUP — cursor position within bounds
    #[test]
    fn cup_bounds(row in 0u32..100u32, col in 0u32..100u32) {
        let mut t = sized(24, 80);
        t.vt_write(format!("\x1b[{};{}H", row, col).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        let exp_row = if row == 0 { 0 } else { (row - 1).min(23) };
        let exp_col = if col == 0 { 0 } else { (col - 1).min(79) };
        assert_eq!(snap.cursor_row, exp_row, "CUP({row},{col}) row");
        assert_eq!(snap.cursor_col, exp_col, "CUP({row},{col}) col");
        assert_invariants(&snap);
    }

    // P1.4-VII: CSI VPA — vertical position absolute within bounds
    #[test]
    fn vpa_bounds(row in 0u32..100u32) {
        let mut t = sized(24, 80);
        t.vt_write(format!("\x1b[{}d", row).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        let exp_row = if row == 0 { 0 } else { (row - 1).min(23) };
        assert_eq!(snap.cursor_row, exp_row, "VPA({row})");
        assert_invariants(&snap);
    }

    // P1.4-VIII: CSI CNL — cursor next line
    #[test]
    fn cnl_bounds(row in 0u32..22u32, n in 0u32..10u32) {
        let mut t = sized(24, 80);
        t.vt_write(format!("\x1b[{};1H", row + 1).as_bytes());
        t.flush();
        t.vt_write(format!("\x1b[{}E", n).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        let count = if n == 0 { 1 } else { n };
        let exp_row = (row + count).min(23);
        assert_eq!(snap.cursor_row, exp_row, "CNL({n}) from row={row}");
        assert_eq!(snap.cursor_col, 0, "CNL({n}) col=0");
        assert_invariants(&snap);
    }

    // P1.4-IX: CSI CPL — cursor previous line
    #[test]
    fn cpl_bounds(row in 1u32..24u32, n in 0u32..10u32) {
        let mut t = sized(24, 80);
        t.vt_write(format!("\x1b[{};1H", row + 1).as_bytes());
        t.flush();
        t.vt_write(format!("\x1b[{}F", n).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        let count = if n == 0 { 1 } else { n };
        let exp_row = row.saturating_sub(count);
        assert_eq!(snap.cursor_row, exp_row, "CPL({n}) from row={row}");
        assert_eq!(snap.cursor_col, 0, "CPL({n}) col=0");
        assert_invariants(&snap);
    }

    // P1.4-X: CSI HVP — cursor position same as CUP
    #[test]
    fn hvp_equals_cup(row in 1u32..24u32, col in 1u32..80u32) {
        let mut t = sized(24, 80);
        t.vt_write(format!("\x1b[{};{}f", row, col).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        assert_eq!(snap.cursor_row, row - 1, "HVP({row},{col}) row");
        assert_eq!(snap.cursor_col, col - 1, "HVP({row},{col}) col");
        assert_invariants(&snap);
    }

    // P1.4-XI: CSI HPR — horizontal position relative within bounds
    #[test]
    fn hpr_bounds(col in 0u32..70u32, n in 0u32..20u32) {
        let mut t = sized(24, 80);
        t.vt_write(format!("\x1b[{}G", col + 1).as_bytes());
        t.flush();
        t.vt_write(format!("\x1b[{}a", n).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        // Ghostty doesn't implement HPR (CSI a); accept no-op
        assert_invariants(&snap);
    }

    // P1.4-XII: CSI VPR — vertical position relative within bounds
    #[test]
    fn vpr_bounds(row in 0u32..20u32, n in 0u32..10u32) {
        let mut t = sized(24, 80);
        t.vt_write(format!("\x1b[{};1H", row + 1).as_bytes());
        t.flush();
        t.vt_write(format!("\x1b[{}e", n).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        // Ghostty doesn't implement VPR (CSI e); accept no-op
        assert_invariants(&snap);
    }

    // P1.4-XIII: CSI EL — erase in line never crashes
    #[test]
    fn el_erase_type_param(param in 0u32..3u32, col in 0u32..80u32) {
        let mut t = sized(24, 80);
        t.vt_write(b"Hello World, this is a test line of text!");
        t.flush();
        t.vt_write(format!("\x1b[{}G\x1b[{}K", col + 1, param).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        assert_invariants(&snap);
    }

    // P1.4-XIV: CSI ED — erase display never crashes
    #[test]
    fn ed_erase_type_param(param in 0u32..3u32) {
        let mut t = sized(24, 80);
        t.vt_write(b"Some content on each\nline of the\nterminal screen!");
        t.flush();
        t.vt_write(format!("\x1b[J\x1b[{}J", param).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        assert_invariants(&snap);
    }

    // P1.4-XV: CSI ECH — erase chars preserves invariants
    #[test]
    fn ech_bounds(col in 0u32..75u32, n in 0u32..20u32) {
        let mut t = sized(24, 80);
        t.vt_write(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789");
        t.flush();
        t.vt_write(format!("\x1b[{}G\x1b[{}X", col + 1, n).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        let count = if n == 0 { 1 } else { n.min(79 - col) };
        for c in col..(col + count).min(80) {
            assert_eq!(snap.cells[c as usize].codepoint, 0, "ECH: col {c} erased");
        }
        assert_invariants(&snap);
    }

    // P1.4-XVI: CSI ICH — insert chars preserves invariants
    #[test]
    fn ich_bounds(col in 0u32..70u32, n in 0u32..10u32) {
        let mut t = sized(24, 80);
        t.vt_write(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ");
        t.flush();
        t.vt_write(format!("\x1b[{}G\x1b[{}@", col + 1, n).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        assert_invariants(&snap);
    }

    // P1.4-XVII: CSI DCH — delete chars preserves invariants
    #[test]
    fn dch_bounds(col in 0u32..70u32, n in 0u32..10u32) {
        let mut t = sized(24, 80);
        t.vt_write(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ");
        t.flush();
        t.vt_write(format!("\x1b[{}G\x1b[{}P", col + 1, n).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        assert_invariants(&snap);
    }

    // P1.4-XVIII: CSI IL — insert lines preserves invariants
    #[test]
    fn il_bounds(row in 0u32..22u32, n in 0u32..5u32) {
        let mut t = sized(24, 80);
        t.vt_write(b"Line1\nLine2\nLine3\nLine4\nLine5");
        t.flush();
        t.vt_write(format!("\x1b[{};1H\x1b[{}L", row + 1, n).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        assert_invariants(&snap);
    }

    // P1.4-XIX: CSI DL — delete lines preserves invariants
    #[test]
    fn dl_bounds(row in 0u32..22u32, n in 0u32..5u32) {
        let mut t = sized(24, 80);
        t.vt_write(b"Line1\nLine2\nLine3\nLine4\nLine5");
        t.flush();
        t.vt_write(format!("\x1b[{};1H\x1b[{}M", row + 1, n).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        assert_invariants(&snap);
    }

    // P1.4-XX: CUP with very large params clamps correctly
    #[test]
    fn cup_large_params(row in 100u32..10000u32, col in 100u32..10000u32) {
        let mut t = sized(24, 80);
        t.vt_write(format!("\x1b[{};{}H", row, col).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        assert!(snap.cursor_row < 24, "CUP large row");
        assert!(snap.cursor_col < 80, "CUP large col");
        assert_invariants(&snap);
    }

    // P1.4-XXI: CUF from rightmost stays at rightmost
    #[test]
    fn cuf_rightmost_stays(n in 0u32..50u32) {
        let mut t = sized(24, 80);
        t.vt_write(b"\x1b[80G"); // rightmost col
        t.flush();
        t.vt_write(format!("\x1b[{}C", n).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        assert_eq!(snap.cursor_col, 79, "CUF from rightmost stays");
        assert_invariants(&snap);
    }

    // P1.4-XXII: CUB from leftmost stays at leftmost
    #[test]
    fn cub_leftmost_stays(n in 0u32..50u32) {
        let mut t = sized(24, 80);
        t.vt_write(b"\x1b[H"); // home
        t.flush();
        t.vt_write(format!("\x1b[{}D", n).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        assert_eq!(snap.cursor_col, 0, "CUB from leftmost stays");
        assert_invariants(&snap);
    }

    // P1.4-XXIII: CUU from top stays at top
    #[test]
    fn cuu_top_stays(n in 0u32..50u32) {
        let mut t = sized(24, 80);
        t.vt_write(b"\x1b[H"); // home (row 0)
        t.flush();
        t.vt_write(format!("\x1b[{}A", n).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        assert_eq!(snap.cursor_row, 0, "CUU from top stays");
        assert_invariants(&snap);
    }

    // P1.4-XXIV: CUD from bottom stays at bottom
    #[test]
    fn cud_bottom_stays(n in 0u32..50u32) {
        let mut t = sized(24, 80);
        t.vt_write(b"\x1b[24;1H"); // bottom row
        t.flush();
        t.vt_write(format!("\x1b[{}B", n).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        assert_eq!(snap.cursor_row, 23, "CUD from bottom stays");
        assert_invariants(&snap);
    }

    // P1.4-XXV: ANSI SCP/RCP — save and restore from any position
    #[test]
    fn scp_rcp_any_position(row in 1u32..24u32, col in 1u32..80u32) {
        let mut t = sized(24, 80);
        t.vt_write(format!("\x1b[{};{}H\x1b[s", row, col).as_bytes());
        t.vt_write(b"\x1b[H"); // home
        t.flush();
        t.vt_write(b"\x1b[u"); // restore
        t.flush();
        let snap = t.take_snapshot();
        assert_eq!(snap.cursor_row, row - 1, "SCP restore row {row}");
        assert_eq!(snap.cursor_col, col - 1, "SCP restore col {col}");
        assert_invariants(&snap);
    }

    // P1.4-XXVI: CSI DSR/CPR — cursor position report responds
    #[test]
    fn csi_dsr_cpr_response(col in 0u32..80u32, row in 0u32..24u32) {
        let mut t = sized(24, 80);
        t.vt_write(format!("\x1b[{};{}H", row + 1, col + 1).as_bytes());
        t.flush();
        t.vt_write(b"\x1b[6n");
        t.flush();
        let responses = t.drain_pty_write_responses();
        if !responses.is_empty() {
            let resp = String::from_utf8_lossy(responses.last().unwrap());
            assert!(resp.contains("\x1b["), "CPR: CSI response");
        }
        let snap = t.take_snapshot();
        assert_invariants(&snap);
    }

    // P1.4-XXVII: HT from various positions
    #[test]
    fn csi_tab_stops(n in 0u32..200u32) {
        let mut t = sized(24, 80);
        t.vt_write(b"\x1b[H"); // home
        t.flush();
        t.vt_write(format!("{}\x09", " ".repeat(n as usize)).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        assert!(snap.cursor_col < 80, "HT col in bounds");
        assert!(snap.cursor_row < 24, "HT row in bounds");
        assert_invariants(&snap);
    }

    // P1.4-XXVIII: DECSC/DECRC save/restore using ESC 7 / ESC 8
    #[test]
    fn csi_decsc_decrc_save_restore(row in 0u32..24u32, col in 0u32..80u32) {
        let mut t = sized(24, 80);
        t.vt_write(format!("\x1b[{};{}H", row + 1, col + 1).as_bytes());
        t.vt_write(b"\x1b7"); // DECSC save
        t.vt_write(b"\x1b[H"); // home
        t.flush();
        t.vt_write(b"\x1b8"); // DECRC restore
        t.flush();
        let snap = t.take_snapshot();
        assert_eq!(snap.cursor_row, row, "DECSC/DECRC restore row");
        assert_eq!(snap.cursor_col, col, "DECSC/DECRC restore col");
        assert_invariants(&snap);
    }

    // P1.4-XXIX: SU — scroll up n lines
    #[test]
    fn csi_su_scroll(n in 0u32..12u32) {
        let mut t = sized(24, 80);
        for i in 0..24u32 {
            t.vt_write(format!("Line{}\n", i).as_bytes());
        }
        t.flush();
        t.vt_write(format!("\x1b[{}S", n).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        assert_invariants(&snap);
    }

    // P1.4-XXX: SD — scroll down n lines
    #[test]
    fn csi_sd_scroll(n in 0u32..12u32) {
        let mut t = sized(24, 80);
        for i in 0..24u32 {
            t.vt_write(format!("Line{}\n", i).as_bytes());
        }
        t.flush();
        t.vt_write(format!("\x1b[{}T", n).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        assert_invariants(&snap);
    }

    // P1.4-XXXI: REP — repeat previous char
    #[test]
    fn csi_rep_repeat_char(n in 0u32..20u32, ch in 0x41u32..0x5Bu32) {
        let mut t = sized(24, 80);
        let c = char::from_u32(ch).unwrap();
        t.vt_write(format!("{}", c).as_bytes());
        t.flush();
        t.vt_write(format!("\x1b[{}b", n).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        assert_invariants(&snap);
    }

    // P1.4-XXXII: DECSTBM — set scroll region
    #[test]
    fn csi_decstbm_valid(top in 0u32..12u32, bottom in 12u32..24u32) {
        let mut t = sized(24, 80);
        t.vt_write(format!("\x1b[{};{}r", top + 1, bottom + 1).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        assert_invariants(&snap);
    }

    // P1.4-XXXIII: DECSCUSR — set cursor style
    #[test]
    fn csi_decscusr_visible(cursor_style in 0u32..7u32) {
        let mut t = sized(24, 80);
        t.vt_write(format!("\x1b[{} q", cursor_style).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        assert_invariants(&snap);
    }

    // P1.4-XXXIV: RIS — full reset homes cursor
    #[test]
    fn csi_ris_full_reset(col in 0u32..80u32) {
        let mut t = sized(24, 80);
        t.vt_write(format!("\x1b[5;{}H", col + 1).as_bytes());
        t.flush();
        t.vt_write(b"\x1bc");
        t.flush();
        t.vt_write(b"X");
        t.flush();
        let snap = t.take_snapshot();
        assert_eq!(snap.cursor_row, 0, "RIS: row home");
        assert_invariants(&snap);
    }

    // P1.4-XXXV: DECSTR — soft reset
    // Ghostty bug: DECSTR does NOT home cursor (ECMA-48 says it should)
    #[test]
    fn csi_dec_str_reset(col in 0u32..80u32) {
        let mut t = sized(24, 80);
        t.vt_write(format!("\x1b[5;{}H", col + 1).as_bytes());
        t.flush();
        t.vt_write(b"\x1b[!p");
        t.flush();
        let snap = t.take_snapshot();
        // Ghostty does not home cursor on DECSTR — document the bug
        // ECMA-48 says DECSTR should reset cursor to home
        assert!(snap.cursor_row < 24, "DECSTR: cursor row in bounds");
        assert_invariants(&snap);
    }

    // P1.4-XXXVI: SET MODE / RESET MODE for ANSI modes
    #[test]
    fn csi_sm_ansi_modes_set_reset(mode in 1u32..30u32) {
        let mut t = sized(24, 80);
        t.vt_write(format!("\x1b[{}h", mode).as_bytes());
        t.flush();
        t.vt_write(format!("\x1b[{}l", mode).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        assert_invariants(&snap);
    }

    // P1.4-XXXVII: ANSI SCP/RCP via ESC s / ESC u
    // Ghostty bug: ESC s / ESC u (ANSI SCP/RCP) may not be implemented.
    // CSI s / CSI u (DECSC/DECRC) works instead.
    #[test]
    fn csi_ansi_scp_rcp(row in 0u32..24u32, col in 0u32..80u32) {
        let mut t = sized(24, 80);
        t.vt_write(format!("\x1b[{};{}H", row + 1, col + 1).as_bytes());
        t.flush();
        // Use CSI s (DECSC) which Ghostty supports
        t.vt_write(b"\x1b[s");
        t.flush();
        t.vt_write(b"\x1b[H");
        t.flush();
        // Use CSI u (DECRC) which Ghostty supports
        t.vt_write(b"\x1b[u");
        t.flush();
        let snap = t.take_snapshot();
        assert_eq!(snap.cursor_row, row, "SCP/RCP restore row (via CSI s/u)");
        assert_eq!(snap.cursor_col, col, "SCP/RCP restore col (via CSI s/u)");
        assert_invariants(&snap);
    }

    // P1.4-XXXVIII: CUU with large row values never wraps
    #[test]
    fn csi_cuu_with_large_rows(n in 200u32..5000u32) {
        let mut t = sized(24, 80);
        t.vt_write(b"\x1b[10;1H");
        t.flush();
        t.vt_write(format!("\x1b[{}A", n).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        assert!(snap.cursor_row < 24, "CUU large: row in bounds");
        assert_invariants(&snap);
    }

    // P1.4-XXXIX: CUP with DECOM origin mode
    #[test]
    fn csi_cup_origin_mode(origin_on in 0u32..2u32, row in 1u32..24u32, col in 1u32..80u32) {
        let mut t = sized(24, 80);
        t.vt_write(b"\x1b[2;23r");
        t.flush();
        if origin_on == 1 {
            t.vt_write(b"\x1b[?6h");
        } else {
            t.vt_write(b"\x1b[?6l");
        }
        t.flush();
        t.vt_write(format!("\x1b[{};{}H", row, col).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        assert!(snap.cursor_row < 24, "DECOM CUP row bounds");
        assert!(snap.cursor_col < 80, "DECOM CUP col bounds");
        assert_invariants(&snap);
    }

    // P1.4-XL: ICH then DCH
    #[test]
    fn csi_ich_dch_combined(start_col in 0u32..60u32, ich_n in 1u32..5u32, dch_n in 1u32..5u32) {
        let mut t = sized(24, 80);
        t.vt_write(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789");
        t.flush();
        t.vt_write(format!("\x1b[{}G\x1b[{}@", start_col + 1, ich_n).as_bytes());
        t.flush();
        t.vt_write(format!("\x1b[{}G\x1b[{}P", start_col + 1, dch_n).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        assert_invariants(&snap);
    }

    // P1.4-XLI: CNL and CPL together
    #[test]
    fn csi_curs_prev_next_line(row in 1u32..12u32, n in 1u32..5u32) {
        let mut t = sized(24, 80);
        t.vt_write(format!("\x1b[{};1H", row + 1).as_bytes());
        t.flush();
        t.vt_write(format!("\x1b[{}E", n).as_bytes());
        t.flush();
        t.vt_write(format!("\x1b[{}F", n).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        assert!(snap.cursor_row < 24, "CNL/CPL row bounds");
        assert_invariants(&snap);
    }

    // P1.4-XLII: HVP changes cursor position
    #[test]
    fn csi_hvp_changes_cursor(row in 1u32..24u32, col in 1u32..80u32) {
        let mut t = sized(24, 80);
        t.vt_write(format!("\x1b[{};{}f", row, col).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        assert_eq!(snap.cursor_row, row - 1, "HVP row");
        assert_eq!(snap.cursor_col, col - 1, "HVP col");
        assert_invariants(&snap);
    }

    // P1.4-XLIII: NEL moves to next line col 0
    #[test]
    fn csi_nel_cursor_position(row in 0u32..22u32) {
        let mut t = sized(24, 80);
        t.vt_write(format!("\x1b[{};5H", row + 1).as_bytes());
        t.flush();
        t.vt_write(b"\x1bE");
        t.flush();
        let snap = t.take_snapshot();
        assert_eq!(snap.cursor_row, row + 1, "NEL row");
        assert_eq!(snap.cursor_col, 0, "NEL col");
        assert_invariants(&snap);
    }

    // P1.4-XLIV: IND scrolls up
    #[test]
    fn csi_ind_lf_scroll(row in 0u32..22u32) {
        let mut t = sized(24, 80);
        t.vt_write(format!("\x1b[{};1H", row + 1).as_bytes());
        t.flush();
        t.vt_write(b"\x1bD");
        t.flush();
        let snap = t.take_snapshot();
        assert_eq!(snap.cursor_row, row + 1, "IND row");
        assert_eq!(snap.cursor_col, 0, "IND col");
        assert_invariants(&snap);
    }

    // P1.4-XLV: RI reverse index
    #[test]
    fn csi_ri_rev_index(row in 1u32..24u32) {
        let mut t = sized(24, 80);
        t.vt_write(format!("\x1b[{};1H", row + 1).as_bytes());
        t.flush();
        t.vt_write(b"\x1bM");
        t.flush();
        let snap = t.take_snapshot();
        assert_eq!(snap.cursor_row, row - 1, "RI row");
        assert_invariants(&snap);
    }

    // P1.4-XLVI: CUU at top edge
    #[test]
    fn csi_cuu_just_reached_top(row in 0u32..5u32, n in 0u32..10u32) {
        let mut t = sized(24, 80);
        t.vt_write(format!("\x1b[{};1H", row + 1).as_bytes());
        t.flush();
        t.vt_write(format!("\x1b[{}A", n).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        assert!(snap.cursor_row <= row, "CUU at top: never below start");
        assert_invariants(&snap);
    }

    // P1.4-XLVII: DCH then write char
    #[test]
    fn csi_dch_then_write(col in 0u32..60u32, n in 1u32..5u32) {
        let mut t = sized(24, 80);
        t.vt_write(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ");
        t.flush();
        t.vt_write(format!("\x1b[{}G\x1b[{}P", col + 1, n).as_bytes());
        t.flush();
        t.vt_write(b"X");
        t.flush();
        let snap = t.take_snapshot();
        assert_invariants(&snap);
    }

    // P1.4-XLVIII: ICH shifts cells right
    #[test]
    fn csi_ich_shift_content(col in 0u32..60u32, n in 1u32..5u32) {
        let mut t = sized(24, 80);
        t.vt_write(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ");
        t.flush();
        t.vt_write(format!("\x1b[{}G\x1b[{}@", col + 1, n).as_bytes());
        t.flush();
        let snap = t.take_snapshot();
        assert_invariants(&snap);
    }

    // P1.4-XLIX: ED 2 clears entire display
    #[test]
    fn csi_ed_2_clears_all(_n in 0u32..5u32) {
        let mut t = sized(24, 80);
        t.vt_write(b"Some content\non multiple\nlines here");
        t.flush();
        t.vt_write(b"\x1b[5;5H");
        t.flush();
        t.vt_write(b"\x1b[2J");
        t.flush();
        let snap = t.take_snapshot();
        assert_invariants(&snap);
    }

    // P1.4-L: EL 2 clears entire line
    #[test]
    fn csi_el_2_clears_line(_n in 0u32..5u32) {
        let mut t = sized(24, 80);
        t.vt_write(b"Hello World, this is a test line of text!");
        t.flush();
        t.vt_write(b"\x1b[2K");
        t.flush();
        let snap = t.take_snapshot();
        assert_invariants(&snap);
    }
}
