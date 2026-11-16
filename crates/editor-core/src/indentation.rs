#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndentConfig {
    pub tab_size: usize,
    pub insert_spaces: bool,
}

impl Default for IndentConfig {
    fn default() -> Self {
        Self {
            tab_size: 4,
            insert_spaces: true,
        }
    }
}

pub struct Indentation;

impl Indentation {
    /// Get the leading whitespace of a line.
    pub fn get_line_indent(line_str: &str) -> String {
        let mut indent = String::new();
        for ch in line_str.chars() {
            if ch == ' ' || ch == '\t' {
                indent.push(ch);
            } else {
                break;
            }
        }
        indent
    }

    /// Compute the auto-indentation string when pressing enter on a given line.
    pub fn compute_enter_indent(line_str: &str, config: &IndentConfig) -> String {
        let current_indent = Self::get_line_indent(line_str);
        let trimmed = line_str.trim_end();

        let should_increase = trimmed.ends_with('{')
            || trimmed.ends_with('[')
            || trimmed.ends_with('(')
            || trimmed.ends_with(':')
            || trimmed.ends_with("->");

        if should_increase {
            let mut indent = current_indent;
            if config.insert_spaces {
                for _ in 0..config.tab_size {
                    indent.push(' ');
                }
            } else {
                indent.push('\t');
            }
            indent
        } else {
            current_indent
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_indent() {
        let config = IndentConfig::default();
        let prev_line = "    fn test() {";
        let next_indent = Indentation::compute_enter_indent(prev_line, &config);
        assert_eq!(next_indent, "        ");

        let normal_line = "        let x = 1;";
        let next_normal = Indentation::compute_enter_indent(normal_line, &config);
        assert_eq!(next_normal, "        ");
    }
}
