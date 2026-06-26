// @Terminal event types, IMPL_CORE_004, impl, [REQ_CORE_004]
// @need-ids: REQ_CORE_004, REQ_CORE_005
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
    fn event_title_changed_carries_string() {
        let e = TerminalEvent::TitleChanged(String::from("My Terminal"));
        if let TerminalEvent::TitleChanged(title) = &e {
            assert_eq!(title.as_str(), "My Terminal");
        } else {
            panic!("Expected TitleChanged variant");
        }
    }

    #[test]
    fn event_title_changed_inequality() {
        let e1 = TerminalEvent::TitleChanged(String::from("a"));
        let e2 = TerminalEvent::TitleChanged(String::from("b"));
        assert_ne!(e1, e2);
    }

    #[test]
    fn event_clipboard_request_carries_operation() {
        let e = TerminalEvent::ClipboardRequest(String::from("paste"));
        if let TerminalEvent::ClipboardRequest(op) = &e {
            assert_eq!(op.as_str(), "paste");
        } else {
            panic!("Expected ClipboardRequest variant");
        }
    }

    #[test]
    fn event_hyperlink_hover_some_vs_none() {
        let some = TerminalEvent::HyperlinkHover(Some(String::from("https://x")));
        let none = TerminalEvent::HyperlinkHover(None);
        assert_ne!(some, none);
    }

    #[test]
    fn event_hyperlink_hover_none_is_distinguishable() {
        let e = TerminalEvent::HyperlinkHover(None);
        if let TerminalEvent::HyperlinkHover(url) = &e {
            assert!(url.is_none());
        } else {
            panic!("Expected HyperlinkHover variant");
        }
    }

    #[test]
    fn event_process_exit_code_distinguishes_success_from_failure() {
        let success = TerminalEvent::ProcessExited(0);
        let failure = TerminalEvent::ProcessExited(1);
        assert_ne!(success, failure, "exit code 0 and 1 should differ");
    }

    #[test]
    fn event_process_exited_carries_code() {
        let e = TerminalEvent::ProcessExited(42);
        if let TerminalEvent::ProcessExited(code) = e {
            assert_eq!(code, 42);
        } else {
            panic!("Expected ProcessExited variant");
        }
    }

    #[test]
    fn event_cursor_changed_carries_state() {
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
    fn event_selection_changed_none_distinguishes_from_some() {
        let none = TerminalEvent::SelectionChanged(None);
        let sel = Selection::new(
            SelectionAnchor { row: 1, col: 2 },
            SelectionAnchor { row: 3, col: 4 },
            SelectionMode::Char,
        );
        let some = TerminalEvent::SelectionChanged(Some(sel));
        assert_ne!(none, some, "None and Some selection should differ");
    }

    #[test]
    fn event_dirty_region_carries_bounds() {
        let region = DirtyRegion {
            start_row: 10,
            end_row: 20,
        };
        let e = TerminalEvent::DirtyRegion(region.clone());
        if let TerminalEvent::DirtyRegion(r) = &e {
            assert_eq!(r.start_row, 10);
            assert_eq!(r.end_row, 20);
        } else {
            panic!("Wrong variant");
        }
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
    fn dirty_region_bounds_are_correct() {
        let region = DirtyRegion {
            start_row: 3,
            end_row: 7,
        };
        assert_eq!(
            region.end_row - region.start_row,
            4,
            "region should span 4 rows"
        );
    }
}
