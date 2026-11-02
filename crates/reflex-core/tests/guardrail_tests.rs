use reflex_core::guardrails::{GuardrailDecision, GuardrailEngine};
use reflex_core::registry::HyprlandClient;
use std::thread::sleep;
use std::time::Duration;

#[test]
fn test_volume_ceiling_and_velocity_clamping() {
    let mut engine = GuardrailEngine {
        current_volume: 40,
        ..Default::default()
    };

    // Velocity clamp: jump from 40 to 100 exceeds +15% velocity limit -> clamped to 55
    let (clamped, decision) = engine.evaluate_volume_set(100);
    assert_eq!(clamped, 55);
    assert_eq!(
        decision,
        GuardrailDecision::VelocityClamped {
            requested: 100,
            clamped: 55
        }
    );

    // Update volume to 75
    engine.current_volume = 75;
    // Ceiling clamp: jump to 85 breaches 80% ceiling -> clamped to 80
    let (clamped, decision) = engine.evaluate_volume_set(85);
    assert_eq!(clamped, 80);
    assert_eq!(
        decision,
        GuardrailDecision::CeilingClamped {
            requested: 85,
            clamped: 80
        }
    );

    // Downward adjustment: unrestricted velocity down
    let (down, decision) = engine.evaluate_volume_step(-20);
    assert_eq!(down, -20);
    assert_eq!(decision, GuardrailDecision::Allowed);
}

#[test]
fn test_unkillable_window_confirmation_and_address_pinning() {
    let mut engine = GuardrailEngine::default();

    let kitty = HyprlandClient {
        address: "0x55a123b".to_string(),
        class: "kitty".to_string(),
        title: "terminal".to_string(),
        workspace: 1,
    };

    let ordinary = HyprlandClient {
        address: "0x77c456".to_string(),
        class: "galculator".to_string(),
        title: "Calculator".to_string(),
        workspace: 2,
    };

    // Ordinary window can close immediately
    assert_eq!(
        engine.evaluate_window_close(&ordinary),
        GuardrailDecision::Allowed
    );

    // Protected window triggers confirmation
    let decision = engine.evaluate_window_close(&kitty);
    assert_eq!(
        decision,
        GuardrailDecision::RequiresConfirmation {
            target_address: "0x55a123b".to_string(),
            window_class: "kitty".to_string(),
        }
    );

    // Immediate confirmation succeeds on pinned address
    assert_eq!(engine.confirm(), Some("0x55a123b".to_string()));
    // Second confirmation fails as state was consumed
    assert_eq!(engine.confirm(), None);
}

#[test]
fn test_confirmation_timeout_expiration() {
    let mut engine = GuardrailEngine {
        confirmation_timeout: Duration::from_millis(50),
        ..Default::default()
    };

    let protected = HyprlandClient {
        address: "0x99e".to_string(),
        class: "neovim".to_string(),
        title: "nvim src/main.rs".to_string(),
        workspace: 1,
    };

    engine.evaluate_window_close(&protected);
    sleep(Duration::from_millis(60));

    // After expiration, confirm returns None
    assert_eq!(engine.confirm(), None);
}
