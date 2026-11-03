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
        .replace("head phone", "headphones")
        .replace("time stamp", "timestamp")
        .replace("date stamp", "timestamp")
        .replace("datestamp", "timestamp")
        .replace("meta data", "metadata")
        .replace("check sum", "checksum")
        .replace("sha 256", "sha256")
        .replace("sha two fifty six", "sha256")
        .replace("down loads", "downloads")
        .replace("down load", "downloads")
        .replace("re name", "rename")
        .replace("ri name", "rename")
        .replace("dupe it", "duplicate this")
        .replace("duplicate it", "duplicate this")
        .replace("trash it", "trash this")
        .replace("copy it", "copy this")
        .replace("clean it", "clean filename")
        .replace("sanitize it", "clean filename")
        .replace("strip it", "strip metadata")
        .replace("full pass", "full path")
        .replace("full pat", "full path")
        .replace("clip board", "clipboard")
        .replace("em pee three", "mp3")
        .replace("mp 3", "mp3")
        .replace("pee en gee", "png")
        .replace("p n g", "png")
        .replace("jay peg", "jpg")
        .replace("j p g", "jpg")
        .replace("web p", "webp")
        .replace("pee dee eff", "pdf")
        .replace("p d f", "pdf");

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
            "coppy" | "coppi" | "kopy" | "copee" | "copi" => *word = "copy".to_string(),
            "pass" | "pat" | "pash" | "paath" => *word = "path".to_string(),
            "tresh" | "trush" | "treash" | "trashed" => *word = "trash".to_string(),
            "delit" | "dilit" | "delet" => *word = "delete".to_string(),
            "duplicat" | "dublicate" | "dublicat" | "doop" | "dupe" => *word = "duplicate".to_string(),
            "conbert" | "kanvert" | "canvert" => *word = "convert".to_string(),
            "adio" | "odiyo" | "audo" => *word = "audio".to_string(),
            "polder" | "pholder" | "foldar" => *word = "folder".to_string(),
            "dawnload" | "dawnloads" | "download" => *word = "downloads".to_string(),
            "dokuments" | "documants" | "docment" => *word = "documents".to_string(),
            "deskop" | "desktp" => *word = "desktop".to_string(),
            "pikchurs" | "piktures" | "photos" | "potos" => *word = "pictures".to_string(),
            "imaje" | "imij" | "imege" => *word = "image".to_string(),
            "imajes" | "imiges" | "imeges" => *word = "images".to_string(),
            "jif" | "giph" | "geef" => *word = "gif".to_string(),
            "terminel" | "tarminal" | "termnl" => *word = "terminal".to_string(),
            "klin" | "klean" => *word = "clean".to_string(),
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
                || (last == "copy" && w == "copy")
                || (last == "trash" && w == "trash")
                || (last == "delete" && w == "delete")
                || (last == "duplicate" && w == "duplicate")
                || (last == "convert" && w == "convert")
                || (last == "move" && w == "move")
                || (last == "clean" && w == "clean")
            {
                continue;
            }
        }
        deduped_words.push(w);
    }
    s = deduped_words.join(" ");

    s
}
