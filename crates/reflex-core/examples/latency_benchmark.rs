//! Latency benchmark measuring normalizer, parser, guardrails, and registry resolution.

use reflex_core::guardrails::GuardrailEngine;
use reflex_core::normalizer::normalize_transcript;
use reflex_core::parser::parse_action;
use reflex_core::registry::{AppRegistry, HyprlandClient};
use std::collections::HashMap;
use std::time::Instant;

fn main() {
    println!("============================================================");
    println!("       REFLEX LATENCY BENCHMARK (AMD Ryzen 5 3400G)         ");
    println!("============================================================");

    let iterations = 20_000;

    // 1. Normalizer Benchmark
    let raw_sample = "please can you volume up 10";
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = normalize_transcript(raw_sample);
    }
    let norm_elapsed = start.elapsed();
    let norm_ns = norm_elapsed.as_nanos() as f64 / iterations as f64;
    println!(
        "1. Normalizer Latency:      {:>8.2} ns/op ({:.3} μs)",
        norm_ns,
        norm_ns / 1000.0
    );

    // 2. Parser Benchmark
    let clean_sample = "move youtube to workspace 4";
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = parse_action(clean_sample);
    }
    let parse_elapsed = start.elapsed();
    let parse_ns = parse_elapsed.as_nanos() as f64 / iterations as f64;
    println!(
        "2. Lexical Parser Latency:  {:>8.2} ns/op ({:.3} μs)",
        parse_ns,
        parse_ns / 1000.0
    );

    // 3. Guardrail Evaluation Benchmark
    let mut guardrails = GuardrailEngine::default();
    let start = Instant::now();
    for i in 0..iterations {
        let target = ((i % 100) + 1) as u8;
        let _ = guardrails.evaluate_volume_set(target);
    }
    let guard_elapsed = start.elapsed();
    let guard_ns = guard_elapsed.as_nanos() as f64 / iterations as f64;
    println!(
        "3. Guardrail Policy Latency:{:>8.2} ns/op ({:.3} μs)",
        guard_ns,
        guard_ns / 1000.0
    );

    // 4. AppRegistry 4-Tier Resolution Benchmark
    let mut registry = AppRegistry::new();
    let mut aliases = HashMap::new();
    aliases.insert(
        "youtube".to_string(),
        "brave-agimnkijcaahngcdmfeangaknmldooml-Default".to_string(),
    );
    registry.set_aliases(aliases);

    let clients = vec![
        HyprlandClient {
            address: "0x1".to_string(),
            class: "kitty".to_string(),
            title: "terminal".to_string(),
            workspace: 1,
        },
        HyprlandClient {
            address: "0x2".to_string(),
            class: "brave-agimnkijcaahngcdmfeangaknmldooml-Default".to_string(),
            title: "YouTube - Brave".to_string(),
            workspace: 2,
        },
        HyprlandClient {
            address: "0x3".to_string(),
            class: "code".to_string(),
            title: "reflex-core/src/lib.rs - Code".to_string(),
            workspace: 3,
        },
    ];

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = registry.resolve_client("youtube", &clients);
    }
    let reg_elapsed = start.elapsed();
    let reg_ns = reg_elapsed.as_nanos() as f64 / iterations as f64;
    println!(
        "4. AppRegistry Match Latency:{:>7.2} ns/op ({:.3} μs)",
        reg_ns,
        reg_ns / 1000.0
    );

    let total_decision_ns = norm_ns + parse_ns + guard_ns + reg_ns;
    println!("------------------------------------------------------------");
    println!(
        "TOTAL DECISION LATENCY:     {:>8.2} ns ({:.3} μs)",
        total_decision_ns,
        total_decision_ns / 1000.0
    );
    println!(
        "SLA TARGET (<50 μs):         {}",
        if total_decision_ns / 1000.0 < 50.0 {
            "✅ MET"
        } else {
            "❌ BREACHED"
        }
    );
    println!("============================================================");
}
