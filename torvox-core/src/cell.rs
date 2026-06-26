// @Cell data model, IMPL_CORE_001, impl, [REQ_CORE_001]
// @need-ids: REQ_CORE_001, REQ_CORE_002
use serde::{Deserialize, Serialize};

#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cell {
    pub char: char,
    pub fg: Color,
    pub bg: Color,
    pub attrs: Attrs,
    pub width: u8,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            char: ' ',
            fg: Color::default(),
            bg: Color::default(),
            attrs: Attrs::default(),
            width: 1,
        }
    }
}

#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
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

#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Attrs {
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub double_underline: bool,
    pub reverse: bool,
    pub strikethrough: bool,
    pub blink: bool,
    pub hidden: bool,
    pub overline: bool,
    pub protected: bool,
    pub double_width: bool,
    pub double_height_top: bool,
    pub double_height_bottom: bool,
}

const BITS_PER_PARTITION: u32 = 64;

#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirtyMask {
    partitions: alloc::vec::Vec<u64>,
}

impl DirtyMask {
    pub fn new(total_rows: u32) -> Self {
        let num_partitions = (total_rows as usize).div_ceil(BITS_PER_PARTITION as usize);
        Self {
            partitions: alloc::vec![0u64; num_partitions.max(1)],
        }
    }

    pub fn is_dirty(&self, row: u32) -> bool {
        let (part, bit) = Self::partition_index(row);
        self.partitions
            .get(part)
            .is_some_and(|p| *p & (1 << bit) != 0)
    }

    pub fn mark(&mut self, row: u32) {
        let (part, bit) = Self::partition_index(row);
        if let Some(p) = self.partitions.get_mut(part) {
            *p |= 1 << bit;
        }
    }

    pub fn mark_all(&mut self, rows: u32) {
        let num_partitions = (rows as usize).div_ceil(BITS_PER_PARTITION as usize);
        self.partitions.clear();
        self.partitions.resize(num_partitions.max(1), !0u64);
        let remainder = rows % BITS_PER_PARTITION;
        if remainder != 0
            && let Some(last) = self.partitions.last_mut()
        {
            *last = (1u64 << remainder) - 1;
        }
    }

    pub fn clear(&mut self) {
        for p in &mut self.partitions {
            *p = 0;
        }
    }

    pub fn any_dirty(&self) -> bool {
        self.partitions.iter().any(|&p| p != 0)
    }

    pub fn resize(&mut self, total_rows: u32) {
        let num_partitions = (total_rows as usize).div_ceil(BITS_PER_PARTITION as usize);
        self.partitions.resize(num_partitions.max(1), 0);
    }

    fn partition_index(row: u32) -> (usize, u32) {
        let part = (row / BITS_PER_PARTITION) as usize;
        let bit = row % BITS_PER_PARTITION;
        (part, bit)
    }
}

impl Color {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self {
            r: red,
            g: green,
            b: blue,
            a: 255,
        }
    }

    pub fn from_ansi(index: u8) -> Self {
        let [r, g, b] = crate::ansi::ansi_to_rgb(index);
        Self { r, g, b, a: 255 }
    }

    pub fn saturating_add(&self, other: &Color) -> Color {
        Color {
            r: self.r.saturating_add(other.r),
            g: self.g.saturating_add(other.g),
            b: self.b.saturating_add(other.b),
            a: self.a.saturating_add(other.a),
        }
    }

    pub fn saturating_mul(&self, scalar: u8) -> Color {
        Color {
            r: self.r.saturating_mul(scalar),
            g: self.g.saturating_mul(scalar),
            b: self.b.saturating_mul(scalar),
            a: self.a.saturating_mul(scalar),
        }
    }
}

