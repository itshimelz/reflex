//! reflex: Sub-millisecond ephemeral CLI client.

use serde::Deserialize;
use std::env;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::process;
use std::time::Duration;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct IpcResponse {
    status: String,
    reason: Option<String>,
    action: Option<String>,
    clamped_to: Option<u8>,
    duration_us: Option<u128>,
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let transcript = if !args.is_empty() {
        args.join(" ")
    } else {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).unwrap_or_default();
        buffer
    };

    let trimmed = transcript.trim();
    if trimmed.is_empty() {
        process::exit(2);
    }

    let socket_path = match env::var("XDG_RUNTIME_DIR") {
        Ok(dir) => format!("{}/reflex.sock", dir),
        Err(_) => "/run/user/1000/reflex.sock".to_string(),
    };

    let stream = match UnixStream::connect(&socket_path) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("reflexd daemon not running at {}", socket_path);
            process::exit(2);
        }
    };

    // Set 2.0s read/write timeout for defensive safety
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));

    let mut writer = stream.try_clone().expect("Failed to clone socket stream");
    let request = format!("{}\n", trimmed);
    if writer.write_all(request.as_bytes()).is_err() {
        process::exit(1);
    }

    let mut reader = BufReader::new(stream);
    let mut response_line = String::new();
    if reader.read_line(&mut response_line).is_err() || response_line.trim().is_empty() {
        process::exit(1);
    }

    let response_str = response_line.trim();
    println!("{}", response_str);

    if let Ok(resp) = serde_json::from_str::<IpcResponse>(response_str) {
        match resp.status.as_str() {
            "Success" => process::exit(0),
            "PassThrough" => process::exit(2),
            _ => process::exit(1),
        }
    }

    process::exit(1);
}
