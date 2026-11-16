pub mod manifest;
pub mod registry;
pub mod sandbox;

pub use manifest::{ExtensionCapability, ExtensionContributions, ExtensionManifest};
pub use registry::ExtensionRegistry;
pub use sandbox::SandboxSecurity;
