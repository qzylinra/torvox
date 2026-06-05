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

    #[test]
    fn width_enum_zero_for_control() {
        assert_eq!(width('\u{0000}'), UnicodeWidth::Zero);
        assert_eq!(width('\u{001B}'), UnicodeWidth::Zero);
    }

    #[test]
    fn width_enum_single_for_ascii() {
        assert_eq!(width('0'), UnicodeWidth::Single);
        assert_eq!(width('~'), UnicodeWidth::Single);
        assert_eq!(width('!'), UnicodeWidth::Single);
    }

    #[test]
    fn width_japanese_kana() {
        assert_eq!(width_val('あ'), 2);
        assert_eq!(width_val('ア'), 2);
        // half-width katakana ｱ (U+FF71) is actually width 1
        assert_eq!(width_val('ｱ'), 1);
    }

    #[test]
    fn width_emoji() {
        // Simple emoji is typically wide
        assert!(is_wide('😀'));
    }

    #[test]
    fn str_width_empty() {
        assert_eq!(str_width(""), 0);
    }

    #[test]
    fn str_width_single_ascii() {
        assert_eq!(str_width("x"), 1);
    }

    #[test]
    fn str_width_cjk() {
        assert_eq!(str_width("中文"), 4);
    }

    #[test]
    fn str_width_combining() {
        // e + combining acute = 1 cell (e is wide=1, combining is 0)
        let s = "é";
        // 分解很重要：U+00E9 'é' 是宽度为 1 的单个字符
        assert_eq!(str_width(s), 1);
    }

    #[test]
    fn str_width_with_zero_width_chars() {
        // ZWJ is zero-width
        let s = "\u{200D}"; // ZWJ
        assert_eq!(str_width(s), 0);
    }

    #[test]
    fn str_width_long_ascii() {
        assert_eq!(str_width(&"a".repeat(100)), 100);
    }

    #[test]
    fn str_width_mixed_long() {
        // "AB中CD文EF" = 6*1 + 2*2 = 6 + 4 = 10
        assert_eq!(str_width("AB中CD文EF"), 10);
    }

    #[test]
    fn is_wide_cjk() {
        assert!(is_wide('日'));
        assert!(is_wide('本'));
        assert!(is_wide('語'));
    }

    #[test]
    fn is_wide_false_for_ascii() {
        for c in 'a'..='z' {
            assert!(!is_wide(c), "ASCII {c} should not be wide");
        }
    }

    #[test]
    fn is_wide_false_for_digit() {
        assert!(!is_wide('0'));
        assert!(!is_wide('9'));
    }

    #[test]
    fn width_val_zero_for_space() {
        // ' ' has width 1 actually
        assert_eq!(width_val(' '), 1);
    }

    #[test]
    fn width_val_zero_for_null() {
        assert_eq!(width_val('\0'), 0);
    }

    #[test]
    fn width_val_tab_is_control() {
        // tab is control = 0
        assert_eq!(width_val('\t'), 0);
    }

    #[test]
    fn width_val_newline_is_control() {
        assert_eq!(width_val('\n'), 0);
    }

    #[test]
    fn width_fullwidth_latin() {
        // Ａ (U+FF21) is fullwidth
        assert_eq!(width_val('Ａ'), 2);
    }

    #[test]
    fn str_width_fullwidth() {
        assert_eq!(str_width("Ａ"), 2);
    }

    #[test]
    fn str_width_emoji() {
        // 😀 (U+1F600) is typically width 2
        assert_eq!(str_width("😀"), 2);
    }
}
