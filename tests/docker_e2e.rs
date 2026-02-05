//! Docker E2E tests for coda-mcp
//!
//! Run with: cargo test --test `docker_e2e` -- --ignored
//! Requires: Docker, `CODA_API_TOKEN` env var

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::Duration;

const IMAGE: &str = "coda-mcp:local";
const TIMEOUT_SECS: u64 = 10;

fn docker_available() -> bool {
    Command::new("docker")
        .arg("version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn image_exists() -> bool {
    Command::new("docker")
        .args(["image", "inspect", IMAGE])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn get_token() -> Option<String> {
    std::env::var("CODA_API_TOKEN").ok()
}

#[test]
#[ignore = "requires Docker and CODA_API_TOKEN"]
fn test_docker_mcp_initialize() {
    if !docker_available() {
        eprintln!("Skipping: Docker not available");
        return;
    }
    if !image_exists() {
        eprintln!("Skipping: Image {IMAGE} not found. Run: docker build -t {IMAGE} .");
        return;
    }
    let Some(token) = get_token() else {
        eprintln!("Skipping: CODA_API_TOKEN not set");
        return;
    };

    let mut child = Command::new("docker")
        .args([
            "run",
            "--rm",
            "-i",
            "-e",
            &format!("CODA_API_TOKEN={token}"),
            IMAGE,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start docker");

    let mut stdin = child.stdin.take().expect("Failed to get stdin");
    let stdout = child.stdout.take().expect("Failed to get stdout");

    // Send initialize request
    let request = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}"#;
    writeln!(stdin, "{request}").expect("Failed to write request");
    stdin.flush().expect("Failed to flush");

    // Read response with timeout
    let reader = BufReader::new(stdout);
    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        if let Some(line) = reader.lines().map_while(Result::ok).next() {
            let _ = tx.send(line);
        }
    });

    let response = rx
        .recv_timeout(Duration::from_secs(TIMEOUT_SECS))
        .expect("Timeout waiting for response");

    // Cleanup
    drop(stdin);
    let _ = child.kill();
    let _ = child.wait();

    // Verify response
    assert!(
        response.contains("\"result\""),
        "Expected result in response: {response}"
    );
    assert!(
        response.contains("\"protocolVersion\""),
        "Expected protocolVersion: {response}"
    );
    assert!(
        response.contains("\"capabilities\""),
        "Expected capabilities: {response}"
    );

    println!("Response: {response}");
}

#[test]
#[ignore = "requires Docker and CODA_API_TOKEN"]
fn test_docker_mcp_list_tools() {
    if !docker_available() || !image_exists() || get_token().is_none() {
        eprintln!("Skipping: prerequisites not met");
        return;
    }
    let token = get_token().unwrap();

    let mut child = Command::new("docker")
        .args([
            "run",
            "--rm",
            "-i",
            "-e",
            &format!("CODA_API_TOKEN={token}"),
            IMAGE,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start docker");

    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();

    // Initialize first
    writeln!(stdin, r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"protocolVersion":"2024-11-05","capabilities":{{}},"clientInfo":{{"name":"test","version":"1.0"}}}}}}"#).unwrap();
    stdin.flush().unwrap();

    // Send initialized notification (required by MCP protocol)
    writeln!(
        stdin,
        r#"{{"jsonrpc":"2.0","method":"notifications/initialized","params":{{}}}}"#
    )
    .unwrap();
    stdin.flush().unwrap();

    // Then list tools
    writeln!(
        stdin,
        r#"{{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{{}}}}"#
    )
    .unwrap();
    stdin.flush().unwrap();

    let reader = BufReader::new(stdout);
    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        for line in reader.lines().map_while(Result::ok) {
            let _ = tx.send(line);
        }
    });

    // Skip initialize response, get tools/list response
    let _ = rx.recv_timeout(Duration::from_secs(TIMEOUT_SECS));
    let response = rx
        .recv_timeout(Duration::from_secs(TIMEOUT_SECS))
        .expect("Timeout waiting for tools/list");

    drop(stdin);
    let _ = child.kill();
    let _ = child.wait();

    assert!(
        response.contains("list_docs"),
        "Expected list_docs tool: {response}"
    );
    assert!(
        response.contains("get_rows"),
        "Expected get_rows tool: {response}"
    );

    println!("Tools response: {response}");
}
