use egui::Color32;
use settings::{ColorRgb, Theme};
use syntax::TokenKind;

pub fn to_egui_color(c: ColorRgb) -> Color32 {
    Color32::from_rgba_premultiplied(c.r, c.g, c.b, c.a)
}

pub fn get_token_color(theme: &Theme, kind: TokenKind) -> Color32 {
    match kind {
        TokenKind::Keyword | TokenKind::ControlFlow => to_egui_color(theme.syntax_keyword),
        TokenKind::Function => to_egui_color(theme.syntax_function),
        TokenKind::Type => to_egui_color(theme.syntax_type),
        TokenKind::String => to_egui_color(theme.syntax_string),
        TokenKind::Number => to_egui_color(theme.syntax_number),
        TokenKind::Comment => to_egui_color(theme.syntax_comment),
        TokenKind::Operator => to_egui_color(theme.syntax_operator),
        TokenKind::Variable | TokenKind::Constant => to_egui_color(theme.syntax_variable),
        TokenKind::Punctuation | TokenKind::Text => to_egui_color(theme.foreground),
    }
}
