use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cell {
    pub char: char,
    pub fg: Color,
    pub bg: Color,
    pub attrs: Attrs,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            char: ' ',
            fg: Color::default(),
            bg: Color::default(),
            attrs: Attrs::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Default for Color {
    fn default() -> Self {
        Self {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Attrs {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub reverse: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DirtyLine {
    Clean,
    Dirty(u32),
}

#[derive(Debug, Clone, Error)]
pub enum CoreError {
    #[error("row index out of bounds: {index} >= {max}")]
    RowOutOfBounds { index: u32, max: u32 },
    #[error("column index out of bounds: {index} >= {max}")]
    ColOutOfBounds { index: u32, max: u32 },
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub fn from_ansi(index: u8) -> Self {
        let [r, g, b] = crate::ansi::ansi_to_rgb(index);
        Self { r, g, b, a: 255 }
    }
}

impl Cell {
    pub fn with_char(c: char) -> Self {
        Self {
            char: c,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_default_is_space() {
        let c = Cell::default();
        assert_eq!(c.char, ' ');
        assert_eq!(c.fg, Color::default());
        assert_eq!(c.bg, Color::default());
    }

    #[test]
    fn cell_with_char() {
        let c = Cell::with_char('X');
        assert_eq!(c.char, 'X');
        assert_eq!(c.fg, Color::default());
    }

    #[test]
    fn color_new() {
        let c = Color::new(255, 128, 0);
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 128);
        assert_eq!(c.b, 0);
        assert_eq!(c.a, 255);
    }

    #[test]
    fn color_from_ansi() {
        let c = Color::from_ansi(1);
        assert_eq!(c.r, 128);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 0);
    }

    #[test]
    fn color_serde_roundtrip() {
        let c = Color::new(10, 20, 30);
        let bytes = postcard::to_allocvec(&c).unwrap();
        let decoded: Color = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(c, decoded);
    }

    #[test]
    fn cell_serde_roundtrip() {
        let c = Cell {
            char: 'A',
            fg: Color::new(255, 0, 0),
            bg: Color::new(0, 0, 255),
            attrs: Attrs {
                bold: true,
                italic: false,
                underline: true,
                reverse: false,
            },
        };
        let bytes = postcard::to_allocvec(&c).unwrap();
        let decoded: Cell = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(c, decoded);
    }

    #[test]
    fn attrs_default_all_false() {
        let a = Attrs::default();
        assert!(!a.bold);
        assert!(!a.italic);
        assert!(!a.underline);
        assert!(!a.reverse);
    }

    #[test]
    fn dirty_line_variants() {
        assert_eq!(DirtyLine::Clean, DirtyLine::Clean);
        assert_eq!(DirtyLine::Dirty(5), DirtyLine::Dirty(5));
    }
}
