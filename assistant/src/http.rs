use crate::Assistant;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

pub async fn serve(assistant: Arc<Assistant>, port: u16) -> anyhow::Result<()> {
    let addr = format!("127.0.0.1:{port}");
    let listener = TcpListener::bind(&addr).await?;
    // Optional shared secret. When set (the app launches the daemon with
    // LAI_DAEMON_TOKEN), every request must carry `X-Lai-Token: <secret>` or
    // it gets 401. This closes the unauthenticated localhost listener: any
    // other app holding INTERNET could otherwise POST to it. When unset, the
    // daemon stays open (dev/host use).
    let token: Option<String> = std::env::var("LAI_DAEMON_TOKEN")
        .ok()
        .filter(|s| !s.is_empty());
    eprintln!(
        "[assistant] HTTP server listening on http://{addr}{}",
        if token.is_some() {
            " (token required)"
        } else {
            ""
        }
    );
    eprintln!("[assistant] POST /chat with JSON body: {{\"text\":\"hello\"}}");

    loop {
        let (stream, _peer) = listener.accept().await?;
        let assistant = assistant.clone();
        let token = token.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, assistant, token).await {
                eprintln!("[assistant] connection error: {e}");
            }
        });
    }
}

async fn handle_connection(
    mut stream: tokio::net::TcpStream,
    assistant: Arc<Assistant>,
    token: Option<String>,
) -> anyhow::Result<()> {
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);

    // Read HTTP request line
    let mut request_line = String::new();
    reader.read_line(&mut request_line).await?;
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    let method = parts.first().copied().unwrap_or("");
    let path = parts.get(1).copied().unwrap_or("/");

    // Read headers (to find Content-Length and the auth token)
    let mut content_length: usize = 0;
    let mut request_token: Option<String> = None;
    loop {
        let mut header = String::new();
        reader.read_line(&mut header).await?;
        if header.trim().is_empty() {
            break;
        }
        let lower = header.to_ascii_lowercase();
        if let Some(val) = lower.strip_prefix("content-length:") {
            content_length = val.trim().parse().unwrap_or(0);
        } else if let Some(val) = lower.strip_prefix("x-lai-token:") {
            request_token = Some(val.trim().to_string());
        }
    }

    // Auth gate: if a token is configured, the request must present it.
    // /health is also gated so a probe can't confirm the port is live without
    // the secret.
    if let Some(expected) = &token {
        match &request_token {
            Some(t) if t == expected => {}
            _ => {
                let body = serde_json::json!({"error": "unauthorized"}).to_string();
                let resp = format!(
                    "HTTP/1.1 401 Unauthorized\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body,
                );
                writer.write_all(resp.as_bytes()).await?;
                writer.flush().await?;
                return Ok(());
            }
        }
    }

    // Read body
    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        use tokio::io::AsyncReadExt;
        reader.read_exact(&mut body).await?;
    }
    let body_str = String::from_utf8_lossy(&body);

    // Route
    let response_body = match (method, path) {
        ("POST", "/chat") => match serde_json::from_str::<ChatRequest>(&body_str) {
            Ok(req) => {
                let resp = assistant.process_text_json(&req.text).await;
                match resp {
                    Ok(r) => serde_json::to_string(&r).unwrap_or_default(),
                    Err(e) => serde_json::json!({"error": e.to_string()}).to_string(),
                }
            }
            Err(e) => serde_json::json!({"error": format!("bad json: {e}")}).to_string(),
        },
        ("GET", "/health") => serde_json::json!({"status": "ok"}).to_string(),
        _ => serde_json::json!({"error": "not found"}).to_string(),
    };

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        response_body.len(),
        response_body,
    );
    writer.write_all(response.as_bytes()).await?;
    writer.flush().await?;
    Ok(())
}

#[derive(serde::Deserialize)]
struct ChatRequest {
    text: String,
}
