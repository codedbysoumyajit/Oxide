use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorSettings {
    pub font_size: f32,
    pub font_family: String,
    pub line_height: f32,
    pub tab_size: usize,
    pub insert_spaces: bool,
    pub word_wrap: bool,
    pub minimap_enabled: bool,
    pub sticky_scroll: bool,
    pub line_numbers: bool,
    pub auto_save_delay_ms: u64,
    pub theme: String,
    pub ui_scale: f32,
    pub memory_limit_mb: usize,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            font_family: "JetBrains Mono, Fira Code, monospace".to_string(),
            line_height: 1.4,
            tab_size: 4,
            insert_spaces: true,
            word_wrap: false,
            minimap_enabled: true,
            sticky_scroll: true,
            line_numbers: true,
            auto_save_delay_ms: 1000,
            theme: "Dark Modern".to_string(),
            ui_scale: 1.0,
            memory_limit_mb: 2048,
        }
    }
}
