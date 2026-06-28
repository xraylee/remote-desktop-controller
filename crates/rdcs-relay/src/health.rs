// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Lightweight HTTP health-check and metrics endpoint for the relay node.
//!
//! Exposes three routes:
//!
//! | Route      | Status | Body                                  |
//! |------------|--------|---------------------------------------|
//! | `GET /health`  | 200    | `ok`                                  |
//! | `GET /metrics` | 200    | Prometheus text exposition format     |
//! | `GET /ready`   | 200/503| `ready` or `not ready`                |
//!
//! The server is built directly on top of `tokio::net::TcpListener` with
//! no external HTTP framework — the relay binary stays small and
//! dependency-free.

use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use crate::metrics::{format_prometheus, RelayMetrics};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Start the health HTTP server.
///
/// * `port` — TCP port to listen on (all interfaces `0.0.0.0`).
/// * `max_capacity` — maximum concurrent sessions the relay supports;
///   used by the `/ready` endpoint to signal back-pressure.
/// * `metrics` — shared [`RelayMetrics`] collector.
/// * `shutdown` — resolves when the boolean flips to `true`.
pub async fn run_health_server(
    addr: String,
    port: u16,
    max_capacity: u64,
    metrics: Arc<RelayMetrics>,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
) -> anyhow::Result<()> {
    let listener = TcpListener::bind(format!("{addr}:{port}")).await?;
    tracing::info!("health endpoint listening on {addr}:{port}");

    loop {
        tokio::select! {
            accept = listener.accept() => {
                let (mut stream, _peer) = accept?;
                let metrics = Arc::clone(&metrics);
                tokio::spawn(async move {
                    let mut buf = vec![0u8; 1024];
                    let n = match stream.read(&mut buf).await {
                        Ok(n) if n > 0 => n,
                        _ => return,
                    };
                    let request = String::from_utf8_lossy(&buf[..n]);
                    let first_line = request.lines().next().unwrap_or("");
                    let (status_line, body) = route(first_line, &metrics, max_capacity);
                    let response = format!(
                        "HTTP/1.1 {status_line}\r\nContent-Length: {}\r\nConnection: close\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n{body}",
                        body.len(),
                    );
                    let _ = stream.write_all(response.as_bytes()).await;
                });
            }
            _ = shutdown.changed() => {
                tracing::debug!("health server shutting down");
                break;
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Routing
// ---------------------------------------------------------------------------

/// Route a raw HTTP request line to the appropriate handler.
///
/// Returns `(status_line, body)`.
fn route(first_line: &str, metrics: &RelayMetrics, max_capacity: u64) -> (String, String) {
    let method_path = first_line.split_whitespace().take(2).collect::<Vec<_>>();
    if method_path.len() < 2 {
        return (
            "400 Bad Request".into(),
            "bad request".into(),
        );
    }
    let method = method_path[0];
    let path = method_path[1];

    if method != "GET" {
        return (
            "405 Method Not Allowed".into(),
            "method not allowed".into(),
        );
    }

    match path {
        "/health" => handle_health(),
        "/metrics" => handle_metrics(metrics),
        "/ready" => handle_ready(metrics, max_capacity),
        _ => (
            "404 Not Found".into(),
            "not found".into(),
        ),
    }
}

fn handle_health() -> (String, String) {
    ("200 OK".into(), "ok".into())
}

fn handle_metrics(metrics: &RelayMetrics) -> (String, String) {
    let snap = metrics.snapshot();
    let body = format_prometheus(&snap);
    ("200 OK".into(), body)
}

fn handle_ready(metrics: &RelayMetrics, max_capacity: u64) -> (String, String) {
    let active = metrics.active_sessions.load(std::sync::atomic::Ordering::Relaxed);
    if active < max_capacity {
        ("200 OK".into(), "ready".into())
    } else {
        ("503 Service Unavailable".into(), "not ready".into())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpStream;

    /// Send a raw HTTP GET to `addr` and return the full response string.
    async fn http_get(addr: &str, path: &str) -> String {
        let mut stream = TcpStream::connect(addr).await.unwrap();
        let request = format!("GET {path} HTTP/1.1\r\nHost: localhost\r\n\r\n");
        stream.write_all(request.as_bytes()).await.unwrap();

        let mut buf = vec![0u8; 8192];
        let n = stream.read(&mut buf).await.unwrap();
        String::from_utf8_lossy(&buf[..n]).to_string()
    }

    /// Extract the HTTP status code from a raw response.
    fn status_code(response: &str) -> u16 {
        let first_line = response.lines().next().unwrap_or("");
        first_line
            .split_whitespace()
            .nth(1)
            .unwrap_or("0")
            .parse()
            .unwrap_or(0)
    }

    /// Extract the body (everything after the blank line).
    fn body(response: &str) -> &str {
        response
            .split("\r\n\r\n")
            .nth(1)
            .unwrap_or("")
    }

    /// Start a health server on a random port and return `(addr, shutdown_tx)`.
    async fn start_test_server(
        max_capacity: u64,
    ) -> (String, tokio::sync::watch::Sender<bool>, Arc<RelayMetrics>) {
        let metrics = Arc::new(RelayMetrics::new());
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

        // Bind to port 0 to let the OS assign a free port.
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let addr_str = format!("127.0.0.1:{}", addr.port());

        // We need the listener already bound; replicate the server logic inline
        // so we can use port 0.
        let metrics_clone = Arc::clone(&metrics);
        let mut shutdown_rx_clone = shutdown_rx;
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    accept = listener.accept() => {
                        let (mut stream, _peer) = match accept {
                            Ok(v) => v,
                            Err(_) => break,
                        };
                        let m = Arc::clone(&metrics_clone);
                        tokio::spawn(async move {
                            let mut buf = vec![0u8; 1024];
                            let n = match stream.read(&mut buf).await {
                                Ok(n) if n > 0 => n,
                                _ => return,
                            };
                            let request = String::from_utf8_lossy(&buf[..n]);
                            let first_line = request.lines().next().unwrap_or("");
                            let (status_line, resp_body) = route(first_line, &m, max_capacity);
                            let response = format!(
                                "HTTP/1.1 {status_line}\r\nContent-Length: {}\r\nConnection: close\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n{resp_body}",
                                resp_body.len(),
                            );
                            let _ = stream.write_all(response.as_bytes()).await;
                        });
                    }
                    _ = shutdown_rx_clone.changed() => {
                        break;
                    }
                }
            }
        });

        (addr_str, shutdown_tx, metrics)
    }

    // -- Acceptance: health_endpoint ------------------------------------------

    #[tokio::test]
    async fn health_endpoint_returns_200_ok() {
        let (addr, _shutdown, _metrics) = start_test_server(100).await;
        let response = http_get(&addr, "/health").await;

        assert_eq!(status_code(&response), 200);
        assert_eq!(body(&response), "ok");
    }

    // -- Acceptance: metrics_endpoint -----------------------------------------

    #[tokio::test]
    async fn metrics_endpoint_returns_prometheus_format() {
        let (addr, _shutdown, metrics) = start_test_server(100).await;

        // Set some metric values.
        metrics.inc_session();
        metrics.inc_session();
        metrics.add_bytes(512);
        metrics.inc_slot_alloc();

        let response = http_get(&addr, "/metrics").await;

        assert_eq!(status_code(&response), 200);
        let resp_body = body(&response);
        assert!(
            resp_body.contains("rdcs_active_sessions 2"),
            "expected active_sessions=2 in: {resp_body}"
        );
        assert!(
            resp_body.contains("rdcs_total_bytes_forwarded 512"),
            "expected total_bytes_forwarded=512 in: {resp_body}"
        );
        assert!(
            resp_body.contains("rdcs_slots_allocated 1"),
            "expected slots_allocated=1 in: {resp_body}"
        );
        assert!(
            resp_body.contains("# TYPE rdcs_active_sessions gauge"),
            "expected TYPE header in: {resp_body}"
        );
    }

    // -- Acceptance: ready_endpoint -------------------------------------------

    #[tokio::test]
    async fn ready_endpoint_returns_200_when_under_capacity() {
        let (addr, _shutdown, _metrics) = start_test_server(10).await;

        let response = http_get(&addr, "/ready").await;

        assert_eq!(status_code(&response), 200);
        assert_eq!(body(&response), "ready");
    }

    #[tokio::test]
    async fn ready_endpoint_returns_503_when_at_capacity() {
        let (addr, _shutdown, metrics) = start_test_server(5).await;

        // Fill to capacity.
        for _ in 0..5 {
            metrics.inc_session();
        }

        let response = http_get(&addr, "/ready").await;

        assert_eq!(status_code(&response), 503);
        assert_eq!(body(&response), "not ready");
    }

    #[tokio::test]
    async fn ready_endpoint_returns_200_after_release() {
        let (addr, _shutdown, metrics) = start_test_server(2).await;

        // Fill to capacity.
        metrics.inc_session();
        metrics.inc_session();

        let response_full = http_get(&addr, "/ready").await;
        assert_eq!(status_code(&response_full), 503);

        // Release one session.
        metrics.dec_session();

        let response = http_get(&addr, "/ready").await;
        assert_eq!(status_code(&response), 200);
        assert_eq!(body(&response), "ready");
    }

    // -- Routing unit tests ---------------------------------------------------

    #[test]
    fn route_unknown_path_returns_404() {
        let m = RelayMetrics::new();
        let (status, resp_body) = route("GET /unknown HTTP/1.1", &m, 100);
        assert_eq!(status, "404 Not Found");
        assert_eq!(resp_body, "not found");
    }

    #[test]
    fn route_non_get_returns_405() {
        let m = RelayMetrics::new();
        let (status, resp_body) = route("POST /health HTTP/1.1", &m, 100);
        assert_eq!(status, "405 Method Not Allowed");
        assert_eq!(resp_body, "method not allowed");
    }

    #[test]
    fn route_malformed_request_returns_400() {
        let m = RelayMetrics::new();
        let (status, resp_body) = route("GARBAGE", &m, 100);
        assert_eq!(status, "400 Bad Request");
        assert_eq!(resp_body, "bad request");
    }

    #[test]
    fn route_health() {
        let m = RelayMetrics::new();
        let (status, resp_body) = route("GET /health HTTP/1.1", &m, 100);
        assert_eq!(status, "200 OK");
        assert_eq!(resp_body, "ok");
    }

    #[test]
    fn route_metrics() {
        let m = RelayMetrics::new();
        m.inc_session();
        m.add_bytes(42);
        let (status, resp_body) = route("GET /metrics HTTP/1.1", &m, 100);
        assert_eq!(status, "200 OK");
        assert!(resp_body.contains("rdcs_active_sessions 1"));
        assert!(resp_body.contains("rdcs_total_bytes_forwarded 42"));
    }

    #[test]
    fn route_ready_under_capacity() {
        let m = RelayMetrics::new();
        let (status, resp_body) = route("GET /ready HTTP/1.1", &m, 10);
        assert_eq!(status, "200 OK");
        assert_eq!(resp_body, "ready");
    }

    #[test]
    fn route_ready_at_capacity() {
        let m = RelayMetrics::new();
        for _ in 0..10 {
            m.inc_session();
        }
        let (status, resp_body) = route("GET /ready HTTP/1.1", &m, 10);
        assert_eq!(status, "503 Service Unavailable");
        assert_eq!(resp_body, "not ready");
    }

    // -- Shutdown test --------------------------------------------------------

    #[tokio::test]
    async fn health_server_shuts_down_cleanly() {
        let metrics = Arc::new(RelayMetrics::new());
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

        let handle = tokio::spawn(run_health_server(
            "127.0.0.1".into(),
            0, // OS-assigned port; we just test that it shuts down
            100,
            metrics,
            shutdown_rx,
        ));

        // Signal shutdown immediately.
        let _ = shutdown_tx.send(true);

        // Server should exit cleanly.
        let result = tokio::time::timeout(std::time::Duration::from_secs(2), handle).await;
        assert!(result.is_ok(), "health server should shut down within 2s");
    }
}
