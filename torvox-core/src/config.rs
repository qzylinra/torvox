use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalConfig {
    pub rows: u32,
    pub cols: u32,
    pub scrollback_lines: u32,
    pub shell: Shell,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            rows: 24,
            cols: 80,
            scrollback_lines: 50_000,
            shell: Shell::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Shell {
    #[default]
    SystemDefault,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
            cursor_style: crate::cursor::CursorStyle::Block,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FontConfig {
    pub family: String,
    pub size: u16,
    pub line_spacing: i16,
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
            name: String::new(), // const fn can't use String::from
            bg: [30, 30, 46],
            fg: [205, 214, 244],
            cursor: [245, 224, 220],
            ansi: [
                [24, 24, 37],    // Black
                [243, 139, 168], // Red
                [166, 227, 161], // Green
                [249, 226, 175], // Yellow
                [137, 180, 250], // Blue
                [203, 166, 247], // Magenta
                [148, 226, 213], // Cyan
                [205, 214, 244], // White
                [108, 112, 134], // Bright Black
                [243, 139, 168], // Bright Red
                [166, 227, 161], // Bright Green
                [249, 226, 175], // Bright Yellow
                [137, 180, 250], // Bright Blue
                [203, 166, 247], // Bright Magenta
                [148, 226, 213], // Bright Cyan
                [187, 194, 222], // Bright White
            ],
        }
    }

    pub fn catppuccin_mocha_named() -> Self {
        let mut t = Self::catppuccin_mocha();
        t.name = String::from("Catppuccin Mocha");
        t
    }

    pub fn dracula() -> Self {
        Self {
            name: String::from("Dracula"),
            bg: [40, 42, 54],
            fg: [248, 248, 242],
            cursor: [248, 248, 242],
            ansi: [
                [0, 0, 0],       // Black
                [255, 85, 85],   // Red
                [80, 250, 123],  // Green
                [255, 184, 108], // Yellow
                [189, 147, 249], // Blue
                [255, 85, 215],  // Magenta
                [139, 233, 253], // Cyan
                [255, 255, 255], // White
                [68, 71, 90],    // Bright Black
                [255, 85, 85],   // Bright Red
                [80, 250, 123],  // Bright Green
                [255, 184, 108], // Bright Yellow
                [189, 147, 249], // Bright Blue
                [255, 85, 215],  // Bright Magenta
                [139, 233, 253], // Bright Cyan
                [255, 255, 255], // Bright White
            ],
        }
    }

    pub fn solarized_dark() -> Self {
        Self {
            name: String::from("Solarized Dark"),
            bg: [0, 43, 54],
            fg: [131, 148, 150],
            cursor: [131, 148, 150],
            ansi: [
                [7, 54, 66],     // Black
                [220, 50, 47],   // Red
                [133, 153, 0],   // Green
                [181, 137, 0],   // Yellow
                [38, 139, 210],  // Blue
                [211, 54, 130],  // Magenta
                [42, 161, 152],  // Cyan
                [238, 232, 213], // White
                [0, 43, 54],     // Bright Black
                [220, 50, 47],   // Bright Red
                [133, 153, 0],   // Bright Green
                [181, 137, 0],   // Bright Yellow
                [38, 139, 210],  // Bright Blue
                [211, 54, 130],  // Bright Magenta
                [42, 161, 152],  // Bright Cyan
                [238, 232, 213], // Bright White
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
                [59, 66, 82],    // Black
                [191, 97, 106],  // Red
                [163, 190, 140], // Green
                [235, 203, 139], // Yellow
                [129, 162, 190], // Blue
                [180, 142, 173], // Magenta
                [143, 188, 187], // Cyan
                [229, 233, 240], // White
                [76, 86, 106],   // Bright Black
                [191, 97, 106],  // Bright Red
                [163, 190, 140], // Bright Green
                [235, 203, 139], // Bright Yellow
                [129, 162, 190], // Bright Blue
                [180, 142, 173], // Bright Magenta
                [143, 188, 187], // Bright Cyan
                [236, 239, 244], // Bright White
            ],
        }
    }

    pub fn tokyo_night() -> Self {
        Self {
            name: String::from("Tokyo Night"),
            bg: [26, 27, 38],
            fg: [192, 202, 245],
            cursor: [192, 202, 245],
            ansi: [
                [24, 25, 38],    // Black
                [247, 118, 142], // Red
                [152, 195, 121], // Green
                [229, 192, 123], // Yellow
                [130, 170, 255], // Blue
                [199, 146, 234], // Magenta
                [86, 192, 196],  // Cyan
                [192, 202, 245], // White
                [69, 71, 90],    // Bright Black
                [247, 118, 142], // Bright Red
                [152, 195, 121], // Bright Green
                [229, 192, 123], // Bright Yellow
                [130, 170, 255], // Bright Blue
                [199, 146, 234], // Bright Magenta
                [86, 192, 196],  // Bright Cyan
                [205, 214, 244], // Bright White
            ],
        }
    }

    pub fn gruvbox_dark() -> Self {
        Self {
            name: String::from("Gruvbox Dark"),
            bg: [29, 32, 33],
            fg: [235, 219, 178],
            cursor: [235, 219, 178],
            ansi: [
                [29, 32, 33],    // Black
                [204, 51, 51],   // Red
                [152, 151, 26],  // Green
                [215, 153, 33],  // Yellow
                [69, 133, 136],  // Blue
                [177, 98, 134],  // Magenta
                [104, 157, 140], // Cyan
                [235, 219, 178], // White
                [80, 73, 69],    // Bright Black
                [251, 73, 52],   // Bright Red
                [184, 187, 38],  // Bright Green
                [250, 189, 47],  // Bright Yellow
                [131, 165, 152], // Bright Blue
                [211, 134, 155], // Bright Magenta
                [142, 192, 124], // Bright Cyan
                [229, 222, 199], // Bright White
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
                [31, 35, 43],    // Black
                [224, 108, 117], // Red
                [152, 195, 121], // Green
                [229, 192, 123], // Yellow
                [97, 175, 239],  // Blue
                [198, 120, 221], // Magenta
                [86, 182, 194],  // Cyan
                [171, 178, 191], // White
                [76, 82, 99],    // Bright Black
                [224, 108, 117], // Bright Red
                [152, 195, 121], // Bright Green
                [229, 192, 123], // Bright Yellow
                [97, 175, 239],  // Bright Blue
                [198, 120, 221], // Bright Magenta
                [86, 182, 194],  // Bright Cyan
                [208, 211, 220], // Bright White
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
                [27, 28, 22],    // Black
                [249, 38, 114],  // Red
                [166, 226, 46],  // Green
                [230, 219, 100], // Yellow
                [102, 217, 239], // Blue
                [174, 129, 255], // Magenta
                [166, 226, 46],  // Cyan
                [248, 248, 242], // White
                [102, 102, 102], // Bright Black
                [249, 38, 114],  // Bright Red
                [166, 226, 46],  // Bright Green
                [230, 219, 100], // Bright Yellow
                [102, 217, 239], // Bright Blue
                [174, 129, 255], // Bright Magenta
                [166, 226, 46],  // Bright Cyan
                [248, 248, 242], // Bright White
            ],
        }
    }

    pub fn github_dark() -> Self {
        Self {
            name: String::from("GitHub Dark"),
            bg: [22, 27, 34],
            fg: [201, 209, 217],
            cursor: [201, 209, 217],
            ansi: [
                [27, 31, 36],    // Black
                [255, 123, 114], // Red
                [63, 185, 80],   // Green
                [229, 192, 123], // Yellow
                [88, 166, 255],  // Blue
                [210, 153, 235], // Magenta
                [103, 224, 221], // Cyan
                [201, 209, 217], // White
                [110, 118, 129], // Bright Black
                [255, 123, 114], // Bright Red
                [63, 185, 80],   // Bright Green
                [229, 192, 123], // Bright Yellow
                [88, 166, 255],  // Bright Blue
                [210, 153, 235], // Bright Magenta
                [103, 224, 221], // Bright Cyan
                [230, 237, 243], // Bright White
            ],
        }
    }

    pub fn rose_pine() -> Self {
        Self {
            name: String::from("Rosé Pine"),
            bg: [25, 23, 36],
            fg: [224, 222, 244],
            cursor: [224, 222, 244],
            ansi: [
                [31, 29, 46],    // Black
                [235, 111, 146], // Red
                [156, 207, 216], // Green
                [246, 193, 119], // Yellow
                [127, 132, 245], // Blue
                [196, 167, 231], // Magenta
                [156, 207, 216], // Cyan
                [224, 222, 244], // White
                [110, 106, 134], // Bright Black
                [235, 111, 146], // Bright Red
                [156, 207, 216], // Bright Green
                [246, 193, 119], // Bright Yellow
                [127, 132, 245], // Bright Blue
                [196, 167, 231], // Bright Magenta
                [156, 207, 216], // Bright Cyan
                [233, 230, 253], // Bright White
            ],
        }
    }

    pub fn all_built_in() -> Vec<Self> {
        vec![
            Self::catppuccin_mocha_named(),
            Self::dracula(),
            Self::solarized_dark(),
            Self::nord(),
            Self::tokyo_night(),
            Self::gruvbox_dark(),
            Self::one_dark(),
            Self::monokai(),
            Self::github_dark(),
            Self::rose_pine(),
        ]
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
        assert_eq!(themes.len(), 10);
        for theme in &themes {
            assert!(!theme.name.is_empty(), "Theme missing name");
        }
    }
}
