use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum CursorStyle {
    #[default]
    Block,
    Underline,
    Bar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct CursorState {
    pub row: u32,
    pub col: u32,
    pub style: CursorStyle,
    pub visible: bool,
}

impl Default for CursorState {
    fn default() -> Self {
        Self {
            row: 0,
            col: 0,
            style: CursorStyle::default(),
            visible: true,
        }
    }
}

impl CursorState {
    pub fn new(row: u32, col: u32) -> Self {
        Self {
            row,
            col,
            ..Default::default()
        }
    }

    pub fn move_to(&mut self, row: u32, col: u32) {
        self.row = row;
        self.col = col;
    }

    pub fn move_up(&mut self, n: u32) {
        self.row = self.row.saturating_sub(n);
    }

    pub fn move_down(&mut self, n: u32, max_rows: u32) {
        self.row = (self.row + n).min(max_rows.saturating_sub(1));
    }

    pub fn move_left(&mut self, n: u32) {
        self.col = self.col.saturating_sub(n);
    }

    pub fn move_right(&mut self, n: u32, max_cols: u32) {
        self.col = (self.col + n).min(max_cols.saturating_sub(1));
    }

    pub fn carriage_return(&mut self) {
        self.col = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_cursor() {
        let c = CursorState::default();
        assert_eq!(c.row, 0);
        assert_eq!(c.col, 0);
        assert_eq!(c.style, CursorStyle::Block);
        assert!(c.visible);
    }

    #[test]
    fn move_cursor() {
        let mut c = CursorState::new(5, 10);
        c.move_to(3, 7);
        assert_eq!(c.row, 3);
        assert_eq!(c.col, 7);
    }

    #[test]
    fn move_up_clamps_at_zero() {
        let mut c = CursorState::new(2, 0);
        c.move_up(5);
        assert_eq!(c.row, 0);
    }

    #[test]
    fn move_down_clamps_at_max() {
        let mut c = CursorState::new(22, 0);
        c.move_down(5, 24);
        assert_eq!(c.row, 23);
    }

    #[test]
    fn move_left_clamps_at_zero() {
        let mut c = CursorState::new(0, 3);
        c.move_left(10);
        assert_eq!(c.col, 0);
    }

    #[test]
    fn move_right_clamps_at_max() {
        let mut c = CursorState::new(0, 76);
        c.move_right(10, 80);
        assert_eq!(c.col, 79);
    }

    #[test]
    fn carriage_return() {
        let mut c = CursorState::new(5, 42);
        c.carriage_return();
        assert_eq!(c.col, 0);
        assert_eq!(c.row, 5);
    }

    #[test]
    fn cursor_style_default_is_block() {
        assert_eq!(CursorStyle::default(), CursorStyle::Block);
    }

    #[test]
    fn cursor_style_variants_distinct() {
        assert_ne!(CursorStyle::Block, CursorStyle::Underline);
        assert_ne!(CursorStyle::Block, CursorStyle::Bar);
        assert_ne!(CursorStyle::Underline, CursorStyle::Bar);
    }

    #[test]
    fn cursor_new_uses_defaults() {
        let c = CursorState::new(10, 20);
        assert_eq!(c.row, 10);
        assert_eq!(c.col, 20);
        assert_eq!(c.style, CursorStyle::Block);
        assert!(c.visible);
    }

    #[test]
    fn cursor_move_to_same_position() {
        let mut c = CursorState::new(5, 10);
        c.move_to(5, 10);
        assert_eq!(c.row, 5);
        assert_eq!(c.col, 10);
    }

    #[test]
    fn cursor_move_up_zero() {
        let mut c = CursorState::new(5, 0);
        c.move_up(0);
        assert_eq!(c.row, 5);
    }

    #[test]
    fn cursor_move_up_partial() {
        let mut c = CursorState::new(10, 0);
        c.move_up(3);
        assert_eq!(c.row, 7);
    }

    #[test]
    fn cursor_move_down_zero_max() {
        let mut c = CursorState::new(0, 0);
        c.move_down(5, 0);
        // (0 + 5).min(0.saturating_sub(1)) = 5.min(0) = 0
        assert_eq!(c.row, 0);
    }

    #[test]
    fn cursor_move_down_clamps_to_max_minus_one() {
        let mut c = CursorState::new(0, 0);
        c.move_down(100, 24);
        assert_eq!(c.row, 23);
    }

    #[test]
    fn cursor_move_left_zero() {
        let mut c = CursorState::new(0, 5);
        c.move_left(0);
        assert_eq!(c.col, 5);
    }

    #[test]
    fn cursor_move_left_partial() {
        let mut c = CursorState::new(0, 10);
        c.move_left(3);
        assert_eq!(c.col, 7);
    }

    #[test]
    fn cursor_move_right_zero_max() {
        let mut c = CursorState::new(0, 0);
        c.move_right(5, 0);
        assert_eq!(c.col, 0);
    }

    #[test]
    fn cursor_move_right_clamps_to_max_minus_one() {
        let mut c = CursorState::new(0, 0);
        c.move_right(100, 80);
        assert_eq!(c.col, 79);
    }

    #[test]
    fn cursor_equality() {
        let a = CursorState::new(1, 2);
        let b = CursorState::new(1, 2);
        assert_eq!(a, b);
        let c = CursorState::new(1, 3);
        assert_ne!(a, c);
    }

    #[test]
    fn cursor_visible_default_true() {
        let c = CursorState::new(0, 0);
        assert!(c.visible);
    }

    #[test]
    fn cursor_toggle_visible() {
        let mut c = CursorState::new(0, 0);
        c.visible = false;
        assert!(!c.visible);
        c.visible = true;
        assert!(c.visible);
    }

    #[test]
    fn cursor_style_serde() {
        for style in [CursorStyle::Block, CursorStyle::Underline, CursorStyle::Bar] {
            let json = serde_json::to_string(&style).unwrap();
            let back: CursorStyle = serde_json::from_str(&json).unwrap();
            assert_eq!(style, back);
        }
    }

    #[test]
    fn cursor_state_serde_roundtrip() {
        let c = CursorState {
            row: 5,
            col: 10,
            style: CursorStyle::Bar,
            visible: false,
        };
        let json = serde_json::to_string(&c).unwrap();
        let back: CursorState = serde_json::from_str(&json).unwrap();
        assert_eq!(c, back);
    }

    #[test]
    fn cursor_state_clone() {
        let c = CursorState::new(1, 2);
        let c2 = c;
        assert_eq!(c, c2);
    }
}
