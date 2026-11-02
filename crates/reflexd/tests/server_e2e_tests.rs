use reflex_core::config::ReflexConfig;
use reflex_core::guardrails::GuardrailEngine;
use reflex_core::registry::AppRegistry;
use reflexd::audio::AudioDriver;
use reflexd::notify::NotificationDriver;
use reflexd::server::{run_server_at_path, ServerContext};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::{Mutex, RwLock};

#[tokio::test]
async fn test_server_e2e_cycle() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let socket_path = temp_dir.path().join("reflex.sock");

    let config = ReflexConfig::default();
    let registry = AppRegistry::new();
    let guardrails = GuardrailEngine::default();

    let audio = Arc::new(AudioDriver::new().await.expect("audio driver"));
    let notify = Arc::new(NotificationDriver::new().await.expect("notify driver"));
    let clipboard = Arc::new(reflexd::clipboard::ClipboardDriver::new());

    let ctx = Arc::new(ServerContext {
        hyprland: None, // Standalone test mode without active Hyprland
        audio,
        notify,
        clipboard,
        registry: Arc::new(RwLock::new(registry)),
        guardrails: Arc::new(Mutex::new(guardrails)),
        config,
    });

    let s_path = socket_path.clone();
    let server_ctx = Arc::clone(&ctx);
    tokio::spawn(async move {
        let _ = run_server_at_path(&s_path, server_ctx).await;
    });

    // Wait briefly for server to bind socket
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 1. Test "volume up"
    let mut stream = UnixStream::connect(&socket_path)
        .await
        .expect("connect to reflex.sock");
    stream
        .write_all(b"volume up\n")
        .await
        .expect("write command");
    stream.flush().await.expect("flush command");

    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader
        .read_line(&mut response)
        .await
        .expect("read response");

    let json: Value = serde_json::from_str(&response).expect("parse json");
    assert_eq!(json["status"], "Success");
    assert_eq!(json["action"], "VolumeStep");
    assert!(json["duration_us"].as_u64().unwrap() < 10000); // well under 10ms

    // 2. Test "unknown query" -> PassThrough
    let mut stream2 = UnixStream::connect(&socket_path)
        .await
        .expect("connect stream 2");
    stream2
        .write_all(b"find the latest receipt in Downloads\n")
        .await
        .expect("write command 2");
    stream2.flush().await.expect("flush command 2");

    let mut reader2 = BufReader::new(stream2);
    let mut response2 = String::new();
    reader2
        .read_line(&mut response2)
        .await
        .expect("read response 2");

    let json2: Value = serde_json::from_str(&response2).expect("parse json 2");
    assert_eq!(json2["status"], "PassThrough");
    assert_eq!(json2["reason"], "NoGrammarMatch");

    // 3. Test "set volume 100" -> RejectedByPolicy
    let mut stream3 = UnixStream::connect(&socket_path)
        .await
        .expect("connect stream 3");
    stream3
        .write_all(b"set volume 100\n")
        .await
        .expect("write command 3");
    stream3.flush().await.expect("flush command 3");

    let mut reader3 = BufReader::new(stream3);
    let mut response3 = String::new();
    reader3
        .read_line(&mut response3)
        .await
        .expect("read response 3");

    let json3: Value = serde_json::from_str(&response3).expect("parse json 3");
    assert_eq!(json3["status"], "RejectedByPolicy");
    assert_eq!(json3["reason"], "VolumeVelocityExceeded");
    assert_eq!(json3["clamped_to"], 70);

    // 4. Test "clear clipboard" -> Success
    let mut stream4 = UnixStream::connect(&socket_path)
        .await
        .expect("connect stream 4");
    stream4
        .write_all(b"clear clipboard\n")
        .await
        .expect("write command 4");
    stream4.flush().await.expect("flush command 4");

    let mut reader4 = BufReader::new(stream4);
    let mut response4 = String::new();
    reader4
        .read_line(&mut response4)
        .await
        .expect("read response 4");

    let json4: Value = serde_json::from_str(&response4).expect("parse json 4");
    assert_eq!(json4["status"], "Success");
    assert_eq!(json4["action"], "ClearClipboard");
}
