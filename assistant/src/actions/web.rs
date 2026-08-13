//! Outbound web tools — the agent's window onto the open internet.
//!
//! Gated behind the `web` feature. An offline-first assistant must not
//! advertise tools it can't honour, so these handlers only compile when the
//! daemon is explicitly allowed to make outbound requests.
//!
//! All three tools shell every request through `ureq` (already a workspace
//! dependency, no TLS surprises) with a hard global timeout. `ureq` is
//! blocking, so each call runs on `spawn_blocking` to preserve the async
//! contract of the surrounding agent loop. Failures fail loud: the handler
//! returns a human-readable error string rather than panicking or hanging.

use std::time::Duration;

/// Hard ceiling on any single outbound request. Keeps a wedged endpoint from
/// stalling the agent loop.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

/// Cap on the body text we hand back to the model, in bytes. Keeps a huge page
/// from blowing the LLM context window.
const MAX_BODY_BYTES: usize = 8_000;

fn agent() -> ureq::Agent {
    let config = ureq::config::Config::builder()
        .timeout_global(Some(REQUEST_TIMEOUT))
        .build();
    ureq::Agent::from(config)
}

fn truncate(mut body: String) -> String {
    if body.len() > MAX_BODY_BYTES {
        let mut end = MAX_BODY_BYTES;
        while !body.is_char_boundary(end) {
            end -= 1;
        }
        body.truncate(end);
        body.push_str("\n…[truncated]");
    }
    body
}

/// Web search via the DuckDuckGo Instant Answer API — a keyless, JSON endpoint.
/// Returns the abstract plus related-topic snippets, or a loud error string.
pub async fn search(query: &str) -> String {
    let query = query.trim().to_string();
    if query.is_empty() {
        return "web_search needs a non-empty query".to_string();
    }
    tokio::task::spawn_blocking(move || {
        let url = format!(
            "https://api.duckduckgo.com/?q={}&format=json&no_html=1&no_redirect=1",
            urlencoding::encode(&query)
        );
        let json: serde_json::Value = match agent().get(&url).call() {
            Ok(mut resp) => match resp.body_mut().read_json() {
                Ok(v) => v,
                Err(e) => return format!("web_search: could not parse response: {e}"),
            },
            Err(e) => return format!("web_search failed: {e}"),
        };

        let mut out = String::new();
        if let Some(abstract_text) = json.get("AbstractText").and_then(|v| v.as_str()) {
            if !abstract_text.is_empty() {
                out.push_str(abstract_text);
                out.push('\n');
            }
        }
        if let Some(topics) = json.get("RelatedTopics").and_then(|v| v.as_array()) {
            for topic in topics.iter().take(5) {
                if let Some(text) = topic.get("Text").and_then(|v| v.as_str()) {
                    if !text.is_empty() {
                        out.push_str("• ");
                        out.push_str(text);
                        out.push('\n');
                    }
                }
            }
        }
        let out = out.trim().to_string();
        if out.is_empty() {
            format!("web_search: no results for '{query}'")
        } else {
            truncate(out)
        }
    })
    .await
    .unwrap_or_else(|e| format!("web_search: task failed: {e}"))
}

/// Fetch a single URL and return its body text (truncated). Enforces http(s).
pub async fn fetch(url: &str) -> String {
    let url = url.trim().to_string();
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return "web_fetch needs an http:// or https:// URL".to_string();
    }
    tokio::task::spawn_blocking(move || match agent().get(&url).call() {
        Ok(mut resp) => match resp.body_mut().read_to_string() {
            Ok(body) => truncate(body),
            Err(e) => format!("web_fetch: could not read body: {e}"),
        },
        Err(e) => format!("web_fetch failed: {e}"),
    })
    .await
    .unwrap_or_else(|e| format!("web_fetch: task failed: {e}"))
}

/// Generic HTTP request. `method` is GET/POST/PUT/DELETE; `body` is optional.
pub async fn http_request(method: &str, url: &str, body: Option<String>) -> String {
    let method = method.trim().to_uppercase();
    let url = url.trim().to_string();
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return "http_request needs an http:// or https:// URL".to_string();
    }
    if !matches!(method.as_str(), "GET" | "POST" | "PUT" | "DELETE") {
        return format!("http_request: unsupported method '{method}'");
    }
    tokio::task::spawn_blocking(move || {
        let agent = agent();
        let result = match method.as_str() {
            "GET" => agent.get(&url).call(),
            "DELETE" => agent.delete(&url).call(),
            "POST" => match body {
                Some(b) => agent.post(&url).send(b.as_bytes()),
                None => agent.post(&url).send_empty(),
            },
            "PUT" => match body {
                Some(b) => agent.put(&url).send(b.as_bytes()),
                None => agent.put(&url).send_empty(),
            },
            _ => unreachable!("method validated above"),
        };
        match result {
            Ok(mut resp) => {
                let status = resp.status();
                match resp.body_mut().read_to_string() {
                    Ok(body) => truncate(format!("HTTP {status}\n{body}")),
                    Err(e) => format!("http_request: HTTP {status}, could not read body: {e}"),
                }
            }
            Err(e) => format!("http_request failed: {e}"),
        }
    })
    .await
    .unwrap_or_else(|e| format!("http_request: task failed: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn search_rejects_empty_query() {
        assert!(search("   ").await.contains("non-empty"));
    }

    #[tokio::test]
    async fn fetch_rejects_non_http_scheme() {
        assert!(fetch("ftp://example.com").await.contains("http://"));
    }

    #[tokio::test]
    async fn http_request_rejects_bad_scheme() {
        assert!(http_request("GET", "file:///etc/passwd", None)
            .await
            .contains("http://"));
    }

    #[tokio::test]
    async fn http_request_rejects_unsupported_method() {
        assert!(http_request("PATCH", "https://example.com", None)
            .await
            .contains("unsupported method"));
    }

    #[test]
    fn truncate_caps_oversized_body() {
        let big = "a".repeat(MAX_BODY_BYTES + 100);
        let out = truncate(big);
        assert!(out.ends_with("…[truncated]"));
        assert!(out.len() <= MAX_BODY_BYTES + 32);
    }
}
