use serde::{Deserialize, Serialize};
use unicode_width::UnicodeWidthChar;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnicodeWidth {
    Zero,
    Single,
    Double,
}

pub fn width(character: char) -> UnicodeWidth {
    let character_width = character.width().unwrap_or(0);
    match character_width {
        0 => UnicodeWidth::Zero,
        1 => UnicodeWidth::Single,
        _ => UnicodeWidth::Double,
    }
}

pub fn width_value(character: char) -> u8 {
    character.width().unwrap_or(0) as u8
}

pub fn string_width(text: &str) -> u32 {
    text.chars()
        .map(|character| width_value(character) as u32)
        .sum()
}

pub fn is_wide(character: char) -> bool {
    width_value(character) == 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_width() {
        assert_eq!(width_value('A'), 1);
        assert_eq!(width_value('z'), 1);
        assert_eq!(width_value(' '), 1);
    }

    #[test]
    fn control_width_zero() {
        assert_eq!(width_value('\u{001F}'), 0);
        assert_eq!(width_value('\u{007F}'), 0);
    }

    #[test]
    fn cjk_width() {
        assert_eq!(width_value('\u{4E00}'), 2);
        assert_eq!(width_value('中'), 2);
        assert_eq!(width_value('文'), 2);
    }

    #[test]
    fn hangul_width() {
        assert_eq!(width_value('\u{AC00}'), 2);
    }

    #[test]
    fn fullwidth_width() {
        assert_eq!(width_value('\u{FF01}'), 2);
    }

    #[test]
    fn combining_zero() {
        assert_eq!(width('\u{0300}'), UnicodeWidth::Zero);
    }

    #[test]
    fn string_width_ascii() {
        assert_eq!(string_width("hello"), 5);
    }

    #[test]
    fn string_width_mixed() {
        assert_eq!(string_width("A中B"), 4);
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
        assert_eq!(width_value('あ'), 2);
        assert_eq!(width_value('ア'), 2);
        assert_eq!(width_value('ｱ'), 1);
    }

    #[test]
    fn width_emoji() {
        assert!(is_wide('😀'));
    }

    #[test]
    fn string_width_empty() {
        assert_eq!(string_width(""), 0);
    }

    #[test]
    fn string_width_single_ascii() {
        assert_eq!(string_width("x"), 1);
    }

    #[test]
    fn string_width_cjk() {
        assert_eq!(string_width("中文"), 4);
    }

    #[test]
    fn string_width_combining() {
        let s = "é";
        assert_eq!(string_width(s), 1);
    }

    #[test]
    fn string_width_with_zero_width_chars() {
        let s = "\u{200D}";
        assert_eq!(string_width(s), 0);
    }

    #[test]
    fn string_width_long_ascii() {
        assert_eq!(string_width(&"a".repeat(100)), 100);
    }

    #[test]
    fn string_width_mixed_long() {
        assert_eq!(string_width("AB中CD文EF"), 10);
    }

    #[test]
    fn is_wide_cjk() {
        assert!(is_wide('日'));
        assert!(is_wide('本'));
        assert!(is_wide('語'));
    }

    #[test]
    fn is_wide_false_for_ascii() {
        for character in 'a'..='z' {
            assert!(!is_wide(character), "ASCII {character} should not be wide");
        }
    }

    #[test]
    fn is_wide_false_for_digit() {
        assert!(!is_wide('0'));
        assert!(!is_wide('9'));
    }

    #[test]
    fn width_value_space() {
        assert_eq!(width_value(' '), 1);
    }

    #[test]
    fn width_value_null() {
        assert_eq!(width_value('\0'), 0);
    }

    #[test]
    fn width_value_tab() {
        assert_eq!(width_value('\t'), 0);
    }

    #[test]
    fn width_value_newline() {
        assert_eq!(width_value('\n'), 0);
    }

    #[test]
    fn width_fullwidth_latin() {
        assert_eq!(width_value('Ａ'), 2);
    }

    #[test]
    fn string_width_fullwidth() {
        assert_eq!(string_width("Ａ"), 2);
    }

    #[test]
    fn string_width_emoji() {
        assert_eq!(string_width("😀"), 2);
    }

    #[test]
    fn width_emoji_skin_tone_wide() {
        assert!(is_wide('👍'));
        assert_eq!(width_value('👍'), 2);
    }

    #[test]
    fn width_emoji_flag() {
        assert_eq!(width_value('🇦'), 1);
        assert_eq!(width_value('🇧'), 1);
    }

    #[test]
    fn width_emoji_keycap() {
        assert_eq!(width_value('1'), 1);
    }

    #[test]
    fn width_soft_hyphen() {
        assert_eq!(width_value('\u{00AD}'), 0);
    }

    #[test]
    fn width_word_joiner() {
        assert_eq!(width_value('\u{2060}'), 0);
    }

    #[test]
    fn width_hangul_filler() {
        assert_eq!(width_value('\u{115F}'), 2);
    }

    #[test]
    fn width_variation_selector() {
        assert_eq!(width_value('\u{FE0F}'), 0);
    }

    #[test]
    fn width_combining_enclosing() {
        assert_eq!(width_value('\u{20DD}'), 0);
    }
}
