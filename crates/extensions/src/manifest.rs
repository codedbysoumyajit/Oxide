use serde::{Deserialize, Serialize};
use commands::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExtensionCapability {
    FilesystemRead,
    FilesystemWrite,
    TerminalExecution,
    NetworkAccess,
    LspProvider,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExtensionContributions {
    pub commands: Vec<Command>,
    pub languages: Vec<String>,
    pub themes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionManifest {
    pub name: String,
    pub display_name: Option<String>,
    pub version: String,
    pub publisher: String,
    pub description: String,
    pub main: Option<String>,
    pub activation_events: Vec<String>,
    pub capabilities: Vec<ExtensionCapability>,
    pub contributes: ExtensionContributions,
}

impl ExtensionManifest {
    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }
}
