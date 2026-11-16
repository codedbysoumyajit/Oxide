use std::collections::HashMap;
use serde_json::Value;
use crate::config::EditorSettings;

pub struct VsCodeImporter;

impl VsCodeImporter {
    /// Imports VS Code `settings.json` format into native `EditorSettings`.
    pub fn import_settings_json(json_str: &str, current: &EditorSettings) -> EditorSettings {
        let mut settings = current.clone();
        let parsed: Result<HashMap<String, Value>, _> = serde_json::from_str(json_str);

        if let Ok(map) = parsed {
            for (key, val) in map {
                match key.as_str() {
                    "editor.fontSize" => {
                        if let Some(num) = val.as_f64() {
                            settings.font_size = num as f32;
                        }
                    }
                    "editor.fontFamily" => {
                        if let Some(s) = val.as_str() {
                            settings.font_family = s.to_string();
                        }
                    }
                    "editor.lineHeight" => {
                        if let Some(num) = val.as_f64() {
                            settings.line_height = num as f32;
                        }
                    }
                    "editor.tabSize" => {
                        if let Some(num) = val.as_u64() {
                            settings.tab_size = num as usize;
                        }
                    }
                    "editor.insertSpaces" => {
                        if let Some(b) = val.as_bool() {
                            settings.insert_spaces = b;
                        }
                    }
                    "editor.wordWrap" => {
                        if let Some(s) = val.as_str() {
                            settings.word_wrap = s == "on" || s == "wordWrapColumn" || s == "bounded";
                        } else if let Some(b) = val.as_bool() {
                            settings.word_wrap = b;
                        }
                    }
                    "editor.minimap.enabled" => {
                        if let Some(b) = val.as_bool() {
                            settings.minimap_enabled = b;
                        }
                    }
                    "editor.lineNumbers" => {
                        if let Some(s) = val.as_str() {
                            settings.line_numbers = s == "on";
                        } else if let Some(b) = val.as_bool() {
                            settings.line_numbers = b;
                        }
                    }
                    "workbench.colorTheme" => {
                        if let Some(s) = val.as_str() {
                            settings.theme = s.to_string();
                        }
                    }
                    _ => {}
                }
            }
        }

        settings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_vscode_settings() {
        let json = r#"{
            "editor.fontSize": 16,
            "editor.tabSize": 2,
            "editor.insertSpaces": true,
            "editor.wordWrap": "on",
            "workbench.colorTheme": "One Dark Pro"
        }"#;

        let defaults = EditorSettings::default();
        let imported = VsCodeImporter::import_settings_json(json, &defaults);

        assert_eq!(imported.font_size, 16.0);
        assert_eq!(imported.tab_size, 2);
        assert!(imported.word_wrap);
        assert_eq!(imported.theme, "One Dark Pro");
    }
}
