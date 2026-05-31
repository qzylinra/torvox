use serde::{Deserialize, Serialize};
use unicode_width::UnicodeWidthChar;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnicodeWidth {
    Zero,
    Single,
    Double,
}

pub fn width(c: char) -> UnicodeWidth {
    let w = c.width().unwrap_or(0);
    match w {
        0 => UnicodeWidth::Zero,
        1 => UnicodeWidth::Single,
        _ => UnicodeWidth::Double,
    }
}

pub fn width_val(c: char) -> u8 {
    c.width().unwrap_or(0) as u8
}

pub fn str_width(s: &str) -> u32 {
    s.chars().map(|c| width_val(c) as u32).sum()
}

pub fn is_wide(c: char) -> bool {
    width_val(c) == 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_width() {
        assert_eq!(width_val('A'), 1);
        assert_eq!(width_val('z'), 1);
        assert_eq!(width_val(' '), 1);
    }

    #[test]
    fn control_width_zero() {
        assert_eq!(width_val('\u{001F}'), 0);
        assert_eq!(width_val('\u{007F}'), 0);
    }

    #[test]
    fn cjk_width() {
        assert_eq!(width_val('\u{4E00}'), 2);
        assert_eq!(width_val('中'), 2);
        assert_eq!(width_val('文'), 2);
    }

    #[test]
    fn hangul_width() {
        assert_eq!(width_val('\u{AC00}'), 2);
    }

    #[test]
    fn fullwidth_width() {
        assert_eq!(width_val('\u{FF01}'), 2);
    }

    #[test]
    fn combining_zero() {
        assert_eq!(width('\u{0300}'), UnicodeWidth::Zero);
    }

    #[test]
    fn str_width_ascii() {
        assert_eq!(str_width("hello"), 5);
    }

    #[test]
    fn str_width_mixed() {
        assert_eq!(str_width("A中B"), 4);
    }

    #[test]
    fn is_wide_check() {
        assert!(is_wide('中'));
        assert!(!is_wide('A'));
    }

    #[test]
    fn unicode_width_enum() {
        assert_eq!(width('A'), UnicodeWidth::Single);
        assert_eq!(width('中'), UnicodeWidth::Double);
    }
}
