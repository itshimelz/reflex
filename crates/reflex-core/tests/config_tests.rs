use reflex_core::config::ReflexConfig;
use reflex_core::earcon::{EarconKind, SAMPLE_RATE};

#[test]
fn test_default_config_values() {
    let config = ReflexConfig::default();
    assert_eq!(config.volume.ceiling, 80);
    assert_eq!(config.volume.max_step_velocity, 15);
    assert_eq!(config.guardrails.confirmation_timeout_secs, 4.0);
    assert!(config
        .guardrails
        .unkillable_classes
        .contains(&"kitty".to_string()));
    assert!(config
        .guardrails
        .unkillable_classes
        .contains(&"brave".to_string()));
    assert!(config
        .guardrails
        .unkillable_classes
        .contains(&"antigravity".to_string()));
}

#[test]
fn test_parse_toml_string() {
    let toml_str = r#"
        [volume]
        ceiling = 75
        max_step_velocity = 10

        [guardrails]
        confirmation_timeout_secs = 5.0
        unkillable_classes = ["alacritty", "firefox"]

        [aliases]
        yt = "brave-agimnkijcaahngcdmfeangaknmldooml-Default"
    "#;

    let config: ReflexConfig = toml::from_str(toml_str).expect("Valid TOML");
    assert_eq!(config.volume.ceiling, 75);
    assert_eq!(config.volume.max_step_velocity, 10);
    assert_eq!(config.guardrails.confirmation_timeout_secs, 5.0);
    assert_eq!(
        config.guardrails.unkillable_classes,
        vec!["alacritty", "firefox"]
    );
    assert_eq!(
        config.aliases.get("yt").map(|s| s.as_str()),
        Some("brave-agimnkijcaahngcdmfeangaknmldooml-Default")
    );
}

#[test]
fn test_earcon_generation() {
    for kind in &[
        EarconKind::Success,
        EarconKind::Warning,
        EarconKind::ConfirmationChime,
        EarconKind::Shutter,
        EarconKind::Error,
    ] {
        let samples = kind.pcm_samples();
        assert!(
            !samples.is_empty(),
            "Samples for {:?} should not be empty",
            kind
        );
        let bytes = kind.pcm_bytes();
        assert_eq!(bytes.len(), samples.len() * 2);
    }
    assert_eq!(SAMPLE_RATE, 48000);
}
