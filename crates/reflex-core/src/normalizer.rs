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
        .replace("shah 256", "sha256")
        .replace("shah256", "sha256")
        .replace("down loads", "downloads")
        .replace("down load", "downloads")
        .replace("re name", "rename")
        .replace("ri name", "rename")
        .replace("dupe it", "duplicate this")
        .replace("duplicate it", "duplicate this")
        .replace("trash it", "trash this")
        .replace("copy it", "copy this")
        .replace("file name", "filename")
        .replace("clean filename clean", "clean filename")
        .replace("clean it", "clean filename")
        .replace("sanitize it", "clean filename")
        .replace("strip it", "strip metadata")
        .replace("lib board", "clipboard")
        .replace("lip board", "clipboard")
        .replace("clip bord", "clipboard")
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
        let clean = word.trim_matches(|c: char| c == ',' || c == '.' || c == ';' || c == '!');
        let mapped = match clean {
            "weppy" | "webby" | "webi" | "wepy" | "weby" | "web-p" => Some("webp"),
            "paws" => Some("pause"),
            "mike" => Some("mic"),
            "togoli" | "togol" | "toggole" | "togle" | "togal" => Some("toggle"),
            "speeker" | "spkr" => Some("speaker"),
            "valum" | "valume" | "volum" | "bolume" => Some("volume"),
            "flot" | "flote" | "floot" => Some("float"),
            "floting" | "ploting" | "plotting" => Some("floating"),
            "jip" | "gyp" | "jeep" | "zyp" | "zep" | "zipp" | "jib" => Some("zip"),
            "unjip" | "anjip" | "onjip" | "anjipp" | "onjipp" | "unchip" | "anchip" | "onchip"
            | "unjeep" | "anjeep" | "onjeep" | "onzip" => Some("unzip"),
            "extrac" | "extrack" => Some("extract"),
            "compres" => Some("compress"),
            "coppy" | "coppi" | "kopy" | "copee" | "copi" => Some("copy"),
            "pass" | "pat" | "pash" | "paath" => Some("path"),
            "tresh" | "trush" | "treash" | "trashed" => Some("trash"),
            "delit" | "dilit" | "delet" => Some("delete"),
            "duplicat" | "dublicate" | "dublicat" | "doop" | "dupe" => Some("duplicate"),
            "conbert" | "kanvert" | "canvert" => Some("convert"),
            "adio" | "odiyo" | "audo" => Some("audio"),
            "polder" | "pholder" | "foldar" => Some("folder"),
            "dawnload" | "dawnloads" | "download" => Some("downloads"),
            "dokuments" | "documants" | "docment" => Some("documents"),
            "deskop" | "desktp" => Some("desktop"),
            "pikchurs" | "piktures" | "photos" | "potos" => Some("pictures"),
            "vedio" | "vedios" | "vidio" | "vidios" => Some("videos"),
            "imaje" | "imij" | "imege" => Some("image"),
            "imajes" | "imiges" | "imeges" => Some("images"),
            "jif" | "giph" | "geef" => Some("gif"),
            "terminel" | "tarminal" | "termnl" => Some("terminal"),
            "klin" | "klean" => Some("clean"),
            "chexum" | "cheksum" | "shah256" => Some("checksum"),
            "libboard" | "lipboard" | "clipbord" => Some("clipboard"),
            _ => None,
        };
        if let Some(m) = mapped {
            *word = m.to_string();
        }
    }

    // 1. Deduplicate consecutive repeated action & keyword tokens
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
                || (last == "webp" && w == "webp")
                || (last == "mp3" && w == "mp3")
                || (last == "png" && w == "png")
                || (last == "jpg" && w == "jpg")
                || (last == "pdf" && w == "pdf")
                || (last == "gif" && w == "gif")
                || (last == "terminal" && w == "terminal")
                || (last == "downloads" && w == "downloads")
                || (last == "documents" && w == "documents")
                || (last == "desktop" && w == "desktop")
                || (last == "pictures" && w == "pictures")
                || (last == "videos" && w == "videos")
                || (last == "music" && w == "music")
                || (last == "filename" && w == "filename")
                || (last == "metadata" && w == "metadata")
                || (last == "checksum" && w == "checksum")
                || (last == "timestamp" && w == "timestamp")
                || (last == "path" && w == "path")
                || (last == "folder" && w == "folder")
                || (last == "clipboard" && w == "clipboard")
            {
                continue;
            }
        }
        deduped_words.push(w);
    }

    // 2. Deduplicate repeated multi-token phrases (e.g. "full path full path", "duplicate this duplicate this")
    let mut phrase_deduped: Vec<String> = Vec::with_capacity(deduped_words.len());
    let mut i = 0;
    while i < deduped_words.len() {
        // 3-word phrase repetition: [A, B, C, A, B, C]
        if i + 5 < deduped_words.len()
            && deduped_words[i] == deduped_words[i + 3]
            && deduped_words[i + 1] == deduped_words[i + 4]
            && deduped_words[i + 2] == deduped_words[i + 5]
        {
            phrase_deduped.push(deduped_words[i].clone());
            phrase_deduped.push(deduped_words[i + 1].clone());
            phrase_deduped.push(deduped_words[i + 2].clone());
            i += 6;
            while i + 2 < deduped_words.len()
                && deduped_words[i] == phrase_deduped[phrase_deduped.len() - 3]
                && deduped_words[i + 1] == phrase_deduped[phrase_deduped.len() - 2]
                && deduped_words[i + 2] == phrase_deduped[phrase_deduped.len() - 1]
            {
                i += 3;
            }
            continue;
        }
        // 2-word phrase repetition: [A, B, A, B]
        if i + 3 < deduped_words.len()
            && deduped_words[i] == deduped_words[i + 2]
            && deduped_words[i + 1] == deduped_words[i + 3]
        {
            phrase_deduped.push(deduped_words[i].clone());
            phrase_deduped.push(deduped_words[i + 1].clone());
            i += 4;
            while i + 1 < deduped_words.len()
                && deduped_words[i] == phrase_deduped[phrase_deduped.len() - 2]
                && deduped_words[i + 1] == phrase_deduped[phrase_deduped.len() - 1]
            {
                i += 2;
            }
            continue;
        }
        phrase_deduped.push(deduped_words[i].clone());
        i += 1;
    }
    s = phrase_deduped.join(" ");

    s
}
