use std::collections::HashSet;
use crate::manifest::{ExtensionCapability, ExtensionManifest};

pub struct SandboxSecurity {
    pub allowed_capabilities: HashSet<ExtensionCapability>,
}

impl SandboxSecurity {
    pub fn from_manifest(manifest: &ExtensionManifest) -> Self {
        let mut allowed = HashSet::new();
        for cap in &manifest.capabilities {
            allowed.insert(*cap);
        }
        Self {
            allowed_capabilities: allowed,
        }
    }

    pub fn check_permission(&self, cap: ExtensionCapability) -> Result<(), String> {
        if self.allowed_capabilities.contains(&cap) {
            Ok(())
        } else {
            Err(format!(
                "Permission denied: extension is not granted capability {:?}",
                cap
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_security() {
        let manifest = ExtensionManifest {
            name: "test-plugin".to_string(),
            display_name: None,
            version: "0.1.0".to_string(),
            publisher: "community".to_string(),
            description: "Test".to_string(),
            main: None,
            activation_events: vec!["*".to_string()],
            capabilities: vec![ExtensionCapability::FilesystemRead],
            contributes: Default::default(),
        };

        let sandbox = SandboxSecurity::from_manifest(&manifest);
        assert!(sandbox.check_permission(ExtensionCapability::FilesystemRead).is_ok());
        assert!(sandbox.check_permission(ExtensionCapability::TerminalExecution).is_err());
    }
}
