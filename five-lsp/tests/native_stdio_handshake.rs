#[cfg(feature = "native")]
mod tests {
    use std::io::{BufRead, BufReader, Read, Write};
    use std::path::PathBuf;
    use std::process::{Command, Stdio};
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    use serde_json::Value;

    fn write_lsp_message(stdin: &mut impl Write, value: &Value) {
        let body = value.to_string();
        let header = format!("Content-Length: {}\r\n\r\n", body.len());
        stdin.write_all(header.as_bytes()).expect("write header");
        stdin.write_all(body.as_bytes()).expect("write body");
        stdin.flush().expect("flush stdin");
    }

    fn read_lsp_message(stdout: &mut BufReader<impl Read>) -> Value {
        let mut content_length = None;

        loop {
            let mut line = String::new();
            stdout.read_line(&mut line).expect("read header line");
            if line == "\r\n" {
                break;
            }

            let lower = line.to_ascii_lowercase();
            if lower.starts_with("content-length:") {
                let value = line
                    .split(':')
                    .nth(1)
                    .expect("content-length value")
                    .trim();
                content_length = Some(value.parse::<usize>().expect("content-length number"));
            }
        }

        let length = content_length.expect("content-length present");
        let mut body = vec![0u8; length];
        stdout.read_exact(&mut body).expect("read body");
        serde_json::from_slice(&body).expect("valid json body")
    }

    #[test]
    fn native_server_handles_initialize_and_shutdown() {
        let bin = std::env::var("CARGO_BIN_EXE_five-lsp").unwrap_or_else(|_| {
            let mut candidate = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            candidate.push("../target/debug");
            candidate.push(if cfg!(windows) { "five-lsp.exe" } else { "five-lsp" });
            candidate.to_string_lossy().to_string()
        });
        let mut child = Command::new(bin)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn five-lsp");

        let mut stdin = child.stdin.take().expect("child stdin");
        let stdout = child.stdout.take().expect("child stdout");
        let mut stdout = BufReader::new(stdout);

        let initialize = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "processId": null,
                "rootUri": null,
                "capabilities": {}
            }
        });
        write_lsp_message(&mut stdin, &initialize);

        let init_response = read_lsp_message(&mut stdout);
        assert_eq!(init_response["id"], 1);
        assert_eq!(init_response["jsonrpc"], "2.0");
        assert_eq!(init_response["result"]["serverInfo"]["name"], "Five LSP");

        let shutdown = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "shutdown",
            "params": null
        });
        write_lsp_message(&mut stdin, &shutdown);

        let shutdown_response = read_lsp_message(&mut stdout);
        assert_eq!(shutdown_response["id"], 2);
        assert!(shutdown_response["result"].is_null());

        let exit = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "exit",
            "params": null
        });
        write_lsp_message(&mut stdin, &exit);
        drop(stdin);

        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let status = child.wait().expect("wait child");
            tx.send(status).expect("send status");
        });

        let status = rx
            .recv_timeout(Duration::from_secs(5))
            .expect("server did not exit after exit notification");
        assert!(status.success() || status.code().is_none());
    }
}

#[cfg(not(feature = "native"))]
#[test]
fn native_handshake_skipped_without_native_feature() {
    eprintln!("native feature not enabled; skipping stdio handshake test");
}
