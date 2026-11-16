use crate::languages::Language;
use crate::token::{SyntaxToken, TokenKind};

pub struct Highlighter;

impl Highlighter {
    /// Tokenizes a single line of text for fast virtualized rendering.
    pub fn highlight_line(line_str: &str, language: Language) -> Vec<SyntaxToken> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = line_str.chars().collect();
        let len = chars.len();
        let mut i = 0;

        let keywords = language.keywords();

        while i < len {
            let ch = chars[i];

            // Whitespace: skip
            if ch.is_whitespace() {
                i += 1;
                continue;
            }

            // Line Comments
            let is_line_comment = match language {
                Language::Python | Language::Bash | Language::Yaml => ch == '#',
                Language::Sql => ch == '-' && i + 1 < len && chars[i + 1] == '-',
                _ => ch == '/' && i + 1 < len && chars[i + 1] == '/',
            };

            if is_line_comment {
                tokens.push(SyntaxToken::new(i..len, TokenKind::Comment));
                break;
            }

            // String literals
            if ch == '"' || ch == '\'' || ch == '`' {
                let quote = ch;
                let start = i;
                i += 1;
                let mut escaped = false;
                while i < len {
                    let cur = chars[i];
                    if escaped {
                        escaped = false;
                    } else if cur == '\\' {
                        escaped = true;
                    } else if cur == quote {
                        i += 1;
                        break;
                    }
                    i += 1;
                }
                tokens.push(SyntaxToken::new(start..i, TokenKind::String));
                continue;
            }

            // Numeric literals
            if ch.is_ascii_digit() {
                let start = i;
                let mut has_dot = false;
                while i < len {
                    let cur = chars[i];
                    if cur.is_ascii_digit() || cur == '_' || cur == 'x' || cur == 'b' || cur == 'o' {
                        i += 1;
                    } else if cur == '.' && !has_dot && i + 1 < len && chars[i + 1].is_ascii_digit() {
                        has_dot = true;
                        i += 1;
                    } else {
                        break;
                    }
                }
                tokens.push(SyntaxToken::new(start..i, TokenKind::Number));
                continue;
            }

            // Identifiers, keywords, function names, types
            if ch.is_alphabetic() || ch == '_' {
                let start = i;
                while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();

                let kind = if keywords.contains(word.as_str()) {
                    TokenKind::Keyword
                } else if i < len && chars[i] == '(' {
                    TokenKind::Function
                } else if word.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                    TokenKind::Type
                } else if word == word.to_uppercase() && word.len() > 1 {
                    TokenKind::Constant
                } else {
                    TokenKind::Variable
                };

                tokens.push(SyntaxToken::new(start..i, kind));
                continue;
            }

            // Operators & Delimiters
            let start = i;
            if matches!(ch, '+' | '-' | '*' | '/' | '%' | '=' | '!' | '<' | '>' | '&' | '|' | '^' | '~' | '?') {
                i += 1;
                tokens.push(SyntaxToken::new(start..i, TokenKind::Operator));
            } else if matches!(ch, '(' | ')' | '[' | ']' | '{' | '}' | ';' | ',' | '.' | ':') {
                i += 1;
                tokens.push(SyntaxToken::new(start..i, TokenKind::Punctuation));
            } else {
                i += 1;
                tokens.push(SyntaxToken::new(start..i, TokenKind::Text));
            }
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_rust_line() {
        let line = "fn main() { let x = 42; // comment";
        let tokens = Highlighter::highlight_line(line, Language::Rust);
        assert!(!tokens.is_empty());

        assert_eq!(tokens[0].kind, TokenKind::Keyword); // fn
        assert_eq!(tokens[1].kind, TokenKind::Function); // main
        assert_eq!(tokens[2].kind, TokenKind::Punctuation); // (
        assert_eq!(tokens[3].kind, TokenKind::Punctuation); // )
        assert_eq!(tokens[4].kind, TokenKind::Punctuation); // {
        assert_eq!(tokens[5].kind, TokenKind::Keyword); // let
        assert_eq!(tokens[6].kind, TokenKind::Variable); // x
        assert_eq!(tokens[7].kind, TokenKind::Operator); // =
        assert_eq!(tokens[8].kind, TokenKind::Number); // 42
        assert_eq!(tokens[9].kind, TokenKind::Punctuation); // ;
        assert_eq!(tokens[10].kind, TokenKind::Comment); // // comment
    }
}
