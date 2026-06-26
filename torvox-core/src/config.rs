// @Terminal configuration, IMPL_CORE_003, impl, [REQ_CORE_003]
// @need-ids: REQ_CORE_003, REQ_CORE_004
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct TerminalConfig {
    pub rows: u32,
    pub cols: u32,
    pub scrollback_lines: u32,
    pub shell: Shell,
    pub font_size_tenths: u32,
    pub backspace_mode: BackspaceMode,
    pub right_alt_mode: RightAltMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum BackspaceMode {
    #[default]
    DEL,
    BS,
}

impl BackspaceMode {
    pub fn byte(&self) -> u8 {
        match self {
            BackspaceMode::DEL => 0x7f,
            BackspaceMode::BS => 0x08,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum RightAltMode {
    #[default]
    CharacterModifier,
    Meta,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            rows: 24,
            cols: 80,
            scrollback_lines: 50_000,
            shell: Shell::default(),
            font_size_tenths: 140,
            backspace_mode: BackspaceMode::default(),
            right_alt_mode: RightAltMode::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub enum Shell {
    #[default]
    SystemDefault,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct RenderConfig {
    pub font: FontConfig,
    pub theme: Theme,
    pub cursor_style: crate::cursor::CursorStyle,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            font: FontConfig::default(),
            theme: Theme::catppuccin_mocha(),
            cursor_style: crate::cursor::CursorStyle::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct FontConfig {
    pub family: String,
    pub size: u32,
    pub line_spacing: i32,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            family: String::from("JetBrains Mono Nerd Font"),
            size: 14,
            line_spacing: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "rkyv",
    derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)
)]
pub struct Theme {
    pub name: String,
    pub bg: [u8; 3],
    pub fg: [u8; 3],
    pub cursor: [u8; 3],
    pub ansi: [[u8; 3]; 16],
}

impl Theme {
    pub const fn catppuccin_mocha() -> Self {
        Self {
            name: String::new(),
            bg: [30, 30, 46],
            fg: [205, 214, 244],
            cursor: [245, 224, 220],
            ansi: [
                [24, 24, 37],
                [243, 139, 168],
                [166, 227, 161],
                [249, 226, 175],
                [137, 180, 250],
                [203, 166, 247],
                [148, 226, 213],
                [205, 214, 244],
                [108, 112, 134],
                [243, 139, 168],
                [166, 227, 161],
                [249, 226, 175],
                [137, 180, 250],
                [203, 166, 247],
                [148, 226, 213],
                [187, 194, 222],
            ],
        }
    }

    pub fn catppuccin_mocha_named() -> Self {
        let mut t = Self::catppuccin_mocha();
        t.name = String::from("Catppuccin Mocha");
        t
    }

    pub fn dracula_plus() -> Self {
        Self {
            name: String::from("Dracula Plus"),
            bg: [33, 33, 33],
            fg: [248, 248, 242],
            cursor: [236, 239, 244],
            ansi: [
                [33, 34, 44],
                [255, 85, 85],
                [80, 250, 123],
                [255, 203, 107],
                [130, 170, 255],
                [199, 146, 234],
                [139, 233, 253],
                [248, 249, 242],
                [84, 84, 84],
                [255, 110, 110],
                [105, 255, 148],
                [255, 203, 107],
                [214, 172, 255],
                [255, 146, 223],
                [164, 255, 255],
                [248, 248, 242],
            ],
        }
    }
    pub fn catppuccin_latte() -> Self {
        Self {
            name: String::from("Catppuccin Latte"),
            bg: [239, 241, 245],
            fg: [76, 79, 105],
            cursor: [220, 138, 120],
            ansi: [
                [92, 95, 119],
                [210, 15, 57],
                [64, 160, 43],
                [223, 142, 29],
                [30, 102, 245],
                [234, 118, 203],
                [23, 146, 153],
                [172, 176, 190],
                [108, 111, 133],
                [210, 15, 57],
                [64, 160, 43],
                [223, 142, 29],
                [30, 102, 245],
                [234, 118, 203],
                [23, 146, 153],
                [188, 192, 204],
            ],
        }
    }
    pub fn nord() -> Self {
        Self {
            name: String::from("Nord"),
            bg: [46, 52, 64],
            fg: [216, 222, 233],
            cursor: [216, 222, 233],
            ansi: [
                [59, 66, 82],
                [191, 97, 106],
                [163, 190, 140],
                [235, 203, 139],
                [129, 161, 193],
                [180, 142, 173],
                [136, 192, 208],
                [229, 233, 240],
                [76, 86, 106],
                [191, 97, 106],
                [163, 190, 140],
                [235, 203, 139],
                [129, 161, 193],
                [180, 142, 173],
                [143, 188, 187],
                [236, 239, 244],
            ],
        }
    }
    pub fn tokyo_night() -> Self {
        Self {
            name: String::from("Tokyo Night"),
            bg: [26, 27, 38],
            fg: [169, 177, 214],
            cursor: [169, 177, 214],
            ansi: [
                [50, 52, 74],
                [247, 118, 142],
                [158, 206, 106],
                [224, 175, 104],
                [122, 162, 247],
                [173, 142, 230],
                [68, 157, 171],
                [120, 124, 153],
                [68, 75, 106],
                [255, 122, 147],
                [185, 242, 124],
                [255, 158, 100],
                [125, 166, 255],
                [187, 154, 247],
                [13, 185, 215],
                [172, 176, 208],
            ],
        }
    }
    pub fn rose_pine() -> Self {
        Self {
            name: String::from("Rose Pine"),
            bg: [25, 23, 36],
            fg: [224, 222, 244],
            cursor: [82, 79, 103],
            ansi: [
                [38, 35, 58],
                [235, 111, 146],
                [49, 116, 143],
                [246, 193, 119],
                [156, 207, 216],
                [196, 167, 231],
                [235, 188, 186],
                [224, 222, 244],
                [110, 106, 134],
                [235, 111, 146],
                [49, 116, 143],
                [246, 193, 119],
                [156, 207, 216],
                [196, 167, 231],
                [235, 188, 186],
                [224, 222, 244],
            ],
        }
    }
    pub fn gruvbox_dark() -> Self {
        Self {
            name: String::from("Gruvbox Dark"),
            bg: [40, 40, 40],
            fg: [235, 219, 178],
            cursor: [235, 219, 178],
            ansi: [
                [40, 40, 40],
                [204, 36, 29],
                [152, 151, 26],
                [215, 153, 33],
                [69, 133, 136],
                [177, 98, 134],
                [104, 157, 106],
                [168, 153, 132],
                [146, 131, 116],
                [251, 73, 52],
                [184, 187, 38],
                [250, 189, 47],
                [131, 165, 152],
                [211, 134, 155],
                [142, 192, 124],
                [235, 219, 178],
            ],
        }
    }
    pub fn gruvbox_light() -> Self {
        Self {
            name: String::from("Gruvbox Light"),
            bg: [251, 241, 199],
            fg: [60, 56, 54],
            cursor: [60, 56, 54],
            ansi: [
                [251, 241, 199],
                [204, 36, 29],
                [152, 151, 26],
                [215, 153, 33],
                [69, 133, 136],
                [177, 98, 134],
                [104, 157, 106],
                [124, 111, 100],
                [146, 131, 116],
                [157, 0, 6],
                [121, 116, 14],
                [181, 118, 20],
                [7, 102, 120],
                [143, 63, 113],
                [66, 123, 88],
                [60, 56, 54],
            ],
        }
    }
    pub fn everforest_dark() -> Self {
        Self {
            name: String::from("Everforest Dark"),
            bg: [45, 53, 59],
            fg: [211, 198, 170],
            cursor: [211, 198, 170],
            ansi: [
                [71, 82, 88],
                [230, 126, 128],
                [167, 192, 128],
                [219, 188, 127],
                [127, 187, 179],
                [214, 153, 182],
                [131, 192, 146],
                [211, 198, 170],
                [71, 82, 88],
                [230, 126, 128],
                [167, 192, 128],
                [219, 188, 127],
                [127, 187, 179],
                [214, 153, 182],
                [131, 192, 146],
                [211, 198, 170],
            ],
        }
    }
    pub fn one_dark() -> Self {
        Self {
            name: String::from("One Dark"),
            bg: [40, 44, 52],
            fg: [171, 178, 191],
            cursor: [171, 178, 191],
            ansi: [
                [30, 33, 39],
                [224, 108, 117],
                [152, 195, 121],
                [209, 154, 102],
                [97, 175, 239],
                [198, 120, 221],
                [86, 182, 194],
                [171, 178, 191],
                [92, 99, 112],
                [224, 108, 117],
                [152, 195, 121],
                [209, 154, 102],
                [97, 175, 239],
                [198, 120, 221],
                [86, 182, 194],
                [255, 255, 255],
            ],
        }
    }
    pub fn one_light() -> Self {
        Self {
            name: String::from("One Light"),
            bg: [248, 248, 248],
            fg: [42, 43, 51],
            cursor: [42, 43, 51],
            ansi: [
                [0, 0, 0],
                [222, 61, 53],
                [62, 149, 58],
                [210, 182, 123],
                [47, 90, 243],
                [160, 0, 149],
                [62, 149, 58],
                [187, 187, 187],
                [0, 0, 0],
                [222, 61, 53],
                [62, 149, 58],
                [210, 182, 123],
                [47, 90, 243],
                [160, 0, 149],
                [62, 149, 58],
                [255, 255, 255],
            ],
        }
    }
    pub fn monokai() -> Self {
        Self {
            name: String::from("Monokai"),
            bg: [39, 40, 34],
            fg: [248, 248, 242],
            cursor: [248, 248, 242],
            ansi: [
                [39, 40, 34],
                [249, 38, 114],
                [166, 226, 46],
                [244, 191, 117],
                [102, 217, 239],
                [174, 129, 255],
                [161, 239, 228],
                [248, 248, 242],
                [117, 113, 94],
                [249, 38, 114],
                [166, 226, 46],
                [244, 191, 117],
                [102, 217, 239],
                [174, 129, 255],
                [161, 239, 228],
                [249, 248, 245],
            ],
        }
    }
    pub fn ayu_dark() -> Self {
        Self {
            name: String::from("Ayu Dark"),
            bg: [10, 14, 20],
            fg: [179, 177, 173],
            cursor: [179, 177, 173],
            ansi: [
                [1, 6, 14],
                [234, 108, 115],
                [145, 179, 98],
                [249, 175, 79],
                [83, 189, 250],
                [250, 233, 148],
                [144, 225, 198],
                [199, 199, 199],
                [104, 104, 104],
                [240, 113, 120],
                [194, 217, 76],
                [255, 180, 84],
                [89, 194, 255],
                [255, 238, 153],
                [149, 230, 203],
                [255, 255, 255],
            ],
        }
    }
    pub fn ayu_light() -> Self {
        Self {
            name: String::from("Ayu Light"),
            bg: [252, 252, 252],
            fg: [92, 97, 102],
            cursor: [92, 97, 102],
            ansi: [
                [1, 1, 1],
                [231, 102, 106],
                [128, 171, 36],
                [235, 165, 77],
                [65, 150, 223],
                [152, 112, 195],
                [81, 184, 145],
                [193, 193, 193],
                [52, 52, 52],
                [238, 146, 149],
                [159, 211, 47],
                [240, 188, 123],
                [109, 174, 230],
                [178, 148, 210],
                [117, 199, 168],
                [219, 219, 219],
            ],
        }
    }
    pub fn kanagawa_wave() -> Self {
        Self {
            name: String::from("Kanagawa Wave"),
            bg: [31, 31, 40],
            fg: [220, 215, 186],
            cursor: [220, 215, 186],
            ansi: [
                [9, 6, 24],
                [195, 64, 67],
                [118, 148, 106],
                [192, 163, 110],
                [126, 156, 216],
                [149, 127, 184],
                [106, 149, 137],
                [200, 192, 147],
                [114, 113, 105],
                [232, 36, 36],
                [152, 187, 108],
                [230, 195, 132],
                [127, 180, 202],
                [147, 138, 169],
                [122, 168, 159],
                [220, 215, 186],
            ],
        }
    }
    pub fn night_owl() -> Self {
        Self {
            name: String::from("Night Owl"),
            bg: [1, 22, 39],
            fg: [214, 222, 235],
            cursor: [214, 222, 235],
            ansi: [
                [1, 22, 39],
                [239, 83, 80],
                [34, 218, 110],
                [197, 228, 120],
                [130, 170, 255],
                [199, 146, 234],
                [33, 199, 168],
                [255, 255, 255],
                [87, 86, 86],
                [239, 83, 80],
                [34, 218, 110],
                [255, 235, 149],
                [130, 170, 255],
                [199, 146, 234],
                [127, 219, 202],
                [255, 255, 255],
            ],
        }
    }

    pub fn all_built_in() -> Vec<Self> {
        vec![
            Self::dracula_plus(),
            Self::catppuccin_mocha_named(),
            Self::catppuccin_latte(),
            Self::nord(),
            Self::tokyo_night(),
            Self::rose_pine(),
            Self::gruvbox_dark(),
            Self::gruvbox_light(),
            Self::everforest_dark(),
            Self::one_dark(),
            Self::one_light(),
            Self::monokai(),
            Self::ayu_dark(),
            Self::ayu_light(),
            Self::kanagawa_wave(),
            Self::night_owl(),
        ]
    }

    pub fn parse_custom(content: &str) -> Option<Self> {
        let mut name = String::new();
        let mut bg = [0u8; 3];
        let mut fg = [205u8, 214u8, 244u8];
        let mut cursor = [245u8, 224u8, 220u8];
        let mut ansi = [[0u8; 3]; 16];
        let mut found = false;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"').trim_matches('\'');
                match key {
                    "name" => {
                        name = String::from(value);
                        found = true;
                    }
                    "background" | "bg" => {
                        if let Some(c) = parse_color(value) {
                            bg = c;
                        }
                    }
                    "foreground" | "fg" => {
                        if let Some(c) = parse_color(value) {
                            fg = c;
                        }
                    }
                    "cursor" => {
                        if let Some(c) = parse_color(value) {
                            cursor = c;
                        }
                    }
                    "ansi0" | "black" => {
                        if let Some(c) = parse_color(value) {
                            ansi[0] = c;
                        }
                    }
                    "ansi1" | "red" => {
                        if let Some(c) = parse_color(value) {
                            ansi[1] = c;
                        }
                    }
                    "ansi2" | "green" => {
                        if let Some(c) = parse_color(value) {
                            ansi[2] = c;
                        }
                    }
                    "ansi3" | "yellow" => {
                        if let Some(c) = parse_color(value) {
                            ansi[3] = c;
                        }
                    }
                    "ansi4" | "blue" => {
                        if let Some(c) = parse_color(value) {
                            ansi[4] = c;
                        }
                    }
                    "ansi5" | "magenta" => {
                        if let Some(c) = parse_color(value) {
                            ansi[5] = c;
                        }
                    }
                    "ansi6" | "cyan" => {
                        if let Some(c) = parse_color(value) {
                            ansi[6] = c;
                        }
                    }
                    "ansi7" | "white" => {
                        if let Some(c) = parse_color(value) {
                            ansi[7] = c;
                        }
                    }
                    "ansi8" | "bright_black" => {
                        if let Some(c) = parse_color(value) {
                            ansi[8] = c;
                        }
                    }
                    "ansi9" | "bright_red" => {
                        if let Some(c) = parse_color(value) {
                            ansi[9] = c;
                        }
                    }
                    "ansi10" | "bright_green" => {
                        if let Some(c) = parse_color(value) {
                            ansi[10] = c;
                        }
                    }
                    "ansi11" | "bright_yellow" => {
                        if let Some(c) = parse_color(value) {
                            ansi[11] = c;
                        }
                    }
                    "ansi12" | "bright_blue" => {
                        if let Some(c) = parse_color(value) {
                            ansi[12] = c;
                        }
                    }
                    "ansi13" | "bright_magenta" => {
                        if let Some(c) = parse_color(value) {
                            ansi[13] = c;
                        }
                    }
                    "ansi14" | "bright_cyan" => {
                        if let Some(c) = parse_color(value) {
                            ansi[14] = c;
                        }
                    }
                    "ansi15" | "bright_white" => {
                        if let Some(c) = parse_color(value) {
                            ansi[15] = c;
                        }
                    }
                    _ => {}
                }
            }
        }

        if !found {
            return None;
        }

        Some(Self {
            name,
            bg,
            fg,
            cursor,
            ansi,
        })
    }
}

fn parse_color(color_string: &str) -> Option<[u8; 3]> {
    let trimmed = color_string.trim();
    if trimmed.starts_with('#') && trimmed.len() == 7 {
        let red = u8::from_str_radix(&trimmed[1..3], 16).ok()?;
        let green = u8::from_str_radix(&trimmed[3..5], 16).ok()?;
        let blue = u8::from_str_radix(&trimmed[5..7], 16).ok()?;
        Some([red, green, blue])
    } else if trimmed.starts_with('#') && trimmed.len() == 4 {
        let red = u8::from_str_radix(&trimmed[1..2], 16).ok()?;
        let green = u8::from_str_radix(&trimmed[2..3], 16).ok()?;
        let blue = u8::from_str_radix(&trimmed[3..4], 16).ok()?;
        Some([red * 17, green * 17, blue * 17])
    } else {
        let parts: Vec<&str> = trimmed.split(',').collect();
        if parts.len() == 3 {
            let red = parts[0].trim().parse().ok()?;
            let green = parts[1].trim().parse().ok()?;
            let blue = parts[2].trim().parse().ok()?;
            Some([red, green, blue])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_config_default() {
        let c = TerminalConfig::default();
        assert_eq!(c.rows, 24);
        assert_eq!(c.cols, 80);
        assert_eq!(c.scrollback_lines, 50_000);
        assert_eq!(c.font_size_tenths, 140);
    }

    #[test]
    fn render_config_default() {
        let c = RenderConfig::default();
        assert_eq!(c.font.family, "JetBrains Mono Nerd Font");
        assert_eq!(c.font.size, 14);
        assert_eq!(c.theme.name, "");
    }

    #[test]
    fn theme_has_16_ansi_colors() {
        let theme = Theme::catppuccin_mocha();
        assert_eq!(theme.ansi.len(), 16);
    }

    #[test]
    fn all_built_in_themes_have_names() {
        let themes = Theme::all_built_in();
        assert_eq!(themes.len(), 16);
        for theme in &themes {
            assert!(!theme.name.is_empty(), "Theme missing name");
        }
    }

    #[test]
    fn parse_custom_theme_hex() {
        let mut content = String::new();
        content.push_str("name = My Theme\n");
        content.push_str("bg = #1a1b26\n");
        content.push_str("fg = #c0caf5\n");
        content.push_str("cursor = #c0caf5\n");
        content.push_str("red = #f7768e\n");
        content.push_str("green = #98c379\n");
        content.push_str("blue = #82aaff\n");
        let theme = Theme::parse_custom(&content).unwrap();
        assert_eq!(theme.name, "My Theme");
        assert_eq!(theme.bg, [0x1a, 0x1b, 0x26]);
        assert_eq!(theme.fg, [0xc0, 0xca, 0xf5]);
        assert_eq!(theme.ansi[1], [0xf7, 0x76, 0x8e]);
        assert_eq!(theme.ansi[2], [0x98, 0xc3, 0x79]);
        assert_eq!(theme.ansi[4], [0x82, 0xaa, 0xff]);
    }

    #[test]
    fn parse_custom_theme_rgb() {
        let mut content = String::new();
        content.push_str("name = Test\n");
        content.push_str("bg = 30, 30, 46\n");
        content.push_str("fg = 205, 214, 244\n");
        let theme = Theme::parse_custom(&content).unwrap();
        assert_eq!(theme.bg, [30, 30, 46]);
        assert_eq!(theme.fg, [205, 214, 244]);
    }

    #[test]
    fn parse_custom_theme_short_hex() {
        let mut content = String::new();
        content.push_str("name = Short\n");
        content.push_str("bg = #abc\n");
        let theme = Theme::parse_custom(&content).unwrap();
        assert_eq!(theme.bg, [0xaa, 0xbb, 0xcc]);
    }

    #[test]
    fn parse_custom_theme_no_name_returns_none() {
        let content = "bg = #000000\nfg = #ffffff\n";
        assert!(Theme::parse_custom(content).is_none());
    }

    #[test]
    fn parse_custom_theme_comments_ignored() {
        let mut content = String::new();
        content.push_str("# This is a comment\n");
        content.push_str("name = Commented\n");
        content.push_str("# Another comment\n");
        content.push_str("bg = #112233\n");
        let theme = Theme::parse_custom(&content).unwrap();
        assert_eq!(theme.bg, [0x11, 0x22, 0x33]);
    }

    #[test]
    fn terminal_config_serde_roundtrip() {
        let c = TerminalConfig {
            rows: 50,
            cols: 120,
            scrollback_lines: 10000,
            shell: Shell::Custom(String::from("/bin/zsh")),
            font_size_tenths: 160,
            backspace_mode: BackspaceMode::default(),
            right_alt_mode: RightAltMode::default(),
        };
        let json = serde_json::to_string(&c).unwrap();
        let back: TerminalConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(c, back);
    }

    #[test]
    fn shell_default_is_system_default() {
        assert_eq!(Shell::default(), Shell::SystemDefault);
    }

    #[test]
    fn shell_custom_serde() {
        let s = Shell::Custom(String::from("/usr/local/bin/fish"));
        let json = serde_json::to_string(&s).unwrap();
        let back: Shell = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn shell_system_default_serde() {
        let s = Shell::SystemDefault;
        let json = serde_json::to_string(&s).unwrap();
        let back: Shell = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn shell_variants_not_equal() {
        assert_ne!(Shell::SystemDefault, Shell::Custom(String::from("/bin/sh")));
    }

    #[test]
    fn font_config_default_values() {
        let f = FontConfig::default();
        assert_eq!(f.size, 14);
        assert_eq!(f.line_spacing, 0);
        assert!(!f.family.is_empty());
    }

    #[test]
    fn render_config_default_theme_is_catppuccin() {
        let r = RenderConfig::default();
        assert_eq!(r.theme.bg, [30, 30, 46]);
        assert_eq!(r.theme.fg, [205, 214, 244]);
    }

    #[test]
    fn theme_dracula_distinct_from_mocha() {
        let a = Theme::catppuccin_mocha_named();
        let b = Theme::catppuccin_mocha();
        assert_ne!(a.name, b.name);
    }

    #[test]
    fn theme_all_built_in_unique_names() {
        let themes = Theme::all_built_in();
        let mut names: Vec<String> = themes.iter().map(|t| t.name.clone()).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), themes.len());
    }

    #[test]
    fn parse_custom_theme_with_ansi_keys() {
        let content = "name = Test\nansi0 = #000000\nansi1 = #ff0000\nansi15 = #ffffff\n";
        let theme = Theme::parse_custom(content).unwrap();
        assert_eq!(theme.ansi[0], [0, 0, 0]);
        assert_eq!(theme.ansi[1], [255, 0, 0]);
        assert_eq!(theme.ansi[15], [255, 255, 255]);
    }

    #[test]
    fn parse_custom_theme_with_color_names() {
        let content = "name = Test\nred = #ff0000\ngreen = #00ff00\nblue = #0000ff\nyellow = #ffff00\nmagenta = #ff00ff\ncyan = #00ffff\nblack = #000000\nwhite = #ffffff\n";
        let theme = Theme::parse_custom(content).unwrap();
        assert_eq!(theme.ansi[1], [255, 0, 0]);
        assert_eq!(theme.ansi[2], [0, 255, 0]);
        assert_eq!(theme.ansi[4], [0, 0, 255]);
        assert_eq!(theme.ansi[3], [255, 255, 0]);
        assert_eq!(theme.ansi[5], [255, 0, 255]);
        assert_eq!(theme.ansi[6], [0, 255, 255]);
    }

    #[test]
    fn parse_custom_theme_invalid_color_keeps_default() {
        let content = "name = Test\nbg = not_a_color\n";
        let theme = Theme::parse_custom(content).unwrap();
        assert_eq!(theme.bg, [0, 0, 0]);
    }

    #[test]
    fn parse_custom_theme_alternate_keys() {
        let content = "name = Test\nbackground = #111111\nforeground = #eeeeee\n";
        let theme = Theme::parse_custom(content).unwrap();
        assert_eq!(theme.bg, [0x11, 0x11, 0x11]);
        assert_eq!(theme.fg, [0xee, 0xee, 0xee]);
    }

    #[test]
    fn parse_custom_theme_cursor() {
        let content = "name = Test\ncursor = #abcdef\n";
        let theme = Theme::parse_custom(content).unwrap();
        assert_eq!(theme.cursor, [0xab, 0xcd, 0xef]);
    }

    #[test]
    fn parse_custom_theme_bright_keys() {
        let content = "name = Test\nbright_red = #aa0000\nbright_green = #00aa00\n";
        let theme = Theme::parse_custom(content).unwrap();
        assert_eq!(theme.ansi[9], [0xaa, 0, 0]);
        assert_eq!(theme.ansi[10], [0, 0xaa, 0]);
    }

    #[test]
    fn catppuccin_mocha_const_can_be_called() {
        let t = Theme::catppuccin_mocha();
        assert_ne!(t.bg, [0, 0, 0]);
        assert_ne!(t.fg, [0, 0, 0]);
    }

    #[test]
    fn catppuccin_mocha_named_has_name() {
        let t = Theme::catppuccin_mocha_named();
        assert_eq!(t.name, "Catppuccin Mocha");
    }

    #[test]
    fn all_themes_have_16_ansi_colors() {
        for theme in Theme::all_built_in() {
            assert_eq!(theme.ansi.len(), 16);
        }
    }

    #[test]
    fn parse_custom_theme_quoted_value() {
        let content = "name = \"My Theme\"\nbg = \"#ff0000\"\n";
        let theme = Theme::parse_custom(content).unwrap();
        assert_eq!(theme.name, "My Theme");
        assert_eq!(theme.bg, [255, 0, 0]);
    }

    #[test]
    fn parse_custom_theme_single_quoted_value() {
        let content = "name = 'My Theme'\nbg = '#ff0000'\n";
        let theme = Theme::parse_custom(content).unwrap();
        assert_eq!(theme.name, "My Theme");
        assert_eq!(theme.bg, [255, 0, 0]);
    }

    #[test]
    fn terminal_config_default_uses_system_shell() {
        let c = TerminalConfig::default();
        assert_eq!(c.shell, Shell::SystemDefault);
    }

    #[test]
    fn render_config_serde_roundtrip() {
        let r = RenderConfig {
            font: FontConfig {
                family: String::from("Mono"),
                size: 16,
                line_spacing: 2,
            },
            theme: Theme::catppuccin_mocha_named(),
            cursor_style: crate::cursor::CursorStyle::Bar,
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: RenderConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
    }

    #[test]
    fn font_config_serde_roundtrip() {
        let f = FontConfig {
            family: String::from("Test"),
            size: 18,
            line_spacing: -1,
        };
        let json = serde_json::to_string(&f).unwrap();
        let back: FontConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(f, back);
    }

    #[test]
    fn theme_serde_roundtrip() {
        let t = Theme::nord();
        let json = serde_json::to_string(&t).unwrap();
        let back: Theme = serde_json::from_str(&json).unwrap();
        assert_eq!(t, back);
    }

    #[test]
    fn parse_custom_theme_short_hex_with_alpha_in_3() {
        let content = "name = X\nbg = #fff\n";
        let t = Theme::parse_custom(content).unwrap();
        assert_eq!(t.bg, [255, 255, 255]);
    }

    #[test]
    fn parse_custom_theme_empty_string_returns_none() {
        assert!(Theme::parse_custom("").is_none());
    }

    #[test]
    fn parse_custom_theme_only_comments_returns_none() {
        assert!(Theme::parse_custom("# comment\n# another\n").is_none());
    }

    #[test]
    fn all_built_in_returns_sixteen_themes() {
        assert_eq!(Theme::all_built_in().len(), 16);
    }

    #[test]
    fn all_built_in_entries_have_unique_names() {
        let themes = Theme::all_built_in();
        let mut names: Vec<&str> = themes.iter().map(|t| t.name.as_str()).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), themes.len());
    }

    #[test]
    fn shell_custom_empty_string_stored_as_empty() {
        let s = Shell::Custom(String::new());
        let json = serde_json::to_string(&s).unwrap();
        let back: Shell = serde_json::from_str(&json).unwrap();
        assert_eq!(back, Shell::Custom(String::new()));
    }

    #[test]
    fn parse_color_rrggbb() {
        assert_eq!(parse_color("#ff8040"), Some([255, 128, 64]));
    }

    #[test]
    fn parse_color_rgb_shorthand() {
        assert_eq!(parse_color("#f80"), Some([255, 136, 0]));
    }

    #[test]
    fn parse_color_comma_separated() {
        assert_eq!(parse_color("255,128,64"), Some([255, 128, 64]));
    }

    #[test]
    fn parse_color_invalid_too_long() {
        assert_eq!(parse_color("#ff80400"), None);
    }

    #[test]
    fn parse_color_invalid_hex_chars() {
        assert_eq!(parse_color("#xyzxyz"), None);
    }

    #[test]
    fn parse_color_empty_returns_none() {
        assert_eq!(parse_color(""), None);
    }

    #[test]
    fn parse_color_no_hash_returns_none() {
        assert_eq!(parse_color("ff8040"), None);
    }

    #[test]
    fn parse_color_three_part_invalid_parse() {
        assert_eq!(parse_color("abc,def,ghi"), None);
    }

    #[test]
    fn parse_color_comma_with_whitespace() {
        assert_eq!(parse_color("255, 128, 64"), Some([255, 128, 64]));
    }
}
