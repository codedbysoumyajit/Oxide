use std::collections::HashMap;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use serde_json::Value;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::{mpsc, oneshot, Mutex};

#[derive(Error, Debug)]
pub enum LspError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Server error: {0}")]
    ServerError(String),
    #[error("Client shut down")]
    Shutdown,
}

type PendingRequests = Arc<Mutex<HashMap<u64, oneshot::Sender<Result<Value, LspError>>>>>;

pub struct LspClient {
    next_id: AtomicU64,
    stdin: Arc<Mutex<ChildStdin>>,
    pending_requests: PendingRequests,
}

impl LspClient {
    /// Spawns a language server process and starts the stdio JSON-RPC reader task.
    pub fn spawn(
        cmd: &str,
        args: &[&str],
        notification_tx: mpsc::Sender<(String, Value)>,
    ) -> Result<Self, LspError> {
        let mut child: Child = Command::new(cmd)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        let stdin = child.stdin.take().expect("Failed to open stdin");
        let stdout = child.stdout.take().expect("Failed to open stdout");

        let pending: PendingRequests = Arc::new(Mutex::new(HashMap::new()));
        let pending_clone = pending.clone();

        // Spawn async reader loop
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            let mut header_buf = String::new();

            loop {
                header_buf.clear();
                let mut content_length: Option<usize> = None;

                // Read HTTP-like headers
                loop {
                    let _bytes_read = match reader.read_line(&mut header_buf).await {
                        Ok(n) if n > 0 => n,
                        _ => return, // EOF or error
                    };

                    let line = header_buf.trim_end();
                    if line.is_empty() {
                        break; // End of headers
                    }

                    if let Some(len_str) = line.strip_prefix("Content-Length: ") {
                        if let Ok(len) = len_str.trim().parse::<usize>() {
                            content_length = Some(len);
                        }
                    }
                    header_buf.clear();
                }

                if let Some(length) = content_length {
                    let mut body_buf = vec![0u8; length];
                    if reader.read_exact(&mut body_buf).await.is_err() {
                        return;
                    }

                    if let Ok(val) = serde_json::from_slice::<Value>(&body_buf) {
                        // Check if it's a response to a request
                        if let Some(id) = val.get("id").and_then(|id| id.as_u64()) {
                            let mut map = pending_clone.lock().await;
                            if let Some(sender) = map.remove(&id) {
                                if let Some(err) = val.get("error") {
                                    let _ = sender.send(Err(LspError::ServerError(err.to_string())));
                                } else {
                                    let result = val.get("result").cloned().unwrap_or(Value::Null);
                                    let _ = sender.send(Ok(result));
                                }
                            }
                        } else if let Some(method) = val.get("method").and_then(|m| m.as_str()) {
                            // Server notification
                            let params = val.get("params").cloned().unwrap_or(Value::Null);
                            let _ = notification_tx.send((method.to_string(), params)).await;
                        }
                    }
                }
            }
        });

        Ok(Self {
            next_id: AtomicU64::new(1),
            stdin: Arc::new(Mutex::new(stdin)),
            pending_requests: pending,
        })
    }

    /// Sends a JSON-RPC request to the language server and awaits the response.
    pub async fn request(&self, method: &str, params: Value) -> Result<Value, LspError> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = oneshot::channel();

        {
            let mut pending = self.pending_requests.lock().await;
            pending.insert(id, tx);
        }

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });

        let body_bytes = serde_json::to_vec(&body)?;
        let header = format!("Content-Length: {}\r\n\r\n", body_bytes.len());

        let mut stdin = self.stdin.lock().await;
        stdin.write_all(header.as_bytes()).await?;
        stdin.write_all(&body_bytes).await?;
        stdin.flush().await?;

        rx.await.map_err(|_| LspError::Shutdown)?
    }

    /// Sends a JSON-RPC notification (fire-and-forget, no ID, no response).
    pub async fn notify(&self, method: &str, params: Value) -> Result<(), LspError> {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });

        let body_bytes = serde_json::to_vec(&body)?;
        let header = format!("Content-Length: {}\r\n\r\n", body_bytes.len());

        let mut stdin = self.stdin.lock().await;
        stdin.write_all(header.as_bytes()).await?;
        stdin.write_all(&body_bytes).await?;
        stdin.flush().await?;

        Ok(())
    }
}
