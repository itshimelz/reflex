//! Zero-allocation lexical parser for Reflex.

use crate::ast::*;

/// Parse a natural or digit number string (0..=100) into a u8.
pub fn parse_number(input: &str) -> Option<u8> {
    let clean = input.trim().to_lowercase();
    if let Ok(n) = clean.parse::<u8>() {
        return Some(n);
    }
    match clean.as_str() {
        "zero" => Some(0),
        "one" => Some(1),
        "two" => Some(2),
        "three" => Some(3),
        "four" => Some(4),
        "five" => Some(5),
        "six" => Some(6),
        "seven" => Some(7),
        "eight" => Some(8),
        "nine" => Some(9),
        "ten" => Some(10),
        "eleven" => Some(11),
        "twelve" => Some(12),
        "thirteen" => Some(13),
        "fourteen" => Some(14),
        "fifteen" => Some(15),
        "sixteen" => Some(16),
        "seventeen" => Some(17),
        "eighteen" => Some(18),
        "nineteen" => Some(19),
        "twenty" => Some(20),
        "twenty one" | "twenty-one" => Some(21),
        "twenty two" | "twenty-two" => Some(22),
        "twenty three" | "twenty-three" => Some(23),
        "twenty four" | "twenty-four" => Some(24),
        "twenty five" | "twenty-five" => Some(25),
        "twenty six" | "twenty-six" => Some(26),
        "twenty seven" | "twenty-seven" => Some(27),
        "twenty eight" | "twenty-eight" => Some(28),
        "twenty nine" | "twenty-nine" => Some(29),
        "thirty" => Some(30),
        "thirty five" | "thirty-five" => Some(35),
        "forty" => Some(40),
        "forty five" | "forty-five" => Some(45),
        "fifty" => Some(50),
        "fifty five" | "fifty-five" => Some(55),
        "sixty" => Some(60),
        "sixty five" | "sixty-five" => Some(65),
        "seventy" => Some(70),
        "seventy five" | "seventy-five" => Some(75),
        "eighty" => Some(80),
        "eighty five" | "eighty-five" => Some(85),
        "ninety" => Some(90),
        "ninety five" | "ninety-five" => Some(95),
        "hundred" | "one hundred" => Some(100),
        // Ordinals
        "first" | "1st" => Some(1),
        "second" | "2nd" => Some(2),
        "third" | "3rd" => Some(3),
        "fourth" | "4th" => Some(4),
        "fifth" | "5th" => Some(5),
        "sixth" | "6th" => Some(6),
        "seventh" | "7th" => Some(7),
        "eighth" | "8th" => Some(8),
        "ninth" | "9th" => Some(9),
        "tenth" | "10th" => Some(10),
        _ => None,
    }
}

/// Helper to parse direction strings
fn parse_direction(input: &str) -> Option<Direction> {
    match input.trim().to_lowercase().as_str() {
        "left" => Some(Direction::Left),
        "right" => Some(Direction::Right),
        "up" | "top" => Some(Direction::Up),
        "down" | "bottom" => Some(Direction::Down),
        _ => None,
    }
}

