use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use serde_json::Value;
use tokio::sync::{mpsc, Mutex};

use crate::client::{LspClient, LspError};
use crate::diagnostics::DiagnosticStore;
use crate::protocol::Diagnostic;

pub struct LspManager {
    clients: HashMap<String, Arc<LspClient>>,
    pub diagnostics: Arc<Mutex<DiagnosticStore>>,
    notification_rx: mpsc::Receiver<(String, Value)>,
    notification_tx: mpsc::Sender<(String, Value)>,
}

impl Default for LspManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LspManager {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(100);
        Self {
            clients: HashMap::new(),
            diagnostics: Arc::new(Mutex::new(DiagnosticStore::new())),
            notification_rx: rx,
            notification_tx: tx,
        }
    }

    /// Returns the standard LSP server command for a given language.
    pub fn get_server_command(language_id: &str) -> Option<(&'static str, &'static [&'static str])> {
        match language_id {
            "rust" => Some(("rust-analyzer", &[])),
            "python" => Some(("pyright-langserver", &["--stdio"])),
            "javascript" | "typescript" => Some(("typescript-language-server", &["--stdio"])),
            "c" | "cpp" => Some(("clangd", &[])),
            "go" => Some(("gopls", &[])),
            _ => None,
        }
    }

    /// Lazily starts an LSP server for the given language if one is installed and not yet running.
    pub async fn ensure_server(&mut self, language_id: &str, root_path: &Path) -> Result<Arc<LspClient>, LspError> {
        if let Some(client) = self.clients.get(language_id) {
            return Ok(client.clone());
        }

        let (cmd, args) = Self::get_server_command(language_id)
            .ok_or_else(|| LspError::ServerError(format!("No LSP server registered for {}", language_id)))?;

        let client = Arc::new(LspClient::spawn(cmd, args, self.notification_tx.clone())?);

        // Initialize server
        let root_uri = format!("file://{}", root_path.to_string_lossy());
        let init_params = serde_json::json!({
            "processId": std::process::id(),
            "rootUri": root_uri,
            "capabilities": {
                "textDocument": {
                    "completion": { "completionItem": { "snippetSupport": true } },
                    "hover": { "contentFormat": ["markdown", "plaintext"] },
                    "definition": { "dynamicRegistration": true }
                }
            }
        });

        let _ = client.request("initialize", init_params).await?;
        client.notify("initialized", serde_json::json!({})).await?;

        self.clients.insert(language_id.to_string(), client.clone());
        Ok(client)
    }

    /// Process incoming notifications from language servers (like published diagnostics).
    pub async fn poll_notifications(&mut self) {
        while let Ok((method, params)) = self.notification_rx.try_recv() {
            if method == "textDocument/publishDiagnostics" {
                if let (Some(uri_str), Some(diag_array)) = (
                    params.get("uri").and_then(|u| u.as_str()),
                    params.get("diagnostics").and_then(|d| d.as_array()),
                ) {
                    let path_str = uri_str.trim_start_matches("file://");
                    let path = PathBuf::from(path_str);
                    let mut diags = Vec::new();
                    for d_val in diag_array {
                        if let Ok(diag) = serde_json::from_value::<Diagnostic>(d_val.clone()) {
                            diags.push(diag);
                        }
                    }
                    let mut store = self.diagnostics.lock().await;
                    store.set_diagnostics(path, diags);
                }
            }
        }
    }
}
