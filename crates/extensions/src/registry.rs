use std::collections::HashMap;
use crate::manifest::ExtensionManifest;

pub struct ExtensionRegistry {
    extensions: HashMap<String, ExtensionManifest>,
    active_extensions: HashMap<String, bool>,
}

impl Default for ExtensionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ExtensionRegistry {
    pub fn new() -> Self {
        Self {
            extensions: HashMap::new(),
            active_extensions: HashMap::new(),
        }
    }

    pub fn register(&mut self, manifest: ExtensionManifest) {
        let name = manifest.name.clone();
        self.extensions.insert(name.clone(), manifest);
        self.active_extensions.insert(name, false);
    }

    pub fn activate(&mut self, name: &str) -> bool {
        if self.extensions.contains_key(name) {
            self.active_extensions.insert(name.to_string(), true);
            true
        } else {
            false
        }
    }

    pub fn is_active(&self, name: &str) -> bool {
        self.active_extensions.get(name).copied().unwrap_or(false)
    }

    pub fn list_all(&self) -> Vec<&ExtensionManifest> {
        self.extensions.values().collect()
    }
}
