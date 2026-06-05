use alloc::string::String;
use serde::{Deserialize, Serialize};

use crate::cursor::CursorState;
use crate::selection::Selection;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
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
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct DirtyRegion {
    pub start_row: u32,
    pub end_row: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cursor::{CursorState, CursorStyle};
    use crate::selection::{Selection, SelectionAnchor, SelectionMode};

    #[test]
    fn event_output_ready_equality() {
        assert_eq!(TerminalEvent::OutputReady, TerminalEvent::OutputReady);
    }

    #[test]
    fn event_bell_equality() {
        assert_eq!(TerminalEvent::Bell, TerminalEvent::Bell);
    }

    #[test]
    fn event_title_changed_equality() {
        let e1 = TerminalEvent::TitleChanged(String::from("hello"));
        let e2 = TerminalEvent::TitleChanged(String::from("hello"));
        assert_eq!(e1, e2);
    }

    #[test]
    fn event_title_changed_inequality() {
        let e1 = TerminalEvent::TitleChanged(String::from("a"));
        let e2 = TerminalEvent::TitleChanged(String::from("b"));
        assert_ne!(e1, e2);
    }

    #[test]
    fn event_clipboard_request_equality() {
        let e1 = TerminalEvent::ClipboardRequest(String::from("text"));
        let e2 = TerminalEvent::ClipboardRequest(String::from("text"));
        assert_eq!(e1, e2);
    }

    #[test]
    fn event_hyperlink_hover_some() {
        let e1 = TerminalEvent::HyperlinkHover(Some(String::from("https://x")));
        let e2 = TerminalEvent::HyperlinkHover(Some(String::from("https://x")));
        assert_eq!(e1, e2);
    }

    #[test]
    fn event_hyperlink_hover_none() {
        assert_eq!(
            TerminalEvent::HyperlinkHover(None),
            TerminalEvent::HyperlinkHover(None)
        );
    }

    #[test]
    fn event_hyperlink_hover_some_vs_none() {
        let some = TerminalEvent::HyperlinkHover(Some(String::from("a")));
        let none = TerminalEvent::HyperlinkHover(None);
        assert_ne!(some, none);
    }

    #[test]
    fn event_process_exited_zero() {
        assert_eq!(
            TerminalEvent::ProcessExited(0),
            TerminalEvent::ProcessExited(0)
        );
    }

    #[test]
    fn event_process_exited_nonzero() {
        assert_eq!(
            TerminalEvent::ProcessExited(127),
            TerminalEvent::ProcessExited(127)
        );
    }

    #[test]
    fn event_process_exited_different_codes() {
        assert_ne!(
            TerminalEvent::ProcessExited(0),
            TerminalEvent::ProcessExited(1)
        );
    }

    #[test]
    fn event_cursor_changed() {
        let cursor = CursorState::new(5, 10);
        let e = TerminalEvent::CursorChanged(cursor);
        if let TerminalEvent::CursorChanged(c) = e {
            assert_eq!(c.row, 5);
            assert_eq!(c.col, 10);
            assert_eq!(c.style, CursorStyle::Block);
        } else {
            panic!("Wrong variant");
        }
    }

    #[test]
    fn event_selection_changed_some() {
        let sel = Selection::new(
            SelectionAnchor { row: 1, col: 2 },
            SelectionAnchor { row: 3, col: 4 },
            SelectionMode::Char,
        );
        let e = TerminalEvent::SelectionChanged(Some(sel));
        if let TerminalEvent::SelectionChanged(Some(s)) = e {
            assert_eq!(s.start.row, 1);
            assert_eq!(s.mode, SelectionMode::Char);
        } else {
            panic!("Wrong variant");
        }
    }

    #[test]
    fn event_selection_changed_none() {
        let e = TerminalEvent::SelectionChanged(None);
        if let TerminalEvent::SelectionChanged(None) = e {
            // pass
        } else {
            panic!("Wrong variant");
        }
    }

    #[test]
    fn event_dirty_region() {
        let region = DirtyRegion {
            start_row: 0,
            end_row: 24,
        };
        let e = TerminalEvent::DirtyRegion(region.clone());
        if let TerminalEvent::DirtyRegion(r) = &e {
            assert_eq!(r.start_row, 0);
            assert_eq!(r.end_row, 24);
        } else {
            panic!("Wrong variant");
        }
        assert_eq!(e, TerminalEvent::DirtyRegion(region));
    }

    #[test]
    fn event_distinct_variants_not_equal() {
        assert_ne!(TerminalEvent::OutputReady, TerminalEvent::Bell);
        assert_ne!(
            TerminalEvent::OutputReady,
            TerminalEvent::TitleChanged(String::from("a"))
        );
    }

    #[test]
    fn event_serde_json_roundtrip() {
        use alloc::vec;
        let events = vec![
            TerminalEvent::OutputReady,
            TerminalEvent::Bell,
            TerminalEvent::TitleChanged(String::from("hello world")),
            TerminalEvent::ClipboardRequest(String::from("x")),
            TerminalEvent::HyperlinkHover(Some(String::from("https://a"))),
            TerminalEvent::HyperlinkHover(None),
            TerminalEvent::ProcessExited(0),
            TerminalEvent::ProcessExited(42),
            TerminalEvent::CursorChanged(CursorState {
                row: 1,
                col: 2,
                style: CursorStyle::Bar,
                visible: false,
            }),
            TerminalEvent::SelectionChanged(None),
            TerminalEvent::DirtyRegion(DirtyRegion {
                start_row: 0,
                end_row: 24,
            }),
        ];
        for e in events {
            let json = serde_json::to_string(&e).expect("serialize");
            let back: TerminalEvent = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(e, back);
        }
    }

    #[test]
    fn dirty_region_equality() {
        let a = DirtyRegion {
            start_row: 1,
            end_row: 5,
        };
        let b = DirtyRegion {
            start_row: 1,
            end_row: 5,
        };
        assert_eq!(a, b);
    }

    #[test]
    fn dirty_region_clone() {
        let a = DirtyRegion {
            start_row: 3,
            end_row: 7,
        };
        let b = a.clone();
        assert_eq!(a, b);
    }
}
