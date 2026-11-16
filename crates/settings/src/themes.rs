use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColorRgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl ColorRgb {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub is_dark: bool,
    pub background: ColorRgb,
    pub foreground: ColorRgb,
    pub gutter_bg: ColorRgb,
    pub gutter_fg: ColorRgb,
    pub active_line_bg: ColorRgb,
    pub selection_bg: ColorRgb,
    pub cursor_color: ColorRgb,
    pub tab_bg: ColorRgb,
    pub active_tab_bg: ColorRgb,
    pub sidebar_bg: ColorRgb,
    pub statusbar_bg: ColorRgb,
    pub statusbar_fg: ColorRgb,

    // Syntax colors
    pub syntax_keyword: ColorRgb,
    pub syntax_function: ColorRgb,
    pub syntax_type: ColorRgb,
    pub syntax_string: ColorRgb,
    pub syntax_number: ColorRgb,
    pub syntax_comment: ColorRgb,
    pub syntax_operator: ColorRgb,
    pub syntax_variable: ColorRgb,
}

impl Theme {
    /// Default modern dark theme (inspired by VS Code Dark Modern).
    pub fn dark_modern() -> Self {
        Self {
            name: "Dark Modern".to_string(),
            is_dark: true,
            background: ColorRgb::rgb(30, 30, 30),
            foreground: ColorRgb::rgb(212, 212, 212),
            gutter_bg: ColorRgb::rgb(30, 30, 30),
            gutter_fg: ColorRgb::rgb(133, 133, 133),
            active_line_bg: ColorRgb::rgba(255, 255, 255, 10),
            selection_bg: ColorRgb::rgba(38, 79, 120, 180),
            cursor_color: ColorRgb::rgb(82, 139, 255),
            tab_bg: ColorRgb::rgb(45, 45, 45),
            active_tab_bg: ColorRgb::rgb(30, 30, 30),
            sidebar_bg: ColorRgb::rgb(37, 37, 38),
            statusbar_bg: ColorRgb::rgb(0, 122, 204),
            statusbar_fg: ColorRgb::rgb(255, 255, 255),

            syntax_keyword: ColorRgb::rgb(86, 156, 214), // Blue
            syntax_function: ColorRgb::rgb(220, 220, 170), // Yellow
            syntax_type: ColorRgb::rgb(78, 201, 176), // Teal
            syntax_string: ColorRgb::rgb(206, 145, 120), // Orange/Brown
            syntax_number: ColorRgb::rgb(181, 206, 168), // Light green
            syntax_comment: ColorRgb::rgb(106, 153, 85), // Green
            syntax_operator: ColorRgb::rgb(212, 212, 212),
            syntax_variable: ColorRgb::rgb(156, 220, 254), // Light blue
        }
    }

    /// One Dark Pro theme.
    pub fn one_dark() -> Self {
        Self {
            name: "One Dark Pro".to_string(),
            is_dark: true,
            background: ColorRgb::rgb(40, 44, 52),
            foreground: ColorRgb::rgb(171, 178, 191),
            gutter_bg: ColorRgb::rgb(40, 44, 52),
            gutter_fg: ColorRgb::rgb(92, 99, 112),
            active_line_bg: ColorRgb::rgba(255, 255, 255, 8),
            selection_bg: ColorRgb::rgba(62, 68, 81, 220),
            cursor_color: ColorRgb::rgb(82, 139, 255),
            tab_bg: ColorRgb::rgb(33, 37, 43),
            active_tab_bg: ColorRgb::rgb(40, 44, 52),
            sidebar_bg: ColorRgb::rgb(33, 37, 43),
            statusbar_bg: ColorRgb::rgb(33, 37, 43),
            statusbar_fg: ColorRgb::rgb(157, 165, 180),

            syntax_keyword: ColorRgb::rgb(198, 120, 221), // Purple
            syntax_function: ColorRgb::rgb(97, 175, 239), // Blue
            syntax_type: ColorRgb::rgb(229, 192, 123), // Yellow
            syntax_string: ColorRgb::rgb(152, 195, 121), // Green
            syntax_number: ColorRgb::rgb(209, 154, 102), // Dark yellow
            syntax_comment: ColorRgb::rgb(92, 99, 112), // Grey
            syntax_operator: ColorRgb::rgb(86, 182, 194), // Cyan
            syntax_variable: ColorRgb::rgb(224, 108, 117), // Red
        }
    }

    /// Modern light theme.
    pub fn light_modern() -> Self {
        Self {
            name: "Light Modern".to_string(),
            is_dark: false,
            background: ColorRgb::rgb(255, 255, 255),
            foreground: ColorRgb::rgb(0, 0, 0),
            gutter_bg: ColorRgb::rgb(255, 255, 255),
            gutter_fg: ColorRgb::rgb(100, 100, 100),
            active_line_bg: ColorRgb::rgba(0, 0, 0, 8),
            selection_bg: ColorRgb::rgba(173, 214, 255, 180),
            cursor_color: ColorRgb::rgb(0, 90, 200),
            tab_bg: ColorRgb::rgb(243, 243, 243),
            active_tab_bg: ColorRgb::rgb(255, 255, 255),
            sidebar_bg: ColorRgb::rgb(248, 248, 248),
            statusbar_bg: ColorRgb::rgb(0, 122, 204),
            statusbar_fg: ColorRgb::rgb(255, 255, 255),

            syntax_keyword: ColorRgb::rgb(0, 0, 255),
            syntax_function: ColorRgb::rgb(121, 94, 38),
            syntax_type: ColorRgb::rgb(38, 127, 153),
            syntax_string: ColorRgb::rgb(163, 21, 21),
            syntax_number: ColorRgb::rgb(9, 134, 88),
            syntax_comment: ColorRgb::rgb(0, 128, 0),
            syntax_operator: ColorRgb::rgb(0, 0, 0),
            syntax_variable: ColorRgb::rgb(0, 16, 128),
        }
    }
}
