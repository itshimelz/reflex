use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::process::Command;
use std::thread;
use std::time::Duration;

#[test]
fn test_cli_daemon_offline_exits_2() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let output = Command::new(env!("CARGO_BIN_EXE_reflex"))
        .env("XDG_RUNTIME_DIR", temp_dir.path())
        .arg("volume up")
        .output()
        .expect("Failed to run reflex");

    // Daemon not running -> exit code 2 (routes to fallback)
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn test_cli_empty_input_exits_2() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let output = Command::new(env!("CARGO_BIN_EXE_reflex"))
        .env("XDG_RUNTIME_DIR", temp_dir.path())
        .arg("   ")
        .output()
        .expect("Failed to run reflex");

    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn test_cli_mock_responses() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let socket_path = temp_dir.path().join("reflex.sock");
    let listener = UnixListener::bind(&socket_path).expect("bind mock socket");

    // Spawn mock server thread
    let server_handle = thread::spawn(move || {
        for _ in 0..3 {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                let _ = reader.read_line(&mut line);
                let trimmed = line.trim();

                let resp = match trimmed {
                    "volume up" => {
                        r#"{"status":"Success","action":"VolumeStep","duration_us":120}"#
                    }
                    "unknown query" => {
                        r#"{"status":"PassThrough","reason":"NoGrammarMatch","duration_us":45}"#
                    }
                    "set volume 100" => {
                        r#"{"status":"RejectedByPolicy","reason":"VolumeVelocityExceeded","clamped_to":65,"duration_us":80}"#
                    }
                    _ => r#"{"status":"Error","duration_us":10}"#,
                };

                let _ = stream.write_all(format!("{}\n", resp).as_bytes());
            }
        }
    });

    thread::sleep(Duration::from_millis(50));

    // 1. Success -> exit code 0
    let out_success = Command::new(env!("CARGO_BIN_EXE_reflex"))
        .env("XDG_RUNTIME_DIR", temp_dir.path())
        .arg("volume up")
        .output()
        .expect("run reflex");
    assert_eq!(out_success.status.code(), Some(0));

    // 2. PassThrough -> exit code 2
    let out_passthrough = Command::new(env!("CARGO_BIN_EXE_reflex"))
        .env("XDG_RUNTIME_DIR", temp_dir.path())
        .arg("unknown query")
        .output()
        .expect("run reflex");
    assert_eq!(out_passthrough.status.code(), Some(2));

    // 3. RejectedByPolicy -> exit code 1
    let out_rejected = Command::new(env!("CARGO_BIN_EXE_reflex"))
        .env("XDG_RUNTIME_DIR", temp_dir.path())
        .arg("set volume 100")
        .output()
        .expect("run reflex");
    assert_eq!(out_rejected.status.code(), Some(1));

    let _ = server_handle.join();
}