pub fn parse_action(input: &str) -> Option<Action> {
    let clean = input.trim().to_lowercase();
    if clean.is_empty() {
        return None;
    }

    // 1. Confirmation
    if matches!(
        clean.as_str(),
        "confirm" | "confirmed" | "yes confirm" | "confirm it" | "i confirm"
    ) {
        return Some(Action::Confirm);
    }

    // 2. Fast exact matches for media
    match clean.as_str() {
        "pause"
        | "pause music"
        | "pause video"
        | "pause song"
        | "pause media"
        | "stop"
        | "stop music"
        | "stop video"
        | "stop song"
        | "stop media"
        | "stop playback"
        | "stop audio"
        | "stop playing" => return Some(Action::Media(MediaAction::Pause)),
        "play" | "resume" | "unpause" | "play music" | "play song" => {
            return Some(Action::Media(MediaAction::Play))
        }
        "toggle media" | "toggle play" | "play pause" | "toggle playback" => {
            return Some(Action::Media(MediaAction::Toggle))
        }
        "next" | "skip" | "next song" | "next track" => {
            return Some(Action::Media(MediaAction::Next))
        }
        "previous" | "prev" | "last song" | "back song" | "previous track" => {
            return Some(Action::Media(MediaAction::Previous))
        }
        "replay" | "restart song" | "restart track" => {
            return Some(Action::Media(MediaAction::Replay))
        }
        _ => {}
    }

    // Media seek
    if let Some(rest) = clean.strip_prefix("seek forward") {
        let trimmed = rest
            .trim()
            .trim_end_matches("seconds")
            .trim_end_matches("second")
            .trim();
        if let Some(n) = parse_number(trimmed) {
            return Some(Action::Media(MediaAction::SeekForward(n as u32)));
        }
    } else if let Some(rest) = clean.strip_prefix("seek backward") {
        let trimmed = rest
            .trim()
            .trim_end_matches("seconds")
            .trim_end_matches("second")
            .trim();
        if let Some(n) = parse_number(trimmed) {
            return Some(Action::Media(MediaAction::SeekBackward(n as u32)));
        }
    }

    // 3. Audio actions
    match clean.as_str() {
        "volume up" | "sound up" | "louder" | "turn it up" | "turn sound up" | "turn volume up" => {
            return Some(Action::Audio(AudioAction::VolumeStep(5)))
        }
        "volume down" | "sound down" | "quieter" | "softer" | "turn it down" | "turn sound down" | "turn volume down" => {
            return Some(Action::Audio(AudioAction::VolumeStep(-5)))
        }
        "mute" | "mute audio" | "mute sound" | "silence" => return Some(Action::Audio(AudioAction::Mute(true))),
        "unmute" | "unmute audio" | "unmute sound" => return Some(Action::Audio(AudioAction::Mute(false))),
        "toggle mute" | "toggle sound" => return Some(Action::Audio(AudioAction::MuteToggle)),
        "toggle mic" | "mute mic" | "unmute mic" => {
            return Some(Action::Audio(AudioAction::MicMuteToggle))
        }
        "switch audio"
        | "switch speaker"
        | "switch speakers"
        | "toggle speaker"
        | "toggle speakers"
        | "toggle audio"
        | "change audio"
        | "change speaker"
        | "swap audio"
        | "swap speaker"
        | "toggle output"
        | "switch output"
        | "switch to headphones"
        | "switch to speaker"
        | "switch to speakers"
        | "switch headphones"
        | "headphones"
        | "speaker" => return Some(Action::Audio(AudioAction::SwitchPort)),
        _ => {}
    }

    if let Some(rest) = clean
        .strip_prefix("volume up")
        .or_else(|| clean.strip_prefix("sound up"))
    {
        let trimmed = rest
            .trim()
            .trim_start_matches("by")
            .trim()
            .trim_end_matches('%')
            .trim();
        if let Some(n) = parse_number(trimmed) {
            return Some(Action::Audio(AudioAction::VolumeStep(n as i8)));
        }
    }
    if let Some(rest) = clean
        .strip_prefix("volume down")
        .or_else(|| clean.strip_prefix("sound down"))
    {
        let trimmed = rest
            .trim()
            .trim_start_matches("by")
            .trim()
            .trim_end_matches('%')
            .trim();
        if let Some(n) = parse_number(trimmed) {
            return Some(Action::Audio(AudioAction::VolumeStep(-(n as i8))));
        }
    }
    for prefix in &[
        "set volume",
        "set sound",
        "volume volume",
        "sound sound",
        "volume",
        "sound",
    ] {
        if let Some(rest) = clean.strip_prefix(prefix) {
            let trimmed = rest
                .trim()
                .trim_start_matches("to")
                .trim()
                .trim_end_matches('%')
                .trim();
            if let Some(n) = parse_number(trimmed) {
                return Some(Action::Audio(AudioAction::VolumeSet(n)));
            }
        }
    }

    // App audio muting: "mute <app>", "unmute <app>", "silence <app>", "toggle mute <app>"
    if let Some(rest) = clean.strip_prefix("mute ") {
        let app = rest.trim();
        if !app.is_empty() && app != "audio" && app != "mic" {
            return Some(Action::Audio(AudioAction::AppAudio {
                target: app.to_string(),
                mode: AppAudioMode::Mute(true),
            }));
        }
    }
    if let Some(rest) = clean.strip_prefix("unmute ") {
        let app = rest.trim();
        if !app.is_empty() && app != "audio" && app != "mic" {
            return Some(Action::Audio(AudioAction::AppAudio {
                target: app.to_string(),
                mode: AppAudioMode::Mute(false),
            }));
        }
    }
    if let Some(rest) = clean.strip_prefix("silence ") {
        let app = rest.trim();
        if !app.is_empty() && app != "audio" {
            return Some(Action::Audio(AudioAction::AppAudio {
                target: app.to_string(),
                mode: AppAudioMode::Mute(true),
            }));
        }
    }
    if let Some(rest) = clean.strip_prefix("toggle mute ") {
        let app = rest.trim();
        if !app.is_empty() {
            return Some(Action::Audio(AudioAction::AppAudio {
                target: app.to_string(),
                mode: AppAudioMode::ToggleMute,
            }));
        }
    }

    // App volume: "set <app> volume <num>", "<app> volume <num>", "volume <app> <num>"
    if let Some(rest) = clean.strip_prefix("set ") {
        if let Some(idx) = rest.find(" volume") {
            let app = rest[..idx].trim();
            let after = rest[idx + 7..]
                .trim()
                .trim_start_matches("to")
                .trim()
                .trim_end_matches('%')
                .trim();
            if let Some(n) = parse_number(after) {
                if !app.is_empty() {
                    return Some(Action::Audio(AudioAction::AppAudio {
                        target: app.to_string(),
                        mode: AppAudioMode::VolumeSet(n),
                    }));
                }
            }
        }
    }

    if let Some(idx) = clean.find(" volume ") {
        let app = clean[..idx].trim();
        let after = clean[idx + 8..].trim();
        if !app.is_empty() && app != "set" {
            if after == "up" {
                return Some(Action::Audio(AudioAction::AppAudio {
                    target: app.to_string(),
                    mode: AppAudioMode::VolumeStep(5),
                }));
            } else if after == "down" {
                return Some(Action::Audio(AudioAction::AppAudio {
                    target: app.to_string(),
                    mode: AppAudioMode::VolumeStep(-5),
                }));
            } else {
                let num_str = after
                    .trim_start_matches("to")
                    .trim()
                    .trim_end_matches('%')
                    .trim();
                if let Some(n) = parse_number(num_str) {
                    return Some(Action::Audio(AudioAction::AppAudio {
                        target: app.to_string(),
                        mode: AppAudioMode::VolumeSet(n),
                    }));
                }
            }
        }
    }

    if let Some(rest) = clean.strip_prefix("volume ") {
        let parts: Vec<&str> = rest.split_whitespace().collect();
        if parts.len() >= 2 {
            let last = parts.last().unwrap();
            let num_str = last.trim_end_matches('%');
            if let Some(n) = parse_number(num_str) {
                let app_parts = &parts[..parts.len() - 1];
                let app_str = app_parts.join(" ");
                let clean_app = app_str.trim_end_matches("to").trim();
                if !clean_app.is_empty() && clean_app != "up" && clean_app != "down" {
                    return Some(Action::Audio(AudioAction::AppAudio {
                        target: clean_app.to_string(),
                        mode: AppAudioMode::VolumeSet(n),
                    }));
                }
            }
        }
    }

    if let Some(rest) = clean.strip_prefix("turn up ") {
        let app = rest.trim();
        if !app.is_empty() && app != "volume" && app != "it" {
            return Some(Action::Audio(AudioAction::AppAudio {
                target: app.to_string(),
                mode: AppAudioMode::VolumeStep(5),
            }));
        }
    }
    if let Some(rest) = clean.strip_prefix("turn down ") {
        let app = rest.trim();
        if !app.is_empty() && app != "volume" && app != "it" {
            return Some(Action::Audio(AudioAction::AppAudio {
                target: app.to_string(),
                mode: AppAudioMode::VolumeStep(-5),
            }));
        }
    }

    // 4. Window actions
    match clean.as_str() {
        "close"
        | "close it"
        | "close this"
        | "close window"
        | "close the window"
        | "close this window"
        | "close active"
        | "close active window"
        | "close current"
        | "close current window"
        | "kill"
        | "kill it"
        | "kill this"
        | "kill window"
        | "kill the window"
        | "kill active"
        | "kill current"
        | "quit window" => {
            return Some(Action::Window(WindowAction::CloseActive));
        }
        "fullscreen" | "toggle fullscreen" => {
            return Some(Action::Window(WindowAction::ToggleFullscreen));
        }
        "float"
        | "float window"
        | "float it"
        | "float this"
        | "unfloat"
        | "unfloat window"
        | "unfloat it"
        | "toggle float"
        | "toggle floating"
        | "flot"
        | "flot window"
        | "flot it"
        | "flot this"
        | "unflot"
        | "unflot window"
        | "unflot it"
        | "toggle flot"
        | "toggle floting"
        | "plot"
        | "plot window"
        | "plot it"
        | "plot this"
        | "unplot"
        | "unplot window"
        | "unplot it"
        | "toggle plot"
        | "toggle plotting"
        | "toggle ploting" => return Some(Action::Window(WindowAction::ToggleFloating)),
        "pin"
        | "pin window"
        | "pin it"
        | "pin this"
        | "unpin"
        | "unpin window"
        | "unpin it"
        | "toggle pin" => return Some(Action::Window(WindowAction::TogglePin)),
        "center"
        | "center window"
        | "center it"
        | "center this"
        | "center active"
        | "center active window" => return Some(Action::Window(WindowAction::CenterWindow)),
        "split"
        | "toggle split"
        | "change split"
        | "switch split"
        | "swap split" => return Some(Action::Window(WindowAction::ToggleSplit)),
        _ => {}
    }

    if let Some(rest) = clean.strip_prefix("focus window") {
        if let Some(dir) = parse_direction(rest) {
            return Some(Action::Window(WindowAction::FocusDirection(dir)));
        }
    }
    if let Some(rest) = clean.strip_prefix("focus") {
        if let Some(dir) = parse_direction(rest) {
            return Some(Action::Window(WindowAction::FocusDirection(dir)));
        }
    }
    if let Some(rest) = clean.strip_prefix("move window") {
        if let Some(dir) = parse_direction(rest) {
            return Some(Action::Window(WindowAction::MoveDirection(dir)));
        }
    }
    if let Some(rest) = clean.strip_prefix("move") {
        if let Some(dir) = parse_direction(rest) {
            return Some(Action::Window(WindowAction::MoveDirection(dir)));
        }
    }
    if let Some(rest) = clean.strip_prefix("swap window") {
        if let Some(dir) = parse_direction(rest) {
            return Some(Action::Window(WindowAction::MoveDirection(dir)));
        }
    }
    if let Some(rest) = clean.strip_prefix("swap") {
        if let Some(dir) = parse_direction(rest) {
            return Some(Action::Window(WindowAction::MoveDirection(dir)));
        }
    }

    // 5. Screenshot actions
    match clean.as_str() {
        "screenshot" | "screenshot this" | "screenshot window" | "screenshot this window" => {
            return Some(Action::Screenshot(ScreenshotAction::ActiveWindow));
        }
        "screenshot screen" | "capture screen" | "take screenshot" | "full screenshot" => {
            return Some(Action::Screenshot(ScreenshotAction::FullScreen));
        }
        "recapture" | "screenshot again" => {
            return Some(Action::Screenshot(ScreenshotAction::RecaptureLast));
        }
        "snip" | "custom selection" | "select area" => {
            return Some(Action::Screenshot(ScreenshotAction::InteractiveSnip));
        }
        _ => {}
    }

    if let Some(rest) = clean.strip_prefix("screenshot pane") {
        if let Some(dir) = parse_direction(rest) {
            return Some(Action::Screenshot(ScreenshotAction::DirectionalPane(dir)));
        }
    }

    // 6. Clipboard actions
    match clean.as_str() {
        "type clipboard"
        | "type it"
        | "type that"
        | "type buffer"
        | "paste typing"
        | "paste as typing"
        | "type" => return Some(Action::Clipboard(ClipboardAction::TypeClipboard)),
        "clear clipboard"
        | "empty clipboard"
        | "clear buffer" => return Some(Action::Clipboard(ClipboardAction::ClearClipboard)),
        "strip clipboard"
        | "strip clipboard formatting"
        | "clean clipboard"
        | "plain clipboard" => return Some(Action::Clipboard(ClipboardAction::StripFormatting)),
        _ => {}
    }

    // 6.5 GUI & Computer-Use Actions (Scroll, Focus, Click, Press Key, Set Value)
    if clean == "scroll" || clean == "scroll down" || clean == "page down" {
        return Some(Action::Gui(GuiAction::Scroll {
            direction: GuiScrollDirection::Down,
            amount: 4,
        }));
    }
    if clean == "scroll up" || clean == "page up" {
        return Some(Action::Gui(GuiAction::Scroll {
            direction: GuiScrollDirection::Up,
            amount: 4,
        }));
    }
    if clean == "scroll left" {
        return Some(Action::Gui(GuiAction::Scroll {
            direction: GuiScrollDirection::Left,
            amount: 4,
        }));
    }
    if clean == "scroll right" {
        return Some(Action::Gui(GuiAction::Scroll {
            direction: GuiScrollDirection::Right,
            amount: 4,
        }));
    }

    if let Some(rest) = clean.strip_prefix("scroll ") {
        let trimmed = rest.trim();
        let (dir, remainder) = if let Some(r) = trimmed.strip_prefix("down") {
            (GuiScrollDirection::Down, r.trim())
        } else if let Some(r) = trimmed.strip_prefix("up") {
            (GuiScrollDirection::Up, r.trim())
        } else if let Some(r) = trimmed.strip_prefix("left") {
            (GuiScrollDirection::Left, r.trim())
        } else if let Some(r) = trimmed.strip_prefix("right") {
            (GuiScrollDirection::Right, r.trim())
        } else {
            (GuiScrollDirection::Down, trimmed)
        };

        let clean_num_str = remainder
            .trim_start_matches("by")
            .trim()
            .trim_end_matches("lines")
            .trim_end_matches("pages")
            .trim();
        let amount = if clean_num_str.is_empty() {
            4
        } else {
            parse_number(clean_num_str).unwrap_or(4) as u32
        };

        return Some(Action::Gui(GuiAction::Scroll {
            direction: dir,
            amount,
        }));
    }

    // Click / Tap
    if let Some(rest) = clean.strip_prefix("click ") {
        let target = rest.trim().trim_start_matches("on ").trim();
        if !target.is_empty() {
            return Some(Action::Gui(GuiAction::Click {
                target: target.to_string(),
            }));
        }
    }
    if let Some(rest) = clean.strip_prefix("tap ") {
        let target = rest.trim().trim_start_matches("on ").trim();
        if !target.is_empty() {
            return Some(Action::Gui(GuiAction::Click {
                target: target.to_string(),
            }));
        }
    }

    // Focus input/field
    if let Some(rest) = clean
        .strip_prefix("focus input ")
        .or_else(|| clean.strip_prefix("focus field "))
        .or_else(|| clean.strip_prefix("focus box "))
        .or_else(|| clean.strip_prefix("focus element "))
    {
        let target = rest.trim();
        if !target.is_empty() {
            return Some(Action::Gui(GuiAction::Focus {
                target: target.to_string(),
            }));
        }
    }
    if clean == "focus search"
        || clean == "focus search bar"
        || clean == "focus search box"
        || clean == "focus email"
        || clean == "focus username"
        || clean == "focus password"
        || clean == "focus comment"
    {
        let target = clean.trim_start_matches("focus ").trim();
        return Some(Action::Gui(GuiAction::Focus {
            target: target.to_string(),
        }));
    }

    // One-shot Search: "search for <query>", "search <query>"
    if let Some(rest) = clean.strip_prefix("search for ") {
        let q = rest.trim();
        if !q.is_empty() {
            return Some(Action::Gui(GuiAction::Search {
                query: q.to_string(),
            }));
        }
    } else if let Some(rest) = clean.strip_prefix("search ") {
        let q = rest.trim();
        if !q.is_empty() && q != "bar" && q != "box" && q != "input" {
            return Some(Action::Gui(GuiAction::Search {
                query: q.to_string(),
            }));
        }
    }

    // Press key
    if let Some(rest) = clean
        .strip_prefix("press key ")
        .or_else(|| clean.strip_prefix("send key "))
    {
        let key = rest.trim();
        if !key.is_empty() {
            return Some(Action::Gui(GuiAction::PressKey {
                combo: key.to_string(),
            }));
        }
    }
    if let Some(rest) = clean.strip_prefix("press ") {
        let key = rest.trim();
        if matches!(
            key,
            "enter"
                | "return"
                | "escape"
                | "esc"
                | "tab"
                | "backspace"
                | "space"
                | "delete"
                | "up"
                | "down"
                | "left"
                | "right"
        ) {
            return Some(Action::Gui(GuiAction::PressKey {
                combo: key.to_string(),
            }));
        }
    }

    // Set value / type into focused target
    if let Some(rest) = clean.strip_prefix("type ") {
        let text = rest.trim();
        if !text.is_empty()
            && text != "clipboard"
            && text != "it"
            && text != "that"
            && text != "buffer"
        {
            return Some(Action::Gui(GuiAction::SetValue {
                target: "focused".to_string(),
                value: text.to_string(),
            }));
        }
    }

    // File / Archive actions: zip, unzip, extract
    match clean.as_str() {
        "zip"
        | "zip selected"
        | "zip selected file"
        | "zip selected files"
        | "zip this"
        | "zip this file"
        | "zip this folder"
        | "zip it"
        | "zip them"
        | "zip all"
        | "zip them all"
        | "zip all of them"
        | "zip these"
        | "zip these files"
        | "zip file"
        | "zip files"
        | "compress"
        | "compress selected"
        | "compress this"
        | "compress this file"
        | "compress this folder"
        | "compress it"
        | "compress them"
        | "compress all"
        | "compress them all"
        | "compress all of them"
        | "compress these"
        | "compress file"
        | "compress files"
        // Common ASR mispronunciations without normalizer:
        | "jip"
        | "jip them"
        | "jip all"
        | "jip them all"
        | "jip this"
        | "jip it"
        | "jip selected" => return Some(Action::File(FileAction::ZipSelected)),

        "unzip"
        | "unzip selected"
        | "unzip selected file"
        | "unzip selected files"
        | "unzip this"
        | "unzip this file"
        | "unzip it"
        | "unzip them"
        | "unzip all"
        | "unzip them all"
        | "unzip all of them"
        | "unzip these"
        | "unzip these files"
        | "unzip file"
        | "unzip files"
        | "unzip archive"
        | "extract"
        | "extract selected"
        | "extract selected file"
        | "extract selected files"
        | "extract this"
        | "extract this file"
        | "extract it"
        | "extract them"
        | "extract all"
        | "extract them all"
        | "extract all of them"
        | "extract these"
        | "extract file"
        | "extract files"
        | "extract archive"
        // Common ASR mispronunciations without normalizer:
        | "unjip"
        | "unjip them"
        | "unjip all"
        | "unjip them all"
        | "unjip this"
        | "unjip it"
        | "anjip"
        | "anjip them"
        | "anjip all"
        | "anjip them all"
        | "unchip"
        | "unchip them"
        | "unchip all"
        | "unchip them all" => {
            return Some(Action::File(FileAction::UnzipSelected {
                into_subfolder: true,
            }))
        }

        "extract here"
        | "unzip here"
        | "extract in place"
        | "unzip in place"
        | "unjip here"
        | "unchip here"
        | "anjip here" => {
            return Some(Action::File(FileAction::UnzipSelected {
                into_subfolder: false,
            }))
        }
        _ => {}
    }

    // 7. Workspace focus: "workspace 3", "go to workspace 5", "switch to 4"
    let ws_prefixes = [
        "workspace",
        "go to workspace",
        "switch to workspace",
        "go to",
        "switch to",
    ];
    for p in ws_prefixes {
        if let Some(rest) = clean.strip_prefix(p) {
            let trimmed = rest.trim();
            if let Some(n) = parse_number(trimmed) {
                return Some(Action::Workspace(WorkspaceAction::Focus(n)));
            }
        }
    }

    // 7. Workspace move: "move window to workspace 6", "move this window workspace 6", "move to 6 and follow", "move Chrome to 4"
    if let Some(rest) = clean.strip_prefix("move") {
        let trimmed = rest.trim();
        let follow = trimmed.ends_with("and follow") || trimmed.ends_with("follow");
        let core_rest = if follow {
            trimmed
                .trim_end_matches("and follow")
                .trim_end_matches("follow")
                .trim()
        } else {
            trimmed
        };

        // Case A: "window to workspace 6", "this window workspace 6", "to 6"
        let window_prefixes = [
            "this window to workspace",
            "window to workspace",
            "this window workspace",
            "window workspace",
            "this to workspace",
            "to workspace",
            "this window to",
            "window to",
            "this window",
            "to",
        ];
        let mut matched_active = false;
        for wp in window_prefixes {
            if let Some(after) = core_rest.strip_prefix(wp) {
                let num_str = after.trim().trim_start_matches("workspace").trim();
                if let Some(n) = parse_number(num_str) {
                    return Some(Action::Workspace(WorkspaceAction::MoveWindow {
                        workspace: n,
                        follow,
                    }));
                }
                matched_active = true;
                break;
            }
        }

        if !matched_active {
            // Case B: Named app: "move <app> to workspace 6", "move <app> to 4", "move <app> 4"
            // Split by "to workspace", "to", "workspace", or last token if number
            let parts: Vec<&str> = core_rest.split_whitespace().collect();
            if parts.len() >= 2 {
                // Check if last token is number
                if let Some(last) = parts.last() {
                    if let Some(ws) = parse_number(last) {
                        let app_tokens = &parts[..parts.len() - 1];
                        let app_str = app_tokens.join(" ");
                        let clean_app = app_str
                            .trim_end_matches("to workspace")
                            .trim_end_matches("workspace")
                            .trim_end_matches("to")
                            .trim();
                        if !clean_app.is_empty() {
                            return Some(Action::MoveNamed {
                                app: clean_app.to_string(),
                                workspace: ws,
                                follow,
                            });
                        }
                    }
                }
            }
        }
    }

    // 8. Open / Launch / Focus / Close app
    let clean_app = |raw: &str| -> String {
        let mut s = raw.trim();
        if let Some(r) = s.strip_prefix("up ") {
            s = r.trim();
        }
        if let Some(r) = s.strip_prefix("the ") {
            s = r.trim();
        }
        s.to_string()
    };

    if let Some(rest) = clean.strip_prefix("open") {
        let app = clean_app(rest);
        if !app.is_empty() && app != "window" && app != "active" {
            return Some(Action::Open(app));
        }
    }
    if let Some(rest) = clean.strip_prefix("launch") {
        let app = clean_app(rest);
        if !app.is_empty() && app != "window" && app != "active" {
            return Some(Action::Open(app));
        }
    }
    if let Some(rest) = clean.strip_prefix("focus") {
        let app = clean_app(rest);
        if !app.is_empty() && app != "window" && app != "active" {
            return Some(Action::Open(app));
        }
    }
    if let Some(rest) = clean.strip_prefix("switch to") {
        let app = clean_app(rest);
        if !app.is_empty()
            && app != "window"
            && app != "active"
            && app != "speaker"
            && app != "speakers"
            && app != "headphones"
        {
            return Some(Action::Open(app));
        }
    }
    if let Some(rest) = clean.strip_prefix("go to") {
        let app = clean_app(rest);
        if !app.is_empty() && app != "window" && app != "active" {
            return Some(Action::Open(app));
        }
    }
    if let Some(rest) = clean.strip_prefix("goto") {
        let app = clean_app(rest);
        if !app.is_empty() && app != "window" && app != "active" {
            return Some(Action::Open(app));
        }
    }
    let is_generic_close_target = |t: &str| -> bool {
        matches!(
            t,
            "" | "it"
                | "this"
                | "window"
                | "the window"
                | "this window"
                | "active"
                | "active window"
                | "current"
                | "current window"
        )
    };

    if let Some(rest) = clean.strip_prefix("close") {
        let app = clean_app(rest);
        if is_generic_close_target(&app) {
            return Some(Action::Window(WindowAction::CloseActive));
        } else {
            return Some(Action::Close(app));
        }
    }
    if let Some(rest) = clean.strip_prefix("kill") {
        let app = clean_app(rest);
        if is_generic_close_target(&app) {
            return Some(Action::Window(WindowAction::CloseActive));
        } else {
            return Some(Action::Close(app));
        }
    }

    None
}
