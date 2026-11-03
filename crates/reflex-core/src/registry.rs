//! AppRegistry: Dynamic harvester and 4-tier resolution engine for desktop entries and PWAs.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HyprlandClient {
    pub address: String,
    pub class: String,
    pub title: String,
    pub workspace: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolutionTier {
    Tier1ExactClass,
    Tier2UserAlias,
    Tier3TitleToken,
    Tier4Fuzzy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppEntry {
    pub id: String,
    pub name: String,
    pub exec: String,
    pub wm_class: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct AppRegistry {
    pub desktop_entries: HashMap<String, AppEntry>,
    pub aliases: HashMap<String, String>,
}

impl AppRegistry {
    pub fn new() -> Self {
        Self {
            desktop_entries: HashMap::new(),
            aliases: HashMap::new(),
        }
    }

    pub fn set_aliases(&mut self, aliases: HashMap<String, String>) {
        self.aliases = aliases;
    }

    /// Harvest .desktop files from specified directory paths
    pub fn harvest(&mut self, search_dirs: &[&Path]) {
        for dir in search_dirs {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("desktop") {
                        if let Ok(content) = fs::read_to_string(&path) {
                            if let Some(app) = parse_desktop_entry(&path, &content) {
                                let name_key = app.name.to_lowercase();
                                let id_key = app.id.to_lowercase();
                                self.desktop_entries.insert(name_key, app.clone());
                                self.desktop_entries.insert(id_key, app);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Resolve spoken target against a slice of live open Hyprland clients using the 4-tier algorithm
    pub fn resolve_client<'a>(
        &'a self,
        target: &str,
        clients: &'a [HyprlandClient],
    ) -> Option<(&'a HyprlandClient, ResolutionTier)> {
        let clean_target = target.trim().to_lowercase();
        if clean_target.is_empty() {
            return None;
        }

        // Tier 1: Exact Desktop ID / StartupWMClass / Client Class Match
        for client in clients {
            let client_class = client.class.to_lowercase();
            if client_class == clean_target {
                return Some((client, ResolutionTier::Tier1ExactClass));
            }
            if let Some(entry) = self.desktop_entries.get(&clean_target) {
                if let Some(ref wm_class) = entry.wm_class {
                    if wm_class.to_lowercase() == client_class {
                        return Some((client, ResolutionTier::Tier1ExactClass));
                    }
                }
                if entry.id.to_lowercase() == client_class {
                    return Some((client, ResolutionTier::Tier1ExactClass));
                }
            }
        }

        // Tier 2: User-Configured Aliases in config.toml
        if let Some(target_class) = self.aliases.get(&clean_target) {
            let target_class_lower = target_class.to_lowercase();
            for client in clients {
                if client.class.to_lowercase() == target_class_lower
                    || client.class.to_lowercase().contains(&target_class_lower)
                {
                    return Some((client, ResolutionTier::Tier2UserAlias));
                }
            }
        }

        // Tier 3: Isolated Title Token Match (split by -, |, •, —, :)
        let delimiters = ['-', '|', '•', '—', ':'];
        for client in clients {
            let tokens = client.title.split(&delimiters[..]);
            for token in tokens {
                let clean_token = token.trim().to_lowercase();
                if clean_token == clean_target {
                    return Some((client, ResolutionTier::Tier3TitleToken));
                }
            }
        }

        // Tier 4: Bounded Levenshtein Substring Match (length >= 5, distance <= 1)
        if clean_target.len() >= 5 {
            for client in clients {
                let client_class = client.class.to_lowercase();
                if levenshtein_distance_le1(&clean_target, &client_class) {
                    return Some((client, ResolutionTier::Tier4Fuzzy));
                }
                // Also check primary title tokens
                let tokens = client.title.split(&delimiters[..]);
                for token in tokens {
                    let clean_token = token.trim().to_lowercase();
                    if clean_token.len() >= 5
                        && levenshtein_distance_le1(&clean_target, &clean_token)
                    {
                        return Some((client, ResolutionTier::Tier4Fuzzy));
                    }
                }
            }
        }

        None
    }

    /// Resolve launch command for an application name
    pub fn resolve_launch_exec(&self, target: &str) -> Option<String> {
        let clean = target.trim().to_lowercase();
        if clean.is_empty() {
            return None;
        }

        // 1. Direct match on desktop entry name or ID
        if let Some(entry) = self.desktop_entries.get(&clean) {
            let cmd = clean_exec_command(&entry.exec);
            if !cmd.is_empty() {
                return Some(cmd);
            }
        }

        // 2. Check user aliases (e.g. youtube -> brave-agimnkijcaahngcdmfeangaknmldooml-Default)
        if let Some(alias_target) = self.aliases.get(&clean) {
            let alias_lower = alias_target.to_lowercase();
            if let Some(entry) = self.desktop_entries.get(&alias_lower) {
                let cmd = clean_exec_command(&entry.exec);
                if !cmd.is_empty() {
                    return Some(cmd);
                }
            }
            for entry in self.desktop_entries.values() {
                if entry.id.to_lowercase() == alias_lower
                    || entry.name.to_lowercase() == alias_lower
                {
                    let cmd = clean_exec_command(&entry.exec);
                    if !cmd.is_empty() {
                        return Some(cmd);
                    }
                }
            }
        }

        // 3. Substring match in desktop entry name or ID
        for entry in self.desktop_entries.values() {
            if entry.name.to_lowercase().contains(&clean)
                || entry.id.to_lowercase().contains(&clean)
            {
                let cmd = clean_exec_command(&entry.exec);
                if !cmd.is_empty() {
                    return Some(cmd);
                }
            }
        }

        // 4. Fuzzy Levenshtein <= 1 if length >= 5
        if clean.len() >= 5 {
            for entry in self.desktop_entries.values() {
                let name_lower = entry.name.to_lowercase();
                if levenshtein_distance_le1(&clean, &name_lower) {
                    let cmd = clean_exec_command(&entry.exec);
                    if !cmd.is_empty() {
                        return Some(cmd);
                    }
                }
            }
        }

        None
    }
}

/// Strip parameter placeholders (%U, %u, %f, %F, etc.) from desktop Exec strings
pub fn clean_exec_command(exec: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for token in exec.split_whitespace() {
        if !token.starts_with('%') {
            parts.push(token);
        }
    }
    parts.join(" ")
}

/// Parse a single .desktop file content
fn parse_desktop_entry(path: &Path, content: &str) -> Option<AppEntry> {
    let id = path.file_stem()?.to_string_lossy().to_string();
    let mut name = None;
    let mut exec = None;
    let mut wm_class = None;
    let mut in_desktop_entry = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[Desktop Entry]" {
            in_desktop_entry = true;
            continue;
        } else if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_desktop_entry = false;
            continue;
        }

        if in_desktop_entry {
            if let Some(val) = trimmed.strip_prefix("Name=") {
                if name.is_none() {
                    name = Some(val.trim().to_string());
                }
            } else if let Some(val) = trimmed.strip_prefix("Exec=") {
                if exec.is_none() {
                    exec = Some(val.trim().to_string());
                }
            } else if let Some(val) = trimmed.strip_prefix("StartupWMClass=") {
                if wm_class.is_none() {
                    wm_class = Some(val.trim().to_string());
                }
            }
        }
    }

    Some(AppEntry {
        id,
        name: name?,
        exec: exec.unwrap_or_default(),
        wm_class,
    })
}

/// Fast check whether two ASCII/UTF-8 strings have a Levenshtein distance <= 1
fn levenshtein_distance_le1(s1: &str, s2: &str) -> bool {
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();

    if len1.abs_diff(len2) > 1 {
        return false;
    }

    let mut diff = 0;
    if len1 == len2 {
        for (c1, c2) in s1.chars().zip(s2.chars()) {
            if c1 != c2 {
                diff += 1;
                if diff > 1 {
                    return false;
                }
            }
        }
        return diff <= 1;
    }

    // One deletion/insertion
    let (shorter, longer) = if len1 < len2 { (s1, s2) } else { (s2, s1) };
    let mut short_chars = shorter.chars();
    let mut long_chars = longer.chars();

    let mut short_c = short_chars.next();
    let mut long_c = long_chars.next();
    let mut skipped = false;

    while let (Some(sc), Some(lc)) = (short_c, long_c) {
        if sc == lc {
            short_c = short_chars.next();
            long_c = long_chars.next();
        } else if !skipped {
            skipped = true;
            long_c = long_chars.next();
        } else {
            return false;
        }
    }

    true
}

/// Score matching between a spoken target string and a UI candidate label (0..=100)
pub fn fuzzy_match_element(target: &str, candidate_name: &str) -> Option<u32> {
    let t = target.trim().to_lowercase();
    let c = candidate_name.trim().to_lowercase();

    if t.is_empty() || c.is_empty() {
        return None;
    }

    // 1. Exact match
    if t == c {
        return Some(100);
    }

    // 2. Prefix match
    if c.starts_with(&t) {
        return Some(90);
    }

    // 3. Substring match
    if c.contains(&t) {
        return Some(80);
    }

    // 4. Token-level matching
    let delimiters = [' ', '-', '_', ':', '|', '•', '(', ')', '[', ']'];
    let c_tokens: Vec<&str> = c.split(&delimiters[..]).filter(|s| !s.is_empty()).collect();
    for token in &c_tokens {
        if *token == t {
            return Some(75);
        }
        if t.len() >= 4 && token.len() >= 4 && levenshtein_distance_le1(&t, token) {
            return Some(70);
        }
    }

    // 5. Multi-token target matching: all words in target appear in candidate
    let t_words: Vec<&str> = t.split_whitespace().collect();
    if t_words.len() > 1 && t_words.iter().all(|w| c.contains(w)) {
        return Some(75);
    }

    None
}
