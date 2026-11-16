use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    C,
    Cpp,
    Go,
    Html,
    Css,
    Json,
    Toml,
    Yaml,
    Bash,
    Sql,
    Markdown,
    Plaintext,
}

impl Language {
    pub fn id(&self) -> &'static str {
        match self {
            Language::Rust => "rust",
            Language::Python => "python",
            Language::JavaScript => "javascript",
            Language::TypeScript => "typescript",
            Language::C => "c",
            Language::Cpp => "cpp",
            Language::Go => "go",
            Language::Html => "html",
            Language::Css => "css",
            Language::Json => "json",
            Language::Toml => "toml",
            Language::Yaml => "yaml",
            Language::Bash => "bash",
            Language::Sql => "sql",
            Language::Markdown => "markdown",
            Language::Plaintext => "plaintext",
        }
    }

    pub fn from_path(path: &Path) -> Self {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "rs" => Language::Rust,
            "py" => Language::Python,
            "js" | "mjs" | "cjs" | "jsx" => Language::JavaScript,
            "ts" | "tsx" => Language::TypeScript,
            "c" | "h" => Language::C,
            "cpp" | "cc" | "cxx" | "hpp" => Language::Cpp,
            "go" => Language::Go,
            "html" | "htm" => Language::Html,
            "css" | "scss" | "less" => Language::Css,
            "json" => Language::Json,
            "toml" => Language::Toml,
            "yaml" | "yml" => Language::Yaml,
            "sh" | "bash" | "zsh" => Language::Bash,
            "sql" => Language::Sql,
            "md" | "markdown" => Language::Markdown,
            _ => Language::Plaintext,
        }
    }

    pub fn from_id(id: &str) -> Self {
        match id.to_lowercase().as_str() {
            "rust" | "rs" => Language::Rust,
            "python" | "py" => Language::Python,
            "javascript" | "js" => Language::JavaScript,
            "typescript" | "ts" => Language::TypeScript,
            "c" => Language::C,
            "cpp" | "c++" => Language::Cpp,
            "go" => Language::Go,
            "html" => Language::Html,
            "css" => Language::Css,
            "json" => Language::Json,
            "toml" => Language::Toml,
            "yaml" | "yml" => Language::Yaml,
            "bash" | "sh" => Language::Bash,
            "sql" => Language::Sql,
            "markdown" | "md" => Language::Markdown,
            _ => Language::Plaintext,
        }
    }

    pub fn keywords(&self) -> HashSet<&'static str> {
        let mut set = HashSet::new();
        match self {
            Language::Rust => {
                for kw in &[
                    "fn", "let", "mut", "pub", "struct", "enum", "trait", "impl", "use", "mod",
                    "as", "const", "static", "type", "where", "async", "await", "unsafe", "move",
                    "ref", "match", "if", "else", "loop", "while", "for", "in", "break",
                    "continue", "return", "self", "Self", "true", "false",
                ] {
                    set.insert(*kw);
                }
            }
            Language::Python => {
                for kw in &[
                    "def", "class", "import", "from", "as", "return", "if", "elif", "else",
                    "for", "while", "break", "continue", "try", "except", "finally", "with",
                    "yield", "async", "await", "lambda", "pass", "raise", "True", "False",
                    "None", "is", "in", "not", "and", "or",
                ] {
                    set.insert(*kw);
                }
            }
            Language::JavaScript | Language::TypeScript => {
                for kw in &[
                    "function", "const", "let", "var", "return", "if", "else", "for", "while",
                    "do", "switch", "case", "break", "continue", "default", "import", "export",
                    "from", "class", "extends", "super", "this", "new", "async", "await", "yield",
                    "try", "catch", "finally", "throw", "typeof", "instanceof", "true", "false",
                    "null", "undefined", "interface", "type", "enum",
                ] {
                    set.insert(*kw);
                }
            }
            Language::Go => {
                for kw in &[
                    "func", "package", "import", "return", "var", "const", "type", "struct",
                    "interface", "if", "else", "for", "range", "switch", "case", "select",
                    "chan", "go", "defer", "map", "true", "false", "nil",
                ] {
                    set.insert(*kw);
                }
            }
            Language::C | Language::Cpp => {
                for kw in &[
                    "int", "char", "float", "double", "void", "struct", "class", "enum", "union",
                    "typedef", "return", "if", "else", "for", "while", "do", "switch", "case",
                    "break", "continue", "default", "const", "static", "sizeof", "include",
                    "define", "namespace", "template", "public", "private", "protected", "true",
                    "false", "nullptr",
                ] {
                    set.insert(*kw);
                }
            }
            Language::Sql => {
                for kw in &[
                    "SELECT", "FROM", "WHERE", "INSERT", "INTO", "UPDATE", "DELETE", "JOIN",
                    "LEFT", "RIGHT", "INNER", "OUTER", "GROUP", "BY", "ORDER", "HAVING", "LIMIT",
                    "OFFSET", "CREATE", "TABLE", "DROP", "ALTER", "PRIMARY", "KEY", "FOREIGN",
                    "REFERENCES", "AS", "AND", "OR", "NOT", "NULL", "IS", "IN", "select", "from",
                    "where", "insert", "into", "update", "delete", "join",
                ] {
                    set.insert(*kw);
                }
            }
            _ => {}
        }
        set
    }
}
