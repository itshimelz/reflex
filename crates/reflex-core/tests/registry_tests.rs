use reflex_core::registry::{AppRegistry, HyprlandClient, ResolutionTier};
use std::collections::HashMap;

#[test]
fn test_tier1_exact_wmclass_match() {
    let registry = AppRegistry::new();
    let clients = vec![
        HyprlandClient {
            address: "0x55a1".to_string(),
            class: "brave-browser".to_string(),
            title: "GitHub - Mozilla Firefox".to_string(),
            workspace: 1,
        },
        HyprlandClient {
            address: "0x55a2".to_string(),
            class: "kitty".to_string(),
            title: "fish ~/Tools".to_string(),
            workspace: 2,
        },
    ];

    let result = registry.resolve_client("kitty", &clients);
    assert!(result.is_some());
    let (matched, tier) = result.unwrap();
    assert_eq!(matched.address, "0x55a2");
    assert_eq!(tier, ResolutionTier::Tier1ExactClass);
}

#[test]
fn test_tier2_configured_alias_match() {
    let mut registry = AppRegistry::new();
    let mut aliases = HashMap::new();
    aliases.insert(
        "youtube".to_string(),
        "brave-agimnkijcaahngcdmfeangaknmldooml-Default".to_string(),
    );
    registry.set_aliases(aliases);

    let clients = vec![HyprlandClient {
        address: "0x66b1".to_string(),
        class: "brave-agimnkijcaahngcdmfeangaknmldooml-Default".to_string(),
        title: "YouTube - Brave".to_string(),
        workspace: 3,
    }];

    let result = registry.resolve_client("youtube", &clients);
    assert!(result.is_some());
    let (matched, tier) = result.unwrap();
    assert_eq!(matched.address, "0x66b1");
    assert_eq!(tier, ResolutionTier::Tier2UserAlias);
}

#[test]
fn test_tier3_isolated_title_token_match() {
    let registry = AppRegistry::new();
    let clients = vec![HyprlandClient {
        address: "0x77c1".to_string(),
        class: "webapp-generic".to_string(),
        title: "general | Slack | Acme Corp".to_string(),
        workspace: 4,
    }];

    let result = registry.resolve_client("slack", &clients);
    assert!(result.is_some());
    let (matched, tier) = result.unwrap();
    assert_eq!(matched.address, "0x77c1");
    assert_eq!(tier, ResolutionTier::Tier3TitleToken);
}

#[test]
fn test_tier4_bounded_fuzzy_match() {
    let registry = AppRegistry::new();
    let clients = vec![HyprlandClient {
        address: "0x88d1".to_string(),
        class: "spotify".to_string(),
        title: "Spotify Free".to_string(),
        workspace: 5,
    }];

    // Distance 1: "spotifi" vs "spotify"
    let result = registry.resolve_client("spotifi", &clients);
    assert!(result.is_some());
    let (matched, tier) = result.unwrap();
    assert_eq!(matched.address, "0x88d1");
    assert_eq!(tier, ResolutionTier::Tier4Fuzzy);

    // Reject too distant or short words
    assert!(registry.resolve_client("spot", &clients).is_none());
    assert!(registry.resolve_client("unknownapp", &clients).is_none());
}

#[test]
fn test_resolve_launch_exec() {
    use reflex_core::registry::AppEntry;

    let mut registry = AppRegistry::new();
    let entry = AppEntry {
        id: "brave-agimnkijcaahngcdmfeangaknmldooml-Default".to_string(),
        name: "YouTube".to_string(),
        exec: "/usr/bin/brave --profile-directory=Default --app-id=agimnkijcaahngcdmfeangaknmldooml %U".to_string(),
        wm_class: Some("crx_agimnkijcaahngcdmfeangaknmldooml".to_string()),
    };
    registry
        .desktop_entries
        .insert("youtube".to_string(), entry.clone());
    registry
        .desktop_entries
        .insert(entry.id.to_lowercase(), entry);

    // 1. Direct name lookup
    let cmd = registry.resolve_launch_exec("youtube");
    assert_eq!(
        cmd,
        Some(
            "/usr/bin/brave --profile-directory=Default --app-id=agimnkijcaahngcdmfeangaknmldooml"
                .to_string()
        )
    );

    // 2. Lookup via alias
    let mut aliases = HashMap::new();
    aliases.insert("yt".to_string(), "youtube".to_string());
    registry.set_aliases(aliases);

    let cmd_alias = registry.resolve_launch_exec("yt");
    assert_eq!(
        cmd_alias,
        Some(
            "/usr/bin/brave --profile-directory=Default --app-id=agimnkijcaahngcdmfeangaknmldooml"
                .to_string()
        )
    );

    // 3. Unknown app returns None
    assert!(registry.resolve_launch_exec("nonexistentapp").is_none());
}
