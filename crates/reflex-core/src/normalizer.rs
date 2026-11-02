//! In-memory phonetic homophones, split words, and filler stripping.

pub fn normalize_transcript(input: &str) -> String {
    let mut s = input.trim().to_lowercase();

    // Strip trailing punctuation (. ! ? ;)
    while s.ends_with('.') || s.ends_with('!') || s.ends_with('?') || s.ends_with(';') {
        s.pop();
        s = s.trim_end().to_string();
    }

    // Strip common filler preambles (loop until no preambles remain)
    let preambles = [
        "can you please",
        "could you please",
        "would you please",
        "take a",
        "please",
        "can you",
        "could you",
        "would you",
        "now",
        "just",
        "hey",
    ];

    let mut changed = true;
    while changed {
        changed = false;
        for p in &preambles {
            if s.starts_with(p) {
                let remainder = &s[p.len()..];
                if remainder.is_empty() || remainder.starts_with(' ') {
                    s = remainder.trim_start().to_string();
                    changed = true;
                    break;
                }
            }
        }
    }

    // Common split-word replacements
    s = s
        .replace("work space", "workspace")
        .replace("workplace", "workspace")
        .replace("you tube", "youtube")
        .replace("whats app", "whatsapp")
        .replace("screen shot", "screenshot")
        .replace("screen cap", "screenshot");

    // Token-based phonetic / accent / homophone variations
    let mut words: Vec<String> = s.split_whitespace().map(|w| w.to_string()).collect();
    for word in &mut words {
        match word.as_str() {
            "paws" => *word = "pause".to_string(),
            "mike" => *word = "mic".to_string(),
            "togoli" | "togol" | "toggole" | "togle" | "togal" => *word = "toggle".to_string(),
            "speeker" | "spkr" => *word = "speaker".to_string(),
            "valum" | "valume" | "volum" | "bolume" => *word = "volume".to_string(),
            _ => {}
        }
    }
    s = words.join(" ");

    s
}