impl Cell {
    pub fn with_char(character: char) -> Self {
        Self {
            char: character,
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
    fn attrs_default_all_false() {
        let a = Attrs::default();
        assert!(!a.bold);
        assert!(!a.dim);
        assert!(!a.italic);
        assert!(!a.underline);
        assert!(!a.double_underline);
        assert!(!a.reverse);
        assert!(!a.strikethrough);
        assert!(!a.blink);
        assert!(!a.hidden);
        assert!(!a.overline);
    }

    #[test]
    fn dirty_mask_ops() {
        let mut m = DirtyMask::new(24);
        assert!(!m.any_dirty());
        m.mark(5);
        assert!(m.is_dirty(5));
        assert!(!m.is_dirty(0));
        m.clear();
        assert!(!m.any_dirty());
    }

    #[test]
    fn dirty_mask_mark_all() {
        let mut m = DirtyMask::new(80);
        m.mark_all(80);
        assert!(m.any_dirty());
        for i in 0..80 {
            assert!(m.is_dirty(i));
        }
        assert!(!m.is_dirty(80));
    }

    #[test]
    fn dirty_mask_large_row_count() {
        let mut m = DirtyMask::new(200);
        m.mark(0);
        m.mark(63);
        m.mark(64);
        m.mark(199);
        assert!(m.is_dirty(0));
        assert!(m.is_dirty(63));
        assert!(m.is_dirty(64));
        assert!(m.is_dirty(199));
        assert!(!m.is_dirty(65));
        assert!(!m.is_dirty(198));
    }

    #[test]
    fn dirty_mask_resize() {
        let mut m = DirtyMask::new(24);
        m.mark(5);
        m.resize(200);
        assert!(m.is_dirty(5));
        assert!(!m.is_dirty(100));
        m.mark(150);
        assert!(m.is_dirty(150));
    }

    #[test]
    fn dirty_mask_zero_rows_has_no_dirty() {
        let m = DirtyMask::new(0);
        assert!(!m.any_dirty());
    }

    #[test]
    fn dirty_mask_single_row() {
        let mut m = DirtyMask::new(1);
        m.mark(0);
        assert!(m.is_dirty(0));
        assert!(m.any_dirty());
        m.clear();
        assert!(!m.any_dirty());
    }

    #[test]
    fn dirty_mask_exactly_64_rows_one_partition() {
        let mut m = DirtyMask::new(64);
        for i in 0..64 {
            m.mark(i);
        }
        for i in 0..64 {
            assert!(m.is_dirty(i));
        }
    }

    #[test]
    fn dirty_mask_65_rows_two_partitions() {
        let mut m = DirtyMask::new(65);
        m.mark(0);
        m.mark(63);
        m.mark(64);
        assert!(m.is_dirty(0));
        assert!(m.is_dirty(63));
        assert!(m.is_dirty(64));
        assert!(!m.is_dirty(1));
        assert!(!m.is_dirty(62));
        assert!(!m.is_dirty(65));
    }

    #[test]
    fn dirty_mask_mark_all_exact_partition_boundary() {
        let mut m = DirtyMask::new(64);
        m.mark_all(64);
        for i in 0..64 {
            assert!(m.is_dirty(i));
        }
    }

    #[test]
    fn dirty_mask_mark_all_partial_last_partition() {
        let mut m = DirtyMask::new(100);
        m.mark_all(100);
        for i in 0..100 {
            assert!(m.is_dirty(i));
        }
        assert!(!m.is_dirty(100));
    }

    #[test]
    fn dirty_mask_mark_all_one_row() {
        let mut m = DirtyMask::new(1);
        m.mark_all(1);
        assert!(m.is_dirty(0));
    }

    #[test]
    fn dirty_mask_resize_smaller_keeps_early_marks() {
        let mut m = DirtyMask::new(200);
        m.mark(0);
        m.mark(50);
        m.resize(10);
        assert!(m.is_dirty(0));
        assert!(m.is_dirty(50));
        assert!(!m.is_dirty(9));
    }

    #[test]
    fn dirty_mask_resize_larger_starts_clean() {
        let mut m = DirtyMask::new(10);
        m.mark(5);
        m.resize(1000);
        assert!(m.is_dirty(5));
        assert!(!m.is_dirty(500));
    }

    #[test]
    fn dirty_mask_mark_idempotent() {
        let mut m = DirtyMask::new(100);
        m.mark(42);
        m.mark(42);
        m.mark(42);
        assert!(m.is_dirty(42));
    }

    #[test]
    fn dirty_mask_mark_all_then_clear() {
        let mut m = DirtyMask::new(200);
        m.mark_all(200);
        assert!(m.any_dirty());
        m.clear();
        assert!(!m.any_dirty());
        for i in 0..200 {
            assert!(!m.is_dirty(i));
        }
    }

    #[test]
    fn dirty_mask_partial_clear() {
        let mut m = DirtyMask::new(100);
        m.mark(10);
        m.mark(20);
        m.clear();
        assert!(!m.is_dirty(10));
        assert!(!m.is_dirty(20));
    }

    #[test]
    fn color_from_ansi_all_16() {
        for i in 0..16u8 {
            let c = Color::from_ansi(i);
            let expected = Color {
                r: crate::ansi::ansi_to_rgb(i)[0],
                g: crate::ansi::ansi_to_rgb(i)[1],
                b: crate::ansi::ansi_to_rgb(i)[2],
                a: 255,
            };
            assert_eq!(c, expected);
        }
    }

    #[test]
    fn color_from_ansi_all_216_cube() {
        for i in 16..232u8 {
            let c = Color::from_ansi(i);
            let expected = Color {
                r: crate::ansi::ansi_to_rgb(i)[0],
                g: crate::ansi::ansi_to_rgb(i)[1],
                b: crate::ansi::ansi_to_rgb(i)[2],
                a: 255,
            };
            assert_eq!(c, expected);
        }
    }

    #[test]
    fn color_from_ansi_all_24_grayscale() {
        for i in 232..=255u8 {
            let c = Color::from_ansi(i);
            let expected = Color {
                r: crate::ansi::ansi_to_rgb(i)[0],
                g: crate::ansi::ansi_to_rgb(i)[1],
                b: crate::ansi::ansi_to_rgb(i)[2],
                a: 255,
            };
            assert_eq!(c, expected);
        }
    }

    #[test]
    fn color_from_ansi_alpha_always_255() {
        for i in 0..=255u8 {
            let c = Color::from_ansi(i);
            assert_eq!(c.a, 255);
        }
    }

    #[test]
    fn color_default_is_white_opaque() {
        let c = Color::default();
        assert_eq!(c, Color::new(255, 255, 255));
    }

    #[test]
    fn color_equality() {
        assert_ne!(Color::new(1, 2, 3), Color::new(1, 2, 4));
        assert_ne!(Color::new(1, 2, 3), Color::new(2, 2, 3));
    }

    #[test]
    fn cell_equality_full() {
        let c1 = Cell {
            char: 'A',
            fg: Color::new(1, 2, 3),
            bg: Color::new(4, 5, 6),
            attrs: Attrs {
                bold: true,
                ..Default::default()
            },
            width: 1,
        };
        let c2 = c1;
        assert_eq!(c1, c2);
    }

    #[test]
    fn cell_inequality_different_char() {
        let c1 = Cell::with_char('A');
        let c2 = Cell::with_char('B');
        assert_ne!(c1, c2);
    }

    #[test]
    fn cell_inequality_different_width() {
        let c1 = Cell {
            char: 'A',
            width: 1,
            ..Default::default()
        };
        let c2 = Cell {
            char: 'A',
            width: 2,
            ..Default::default()
        };
        assert_ne!(c1, c2);
    }

    #[test]
    fn cell_inequality_different_attrs() {
        let mut c1 = Cell::with_char('A');
        c1.attrs.bold = true;
        let c2 = Cell::with_char('A');
        assert_ne!(c1, c2);
    }

    #[test]
    fn cell_with_char_keeps_default_colors() {
        let c = Cell::with_char('日');
        assert_eq!(c.char, '日');
        assert_eq!(c.fg, Color::default());
        assert_eq!(c.bg, Color::default());
        assert_eq!(c.width, 1);
    }

    #[test]
    fn attrs_serde_json_roundtrip() {
        let a = Attrs {
            bold: true,
            dim: false,
            italic: true,
            underline: false,
            double_underline: true,
            reverse: false,
            strikethrough: true,
            blink: false,
            hidden: true,
            overline: false,
            protected: true,
            double_width: false,
            double_height_top: false,
            double_height_bottom: false,
        };
        let json = serde_json::to_string(&a).expect("ser");
        let back: Attrs = serde_json::from_str(&json).expect("de");
        assert_eq!(a, back);
    }

    #[test]
    fn color_serde_json_roundtrip() {
        let c = Color::new(10, 20, 30);
        let json = serde_json::to_string(&c).expect("ser");
        let back: Color = serde_json::from_str(&json).expect("de");
        assert_eq!(c, back);
    }

    #[test]
    fn cell_serde_json_roundtrip() {
        let c = Cell {
            char: 'X',
            fg: Color::new(1, 2, 3),
            bg: Color::new(4, 5, 6),
            attrs: Attrs {
                bold: true,
                italic: true,
                ..Default::default()
            },
            width: 2,
        };
        let json = serde_json::to_string(&c).expect("ser");
        let back: Cell = serde_json::from_str(&json).expect("de");
        assert_eq!(c, back);
    }

    #[test]
    fn dirty_mask_serde_json_roundtrip() {
        let mut m = DirtyMask::new(100);
        m.mark(0);
        m.mark(50);
        m.mark(99);
        let json = serde_json::to_string(&m).expect("ser");
        let back: DirtyMask = serde_json::from_str(&json).expect("de");
        for i in 0..100 {
            assert_eq!(m.is_dirty(i), back.is_dirty(i));
        }
    }

    #[test]
    fn dirty_mask_clone_preserves_state() {
        let mut m = DirtyMask::new(50);
        m.mark(10);
        m.mark(20);
        let m2 = m.clone();
        assert!(m2.is_dirty(10));
        assert!(m2.is_dirty(20));
        assert!(!m2.is_dirty(11));
    }

    #[test]
    fn attrs_all_true() {
        let a = Attrs {
            bold: true,
            dim: true,
            italic: true,
            underline: true,
            double_underline: true,
            reverse: true,
            strikethrough: true,
            blink: true,
            hidden: true,
            overline: true,
            protected: true,
            double_width: true,
            double_height_top: true,
            double_height_bottom: true,
        };
        let json = serde_json::to_string(&a).unwrap();
        let back: Attrs = serde_json::from_str(&json).unwrap();
        assert_eq!(a, back);
    }

    #[test]
    fn dirty_mask_partition_boundary_63_64() {
        let mut m = DirtyMask::new(128);
        m.mark(63);
        m.mark(64);
        assert!(m.is_dirty(63));
        assert!(m.is_dirty(64));
        assert!(!m.is_dirty(62));
        assert!(!m.is_dirty(65));
    }

    #[test]
    fn dirty_mask_clear_then_mark() {
        let mut m = DirtyMask::new(10);
        m.mark(0);
        m.mark(5);
        m.clear();
        m.mark(7);
        assert!(!m.is_dirty(0));
        assert!(!m.is_dirty(5));
        assert!(m.is_dirty(7));
    }

    #[test]
    fn color_saturating_add_wraps_at_255() {
        let a = Color {
            r: 200,
            g: 200,
            b: 200,
            a: 255,
        };
        let b = Color {
            r: 100,
            g: 100,
            b: 100,
            a: 255,
        };
        let c = a.saturating_add(&b);
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 255);
        assert_eq!(c.b, 255);
        assert_eq!(c.a, 255);
    }

    #[test]
    fn color_saturating_add_small_values() {
        let a = Color {
            r: 10,
            g: 20,
            b: 30,
            a: 255,
        };
        let b = Color {
            r: 5,
            g: 10,
            b: 15,
            a: 0,
        };
        let c = a.saturating_add(&b);
        assert_eq!(c.r, 15);
        assert_eq!(c.g, 30);
        assert_eq!(c.b, 45);
        assert_eq!(c.a, 255);
    }

    #[test]
    fn color_saturating_add_zero() {
        let a = Color {
            r: 100,
            g: 50,
            b: 25,
            a: 128,
        };
        let zero = Color {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        };
        let c = a.saturating_add(&zero);
        assert_eq!(c, a);
    }

    #[test]
    fn color_saturating_mul_saturates() {
        let c = Color {
            r: 100,
            g: 50,
            b: 25,
            a: 255,
        };
        let d = c.saturating_mul(3);
        assert_eq!(d.r, 255);
        assert_eq!(d.g, 150);
        assert_eq!(d.b, 75);
        assert_eq!(d.a, 255);
    }

    #[test]
    fn color_saturating_mul_by_one() {
        let c = Color {
            r: 100,
            g: 50,
            b: 25,
            a: 200,
        };
        let d = c.saturating_mul(1);
        assert_eq!(d, c);
    }

    #[test]
    fn color_saturating_mul_by_zero() {
        let c = Color {
            r: 100,
            g: 50,
            b: 25,
            a: 200,
        };
        let d = c.saturating_mul(0);
        assert_eq!(d.r, 0);
        assert_eq!(d.g, 0);
        assert_eq!(d.b, 0);
        assert_eq!(d.a, 0);
    }

    #[test]
    fn color_saturating_mul_all_channels_saturate() {
        let c = Color {
            r: 200,
            g: 200,
            b: 200,
            a: 200,
        };
        let d = c.saturating_mul(2);
        assert_eq!(d.r, 255);
        assert_eq!(d.g, 255);
        assert_eq!(d.b, 255);
        assert_eq!(d.a, 255);
    }
}
