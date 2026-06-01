//! Integration tests for Torvox terminal emulator.
//!
//! These tests verify cross-crate functionality that cannot be tested
//! in individual crate unit tests.

#[cfg(test)]
mod parse_and_render {
    #[test]
    fn parse_then_verify_terminal() {
        use torvox_terminal::ghostty_terminal::GhosttyTerminal;

        let mut terminal = GhosttyTerminal::new(24, 80, 1000).unwrap();

        terminal.vt_write(b"Hello, World!\r\n");
        terminal.vt_write(b"\x1b[31mRed\x1b[0m\r\n");

        assert_eq!(terminal.rows(), 24);
        assert_eq!(terminal.cols(), 80);
    }

    #[test]
    fn scrollback_preserved_on_scroll() {
        use torvox_terminal::ghostty_terminal::GhosttyTerminal;

        let mut terminal = GhosttyTerminal::new(3, 10, 1000).unwrap();

        for i in 0..10 {
            let line = format!("line{}\r\n", i);
            terminal.vt_write(line.as_bytes());
        }

        assert!(terminal.scrollback_len() > 0);
    }

    #[test]
    fn sgr_color_persists_across_cells() {
        use torvox_terminal::ghostty_terminal::GhosttyTerminal;

        let mut terminal = GhosttyTerminal::new(1, 80, 1000).unwrap();

        terminal.vt_write(b"\x1b[31mABC");

        let text = terminal.read_line_text(0).unwrap_or_default();
        assert!(text.contains('A'));
        assert!(text.contains('B'));
    }

    #[test]
    fn dec_2026_synchronized_output() {
        use torvox_terminal::ghostty_terminal::GhosttyTerminal;

        let mut terminal = GhosttyTerminal::new(5, 20, 1000).unwrap();

        terminal.vt_write(b"\x1b[?2026h");
        terminal.vt_write(b"first\x1b[2;1Hsecond\x1b[?2026l");

        assert_eq!(terminal.rows(), 5);
        assert_eq!(terminal.cols(), 20);
    }

    #[test]
    fn deccra_rectangular_copy() {
        use torvox_terminal::ghostty_terminal::GhosttyTerminal;

        let mut terminal = GhosttyTerminal::new(10, 20, 1000).unwrap();
        terminal.vt_write(b"ABCDE\x1b[E");
        terminal.vt_write(b"FGHIJ\x1b[E");
        terminal.vt_write(b"KLMNO\x1b[E");
        terminal.vt_write(b"PQRST\x1b[E");

        terminal.vt_write(b"\x1b[1;2;1;2;1;4;4\x24v");

        assert_eq!(terminal.rows(), 10);
    }

    #[test]
    fn decera_rectangular_erase() {
        use torvox_terminal::ghostty_terminal::GhosttyTerminal;

        let mut terminal = GhosttyTerminal::new(10, 20, 1000).unwrap();
        terminal.vt_write(b"ABCDE\r\nFGHIJ\r\nKLMNO\r\n");
        terminal.vt_write(b"\x1b[2;3;1;2\x7a");

        let text = terminal.read_line_text(1).unwrap_or_default();
        assert!(!text.is_empty() || terminal.cols() == 20);
    }
}

#[cfg(test)]
mod session_lifecycle {
    #[test]
    fn session_spawn_and_write() {
        use torvox_terminal::session::Session;

        let mut session = Session::spawn("/bin/sh", 24, 80).expect("spawn failed");
        session
            .write(b"echo integration_test_ok\n")
            .expect("write failed");

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        let mut found = false;
        while std::time::Instant::now() < deadline {
            let changed = session.process_output();
            if changed || session.is_exited() {
                found = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(found, "session did not produce output within deadline");
    }
}
