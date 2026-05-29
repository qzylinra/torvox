use alloc::string::String;
use serde::{Deserialize, Serialize};

use crate::cell::Color;
use crate::cursor::CursorState;
use crate::selection::Selection;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerminalEvent {
    OutputReady,
    Bell,
    TitleChanged(String),
    ClipboardRequest(String),
    HyperlinkHover(Option<String>),
    ProcessExited(i32),
    CursorChanged(CursorState),
    SelectionChanged(Option<Selection>),
    DirtyRegion(DirtyRegion),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirtyRegion {
    pub start_row: u32,
    pub end_row: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellUpdate {
    pub row: u32,
    pub col: u32,
    pub char: char,
    pub fg: Color,
    pub bg: Color,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_event_bell() {
        let e = TerminalEvent::Bell;
        let bytes = postcard::to_allocvec(&e).unwrap();
        let decoded: TerminalEvent = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(e, decoded);
    }

    #[test]
    fn terminal_event_title() {
        let e = TerminalEvent::TitleChanged(String::from("vim"));
        let bytes = postcard::to_allocvec(&e).unwrap();
        let decoded: TerminalEvent = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(e, decoded);
    }

    #[test]
    fn terminal_event_process_exit() {
        let e = TerminalEvent::ProcessExited(0);
        let bytes = postcard::to_allocvec(&e).unwrap();
        let decoded: TerminalEvent = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(e, decoded);
    }

    #[test]
    fn terminal_event_cursor_changed() {
        let e = TerminalEvent::CursorChanged(CursorState::new(5, 10));
        let bytes = postcard::to_allocvec(&e).unwrap();
        let decoded: TerminalEvent = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(e, decoded);
    }

    #[test]
    fn dirty_region_serde() {
        let dr = DirtyRegion {
            start_row: 0,
            end_row: 23,
        };
        let bytes = postcard::to_allocvec(&dr).unwrap();
        let decoded: DirtyRegion = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(dr, decoded);
    }

    #[test]
    fn cell_update_serde() {
        let cu = CellUpdate {
            row: 1,
            col: 2,
            char: 'X',
            fg: Color::default(),
            bg: Color::default(),
        };
        let bytes = postcard::to_allocvec(&cu).unwrap();
        let decoded: CellUpdate = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(cu, decoded);
    }
}
