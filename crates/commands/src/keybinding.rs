use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Modifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool, // Super/Command
}

impl Modifiers {
    pub const NONE: Self = Self {
        ctrl: false,
        shift: false,
        alt: false,
        meta: false,
    };
    pub const CTRL: Self = Self {
        ctrl: true,
        shift: false,
        alt: false,
        meta: false,
    };
    pub const CTRL_SHIFT: Self = Self {
        ctrl: true,
        shift: true,
        alt: false,
        meta: false,
    };
    pub const ALT: Self = Self {
        ctrl: false,
        shift: false,
        alt: true,
        meta: false,
    };
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyBinding {
    pub key: String,
    pub modifiers: Modifiers,
}

impl KeyBinding {
    pub fn new(key: &str, modifiers: Modifiers) -> Self {
        Self {
            key: key.to_string(),
            modifiers,
        }
    }

    /// Parse keybinding string such as "Ctrl+Shift+P" or "Ctrl+S"
    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<String> = s.split('+').map(|p| p.trim().to_lowercase()).collect();
        if parts.is_empty() {
            return None;
        }

        let mut ctrl = false;
        let mut shift = false;
        let mut alt = false;
        let mut meta = false;
        let mut key = String::new();

        for part in parts {
            match part.as_str() {
                "ctrl" | "control" => ctrl = true,
                "shift" => shift = true,
                "alt" | "opt" | "option" => alt = true,
                "cmd" | "command" | "meta" | "super" => meta = true,
                _ => key = part,
            }
        }

        if key.is_empty() {
            None
        } else {
            Some(Self {
                key,
                modifiers: Modifiers {
                    ctrl,
                    shift,
                    alt,
                    meta,
                },
            })
        }
    }

    pub fn to_display_string(&self) -> String {
        let mut s = String::new();
        if self.modifiers.ctrl {
            s.push_str("Ctrl+");
        }
        if self.modifiers.alt {
            s.push_str("Alt+");
        }
        if self.modifiers.shift {
            s.push_str("Shift+");
        }
        if self.modifiers.meta {
            s.push_str("Cmd+");
        }
        s.push_str(&self.key.to_uppercase());
        s
    }
}
