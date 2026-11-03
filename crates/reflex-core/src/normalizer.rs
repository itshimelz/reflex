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
        .replace("screen cap", "screenshot")
        .replace("toggle plot", "toggle float")
        .replace("toggle flot", "toggle float")
        .replace("plot window", "float window")
        .replace("flot window", "float window")
        .replace("plot it", "float it")
        .replace("flot it", "float it")
        .replace("unplot", "unfloat")
        .replace("unflot", "unfloat")
        .replace("page down", "scroll down")
        .replace("page up", "scroll up")
        .replace("click on ", "click ")
        .replace("tap on ", "click ")
        .replace("press on ", "click ")
        .replace("focus on ", "focus ")
        .replace("type into ", "type ")
        .replace("type in ", "type ")
        .replace("sound sound ", "sound ")
        .replace("volume volume ", "volume ")
        .replace("turn sound ", "sound ")
        .replace("turn volume ", "volume ")
        .replace("zip up", "zip")
        .replace("zip, zip", "zip")
        .replace("jip, jip", "zip")
        .replace("unzip, unzip", "unzip")
        .replace("unjip, unjip", "unzip")
        .replace("unchip, unchip", "unzip")
        .replace("zip zip", "zip")
        .replace("jip jip", "zip")
        .replace("unzip unzip", "unzip")
        .replace("unjip unjip", "unzip")
        .replace("unchip unchip", "unzip")
        .replace("compress it", "compress this")
        .replace("extract it", "extract this")
        .replace("unzip it", "unzip this")
        .replace("un jip", "unzip")
        .replace("an jip", "unzip")
        .replace("on jip", "unzip")
        .replace("un chip", "unzip")
        .replace("an chip", "unzip")
        .replace("on chip", "unzip")
        .replace("un jeep", "unzip")
        .replace("an jeep", "unzip")
        .replace("un mute", "unmute")
        .replace("head phones", "headphones")
        .replace("head phone", "headphones");

    // Token-based phonetic / accent / homophone variations
    let mut words: Vec<String> = s.split_whitespace().map(|w| w.to_string()).collect();
    for word in &mut words {
        match word.as_str() {
            "paws" => *word = "pause".to_string(),
            "mike" => *word = "mic".to_string(),
            "togoli" | "togol" | "toggole" | "togle" | "togal" => *word = "toggle".to_string(),
            "speeker" | "spkr" => *word = "speaker".to_string(),
            "valum" | "valume" | "volum" | "bolume" => *word = "volume".to_string(),
            "flot" | "flote" | "floot" => *word = "float".to_string(),
            "floting" | "ploting" | "plotting" => *word = "floating".to_string(),
            "jip" | "gyp" | "jeep" | "zyp" | "zep" | "zipp" | "jib" => *word = "zip".to_string(),
            "unjip" | "anjip" | "onjip" | "anjipp" | "onjipp" | "unchip" | "anchip" | "onchip"
            | "unjeep" | "anjeep" | "onjeep" | "onzip" => *word = "unzip".to_string(),
            "extrac" | "extrack" => *word = "extract".to_string(),
            "compres" => *word = "compress".to_string(),
            _ => {}
        }
    }

    // Deduplicate consecutive repeated action tokens: ["zip", "zip"] -> ["zip"]
    let mut deduped_words: Vec<String> = Vec::with_capacity(words.len());
    for w in words {
        if let Some(last) = deduped_words.last() {
            if (last == "zip" && w == "zip")
                || (last == "unzip" && w == "unzip")
                || (last == "sound" && w == "sound")
                || (last == "volume" && w == "volume")
                || (last == "extract" && w == "extract")
            {
                continue;
            }
        }
        deduped_words.push(w);
    }
    s = deduped_words.join(" ");

    s
}
