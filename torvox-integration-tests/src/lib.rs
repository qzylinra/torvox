//! Integration tests for Torvox terminal emulator.
//!
//! These tests verify cross-crate functionality that cannot be tested
//! in individual crate unit tests.

#[cfg(test)]
mod parse_and_render {
    #[test]
    fn parse_then_verify_grid() {
        use torvox_terminal::parser::VtParser;
        use torvox_terminal::terminal::TerminalState;

        let mut state = TerminalState::new(24, 80);
        let mut parser = VtParser::new();

        parser.advance(&mut state, b"Hello, World!\r\n");
        parser.advance(&mut state, b"\x1b[31mRed\x1b[0m\r\n");

        assert_eq!(state.grid.rows(), 24);
        assert_eq!(state.grid.cols(), 80);
        assert_eq!(state.grid.cell(0, 0).unwrap().char, 'H');
        assert_eq!(state.grid.cell(0, 4).unwrap().char, 'o');
        assert_eq!(state.grid.cell(0, 5).unwrap().char, ',');
        assert_eq!(state.grid.cell(1, 0).unwrap().char, 'R');
        assert!(state.grid.cell(1, 0).unwrap().fg.r > 0);
    }

    #[test]
    fn scrollback_preserved_on_scroll() {
        use torvox_terminal::parser::VtParser;
        use torvox_terminal::terminal::TerminalState;

        let mut state = TerminalState::new(3, 10);
        let mut parser = VtParser::new();

        for i in 0..10 {
            let line = format!("line{}\r\n", i);
            parser.advance(&mut state, line.as_bytes());
        }

        assert!(state.grid.scrollback_len() > 0);
    }

    #[test]
    fn sgr_color_persists_across_cells() {
        use torvox_terminal::parser::VtParser;
        use torvox_terminal::terminal::TerminalState;

        let mut state = TerminalState::new(1, 80);
        let mut parser = VtParser::new();

        parser.advance(&mut state, b"\x1b[31mABC");
        let c = state.grid.cell(0, 0).unwrap();
        assert!(c.fg.r > 0);
        assert_eq!(c.char, 'A');
        let c2 = state.grid.cell(0, 1).unwrap();
        assert!(c2.fg.r > 0);
        assert_eq!(c2.char, 'B');
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
            session.process_output();
            let grid = session.terminal();
            'outer: for row in 0..grid.rows() {
                if let Some(line) = grid.grid.get(row) {
                    for col in 0..line.len() {
                        if line.get(col).is_some_and(|c| c.char == 'o') {
                            found = true;
                            break 'outer;
                        }
                    }
                }
            }
            if found {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(found, "did not find expected output");
    }
}
