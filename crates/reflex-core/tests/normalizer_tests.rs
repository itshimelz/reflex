use reflex_core::normalizer::normalize_transcript;

#[test]
fn test_preamble_stripping() {
    assert_eq!(normalize_transcript("please volume up"), "volume up");
    assert_eq!(normalize_transcript("can you close window"), "close window");
    assert_eq!(normalize_transcript("could you pause"), "pause");
    assert_eq!(normalize_transcript("hey volume down"), "volume down");
    assert_eq!(normalize_transcript("now mute"), "mute");
    assert_eq!(normalize_transcript("just pause"), "pause");
    assert_eq!(normalize_transcript("take a screenshot"), "screenshot");
}

#[test]
fn test_split_word_merges() {
    assert_eq!(normalize_transcript("work space 4"), "workspace 4");
    assert_eq!(
        normalize_transcript("move to work space 6"),
        "move to workspace 6"
    );
    assert_eq!(normalize_transcript("open you tube"), "open youtube");
    assert_eq!(
        normalize_transcript("switch to whats app"),
        "switch to whatsapp"
    );
    assert_eq!(normalize_transcript("take a screen shot"), "screenshot");
    assert_eq!(normalize_transcript("screen cap this"), "screenshot this");
    assert_eq!(normalize_transcript("sound sound 50"), "sound 50");
    assert_eq!(normalize_transcript("volume volume 55"), "volume 55");
}

#[test]
fn test_homophone_replacements() {
    assert_eq!(normalize_transcript("paws music"), "pause music");
    assert_eq!(normalize_transcript("paws"), "pause");
    assert_eq!(normalize_transcript("mike mute"), "mic mute");
    assert_eq!(normalize_transcript("mute the mike"), "mute the mic");
    assert_eq!(normalize_transcript("togoli speaker"), "toggle speaker");
    assert_eq!(normalize_transcript("togol speaker"), "toggle speaker");
    assert_eq!(normalize_transcript("valum up"), "volume up");
    assert_eq!(normalize_transcript("toggle plot"), "toggle float");
    assert_eq!(normalize_transcript("toggle flot"), "toggle float");
    assert_eq!(normalize_transcript("plot window"), "float window");
    assert_eq!(normalize_transcript("flot window"), "float window");
    assert_eq!(normalize_transcript("plot it"), "float it");
    assert_eq!(normalize_transcript("unplot"), "unfloat");
    assert_eq!(normalize_transcript("Jip them"), "zip them");
    assert_eq!(normalize_transcript("Unchip all"), "unzip all");
    assert_eq!(normalize_transcript("Anjip"), "unzip");
    assert_eq!(normalize_transcript("zip, zip"), "zip");
    assert_eq!(normalize_transcript("Unjip them all"), "unzip them all");
}

#[test]
fn test_punctuation_and_whitespace_trimming() {
    assert_eq!(normalize_transcript("  volume up!  "), "volume up");
    assert_eq!(normalize_transcript("pause, please"), "pause, please"); // Preamble is prefix
    assert_eq!(normalize_transcript("workspace 4."), "workspace 4");
}

#[test]
fn test_user_reported_file_mispronunciations_and_stutters() {
    assert_eq!(normalize_transcript("lib board"), "clipboard");
    assert_eq!(normalize_transcript("save lib board here"), "save clipboard here");
    assert_eq!(normalize_transcript("Weppy, Webby, Webby, Webby"), "webp");
    assert_eq!(normalize_transcript("convert to Weppy, Webby, Webby, Webby"), "convert to webp");
    assert_eq!(normalize_transcript("full path full path"), "full path");
    assert_eq!(normalize_transcript("mp3 mp3"), "mp3");
    assert_eq!(normalize_transcript("convert to mp3 mp3"), "convert to mp3");
    assert_eq!(normalize_transcript("clean file name clean"), "clean filename");
    assert_eq!(normalize_transcript("duplicate this duplicate this"), "duplicate this");
    assert_eq!(normalize_transcript("trash this trash this"), "trash this");
    assert_eq!(normalize_transcript("copy this copy this"), "copy this");
    assert_eq!(normalize_transcript("strip metadata strip metadata"), "strip metadata");
    assert_eq!(normalize_transcript("Timestamp"), "timestamp");
    assert_eq!(normalize_transcript("metadata"), "metadata");
    assert_eq!(normalize_transcript("checksum"), "checksum");
    assert_eq!(normalize_transcript("SHA 256"), "sha256");
    assert_eq!(normalize_transcript("downloads"), "downloads");
    assert_eq!(normalize_transcript("rename"), "rename");
}
