use super::super::CliError;
use super::server::McpServer;
use serde_json::{json, Value};
use tokio::io::{self, AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader};

async fn write_message<W: AsyncWrite + Unpin>(
    writer: &mut W,
    message: &Value,
) -> Result<(), CliError> {
    let serialized = serde_json::to_string(message)?;
    writer.write_all(serialized.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    Ok(())
}

pub(crate) fn not_initialized_error(id: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": -32602, "message": "Server not initialized. Call initialize first." }
    })
}

pub(crate) fn mcp_repo_from_params(params: Option<&Value>, default_repo: &str) -> String {
    params
        .and_then(|p| {
            p.get("path")
                .or_else(|| p.get("repo"))
                .and_then(|v| v.as_str())
        })
        .unwrap_or(default_repo)
        .to_string()
}

pub async fn mcp(root: &str) -> Result<(), CliError> {
    let stdin = io::stdin();
    let mut lines = BufReader::new(stdin).lines();

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout);

    let mut server = McpServer::new(root.to_string());

    while let Some(line) = lines.next_line().await? {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let message: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(err) => {
                eprintln!("mcp parse error: {err}");
                let resp = json!({
                    "jsonrpc": "2.0",
                    "id": 0,
                    "error": { "code": -32700, "message": "Parse error" }
                });
                write_message(&mut out, &resp).await?;
                continue;
            }
        };

        if let Some(response) = server.handle_message(message).await {
            write_message(&mut out, &response).await?;
        }
        for notification in server.drain_pending_notifications() {
            write_message(&mut out, &notification).await?;
        }
    }

    Ok(())
}
