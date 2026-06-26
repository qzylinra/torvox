#[allow(clippy::cast_possible_truncation)]
pub const ANSI_256: [[u8; 3]; 256] = {
    let mut table = [[0u8; 3]; 256];
    let mut i = 0;
    while i < 16 {
        table[i] = ANSI_16[i];
        i += 1;
    }
    let mut v = 0;
    while v < 216 {
        let r = (v / 36) * 51;
        let g = ((v % 36) / 6) * 51;
        let b = (v % 6) * 51;
        table[16 + v] = [r as u8, g as u8, b as u8];
        v += 1;
    }
    let mut g = 0;
    while g < 24 {
        let v = 8 + g * 10;
        table[232 + g] = [v as u8, v as u8, v as u8];
        g += 1;
    }
    table
};

const ANSI_16: [[u8; 3]; 16] = [
    [0, 0, 0],
    [128, 0, 0],
    [0, 128, 0],
    [128, 128, 0],
    [0, 0, 128],
    [128, 0, 128],
    [0, 128, 128],
    [192, 192, 192],
    [128, 128, 128],
    [255, 0, 0],
    [0, 255, 0],
    [255, 255, 0],
    [0, 0, 255],
    [255, 0, 255],
    [0, 255, 255],
    [255, 255, 255],
];

pub fn ansi_to_rgb(index: u8) -> [u8; 3] {
    ANSI_256[index as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ansi_16_first_entry_is_black() {
        assert_eq!(ANSI_256[0], [0, 0, 0]);
    }

    #[test]
    fn ansi_16_last_entry_is_white() {
        assert_eq!(ANSI_256[15], [255, 255, 255]);
    }

    #[test]
    fn ansi_216_color_cube() {
        assert_eq!(ANSI_256[16], [0, 0, 0]);
        assert_eq!(ANSI_256[231], [255, 255, 255]);
    }

    #[test]
    fn ansi_grayscale_range() {
        assert_eq!(ANSI_256[232], [8, 8, 8]);
        assert_eq!(ANSI_256[255], [238, 238, 238]);
    }

    #[test]
    fn ansi_to_rgb_index() {
        assert_eq!(ansi_to_rgb(1), [128, 0, 0]);
        assert_eq!(ansi_to_rgb(9), [255, 0, 0]);
    }

    #[test]
    fn ansi_256_table_size() {
        assert_eq!(ANSI_256.len(), 256);
    }

    #[test]
    fn ansi_16_complete() {
        for i in 0..16 {
            assert_eq!(ANSI_256[i], ANSI_16[i]);
        }
    }

    #[test]
    fn ansi_216_cube_starts_black() {
        for v in 0..216 {
            let r = ((v / 36) * 51) as u8;
            let g = (((v % 36) / 6) * 51) as u8;
            let b = ((v % 6) * 51) as u8;
            assert_eq!(ANSI_256[16 + v], [r, g, b]);
        }
    }

    #[test]
    fn ansi_216_cube_first_index() {
        assert_eq!(ANSI_256[16], [0, 0, 0]);
    }

    #[test]
    fn ansi_216_cube_last_index() {
        assert_eq!(ANSI_256[231], [255, 255, 255]);
    }

    #[test]
    fn ansi_216_cube_known_color() {
        assert_eq!(ANSI_256[16 + 36 + 6 + 1], [51, 51, 51]);
        assert_eq!(ANSI_256[16 + 5 * 36 + 5 * 6 + 5], [255, 255, 255]);
    }

    #[test]
    fn ansi_grayscale_starts_at_8() {
        for i in 0..24 {
            let v = 8 + i * 10;
            assert_eq!(ANSI_256[232 + i], [v as u8, v as u8, v as u8]);
        }
    }

    #[test]
    fn ansi_grayscale_first_is_8() {
        assert_eq!(ANSI_256[232], [8, 8, 8]);
    }

    #[test]
    fn ansi_grayscale_last_is_238() {
        assert_eq!(ANSI_256[255], [238, 238, 238]);
    }

    #[test]
    fn ansi_to_rgb_passes_through_table() {
        for i in 0..=255u8 {
            assert_eq!(ansi_to_rgb(i), ANSI_256[i as usize]);
        }
    }

    #[test]
    fn ansi_pure_colors() {
        let cases = [
            (9, [255, 0, 0]),    // red
            (10, [0, 255, 0]),   // green
            (12, [0, 0, 255]),   // blue
            (11, [255, 255, 0]), // yellow
            (14, [0, 255, 255]), // cyan
            (13, [255, 0, 255]), // magenta
        ];
        for (idx, expected) in cases {
            assert_eq!(ansi_to_rgb(idx), expected, "ANSI index {}", idx);
        }
    }
}
