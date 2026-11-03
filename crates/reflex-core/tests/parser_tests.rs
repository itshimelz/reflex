use reflex_core::ast::*;
use reflex_core::parser::parse_action;

#[test]
fn test_media_commands() {
    assert_eq!(
        parse_action("pause"),
        Some(Action::Media(MediaAction::Pause))
    );
    assert_eq!(
        parse_action("pause music"),
        Some(Action::Media(MediaAction::Pause))
    );
    assert_eq!(parse_action("stop"), Some(Action::Media(MediaAction::Pause)));
    assert_eq!(
        parse_action("stop music"),
        Some(Action::Media(MediaAction::Pause))
    );
    assert_eq!(
        parse_action("stop video"),
        Some(Action::Media(MediaAction::Pause))
    );
    assert_eq!(
        parse_action("stop playback"),
        Some(Action::Media(MediaAction::Pause))
    );
    assert_eq!(parse_action("play"), Some(Action::Media(MediaAction::Play)));
    assert_eq!(
        parse_action("resume"),
        Some(Action::Media(MediaAction::Play))
    );
    assert_eq!(
        parse_action("toggle media"),
        Some(Action::Media(MediaAction::Toggle))
    );
    assert_eq!(
        parse_action("next song"),
        Some(Action::Media(MediaAction::Next))
    );
    assert_eq!(
        parse_action("previous track"),
        Some(Action::Media(MediaAction::Previous))
    );
    assert_eq!(
        parse_action("prev"),
        Some(Action::Media(MediaAction::Previous))
    );
    assert_eq!(
        parse_action("replay"),
        Some(Action::Media(MediaAction::Replay))
    );
    assert_eq!(
        parse_action("seek forward 10"),
        Some(Action::Media(MediaAction::SeekForward(10)))
    );
    assert_eq!(
        parse_action("seek backward 15 seconds"),
        Some(Action::Media(MediaAction::SeekBackward(15)))
    );
}

#[test]
fn test_audio_commands() {
    assert_eq!(
        parse_action("volume up"),
        Some(Action::Audio(AudioAction::VolumeStep(5)))
    );
    assert_eq!(
        parse_action("volume down"),
        Some(Action::Audio(AudioAction::VolumeStep(-5)))
    );
    assert_eq!(
        parse_action("volume up 10"),
        Some(Action::Audio(AudioAction::VolumeStep(10)))
    );
    assert_eq!(
        parse_action("volume down 10"),
        Some(Action::Audio(AudioAction::VolumeStep(-10)))
    );
    assert_eq!(
        parse_action("louder"),
        Some(Action::Audio(AudioAction::VolumeStep(5)))
    );
    assert_eq!(
        parse_action("quieter"),
        Some(Action::Audio(AudioAction::VolumeStep(-5)))
    );
    assert_eq!(
        parse_action("set volume to 40%"),
        Some(Action::Audio(AudioAction::VolumeSet(40)))
    );
    assert_eq!(
        parse_action("volume 50"),
        Some(Action::Audio(AudioAction::VolumeSet(50)))
    );
    assert_eq!(
        parse_action("set volume seventy"),
        Some(Action::Audio(AudioAction::VolumeSet(70)))
    );
    assert_eq!(
        parse_action("set volume seventy five"),
        Some(Action::Audio(AudioAction::VolumeSet(75)))
    );
    assert_eq!(
        parse_action("sound 50"),
        Some(Action::Audio(AudioAction::VolumeSet(50)))
    );
    assert_eq!(
        parse_action("sound 55"),
        Some(Action::Audio(AudioAction::VolumeSet(55)))
    );
    assert_eq!(
        parse_action("sound sound 50"),
        Some(Action::Audio(AudioAction::VolumeSet(50)))
    );
    assert_eq!(
        parse_action("sound sound 55"),
        Some(Action::Audio(AudioAction::VolumeSet(55)))
    );
    assert_eq!(
        parse_action("set sound to 50"),
        Some(Action::Audio(AudioAction::VolumeSet(50)))
    );
    assert_eq!(
        parse_action("sound to 55"),
        Some(Action::Audio(AudioAction::VolumeSet(55)))
    );
    assert_eq!(
        parse_action("sound up"),
        Some(Action::Audio(AudioAction::VolumeStep(5)))
    );
    assert_eq!(
        parse_action("sound down"),
        Some(Action::Audio(AudioAction::VolumeStep(-5)))
    );
    assert_eq!(
        parse_action("mute"),
        Some(Action::Audio(AudioAction::Mute(true)))
    );
    assert_eq!(
        parse_action("unmute"),
        Some(Action::Audio(AudioAction::Mute(false)))
    );
    assert_eq!(
        parse_action("toggle mute"),
        Some(Action::Audio(AudioAction::MuteToggle))
    );
    assert_eq!(
        parse_action("toggle mic"),
        Some(Action::Audio(AudioAction::MicMuteToggle))
    );
    assert_eq!(
        parse_action("switch speaker"),
        Some(Action::Audio(AudioAction::SwitchPort))
    );
    assert_eq!(
        parse_action("toggle speaker"),
        Some(Action::Audio(AudioAction::SwitchPort))
    );
    assert_eq!(
        parse_action("switch audio"),
        Some(Action::Audio(AudioAction::SwitchPort))
    );
    assert_eq!(
        parse_action("toggle audio"),
        Some(Action::Audio(AudioAction::SwitchPort))
    );
    assert_eq!(
        parse_action("headphones"),
        Some(Action::Audio(AudioAction::SwitchPort))
    );
    assert_eq!(
        parse_action("mute brave"),
        Some(Action::Audio(AudioAction::AppAudio {
            target: "brave".to_string(),
            mode: AppAudioMode::Mute(true)
        }))
    );
    assert_eq!(
        parse_action("unmute spotify"),
        Some(Action::Audio(AudioAction::AppAudio {
            target: "spotify".to_string(),
            mode: AppAudioMode::Mute(false)
        }))
    );
    assert_eq!(
        parse_action("silence discord"),
        Some(Action::Audio(AudioAction::AppAudio {
            target: "discord".to_string(),
            mode: AppAudioMode::Mute(true)
        }))
    );
    assert_eq!(
        parse_action("toggle mute brave"),
        Some(Action::Audio(AudioAction::AppAudio {
            target: "brave".to_string(),
            mode: AppAudioMode::ToggleMute
        }))
    );
    assert_eq!(
        parse_action("set brave volume 50"),
        Some(Action::Audio(AudioAction::AppAudio {
            target: "brave".to_string(),
            mode: AppAudioMode::VolumeSet(50)
        }))
    );
    assert_eq!(
        parse_action("brave volume 70"),
        Some(Action::Audio(AudioAction::AppAudio {
            target: "brave".to_string(),
            mode: AppAudioMode::VolumeSet(70)
        }))
    );
    assert_eq!(
        parse_action("brave volume up"),
        Some(Action::Audio(AudioAction::AppAudio {
            target: "brave".to_string(),
            mode: AppAudioMode::VolumeStep(5)
        }))
    );
    assert_eq!(
        parse_action("brave volume down"),
        Some(Action::Audio(AudioAction::AppAudio {
            target: "brave".to_string(),
            mode: AppAudioMode::VolumeStep(-5)
        }))
    );
    assert_eq!(
        parse_action("turn up brave"),
        Some(Action::Audio(AudioAction::AppAudio {
            target: "brave".to_string(),
            mode: AppAudioMode::VolumeStep(5)
        }))
    );
    assert_eq!(
        parse_action("turn down brave"),
        Some(Action::Audio(AudioAction::AppAudio {
            target: "brave".to_string(),
            mode: AppAudioMode::VolumeStep(-5)
        }))
    );
}

#[test]
fn test_workspace_commands() {
    assert_eq!(
        parse_action("workspace 3"),
        Some(Action::Workspace(WorkspaceAction::Focus(3)))
    );
    assert_eq!(
        parse_action("go to workspace 5"),
        Some(Action::Workspace(WorkspaceAction::Focus(5)))
    );
    assert_eq!(
        parse_action("switch to 4"),
        Some(Action::Workspace(WorkspaceAction::Focus(4)))
    );
    assert_eq!(
        parse_action("workspace two"),
        Some(Action::Workspace(WorkspaceAction::Focus(2)))
    );
    assert_eq!(
        parse_action("workspace second"),
        Some(Action::Workspace(WorkspaceAction::Focus(2)))
    );

    assert_eq!(
        parse_action("move window to workspace 6"),
        Some(Action::Workspace(WorkspaceAction::MoveWindow {
            workspace: 6,
            follow: false
        }))
    );
    assert_eq!(
        parse_action("move this window workspace 6"),
        Some(Action::Workspace(WorkspaceAction::MoveWindow {
            workspace: 6,
            follow: false
        }))
    );
    assert_eq!(
        parse_action("move to 6 and follow"),
        Some(Action::Workspace(WorkspaceAction::MoveWindow {
            workspace: 6,
            follow: true
        }))
    );
    assert_eq!(
        parse_action("move youtube to 4"),
        Some(Action::MoveNamed {
            app: "youtube".to_string(),
            workspace: 4,
            follow: false
        })
    );
    assert_eq!(
        parse_action("move chrome to workspace 6 and follow"),
        Some(Action::MoveNamed {
            app: "chrome".to_string(),
            workspace: 6,
            follow: true
        })
    );
}

#[test]
fn test_window_commands() {
    assert_eq!(
        parse_action("close"),
        Some(Action::Window(WindowAction::CloseActive))
    );
    assert_eq!(
        parse_action("close it"),
        Some(Action::Window(WindowAction::CloseActive))
    );
    assert_eq!(
        parse_action("close this"),
        Some(Action::Window(WindowAction::CloseActive))
    );
    assert_eq!(
        parse_action("close window"),
        Some(Action::Window(WindowAction::CloseActive))
    );
    assert_eq!(
        parse_action("close the window"),
        Some(Action::Window(WindowAction::CloseActive))
    );
    assert_eq!(
        parse_action("kill it"),
        Some(Action::Window(WindowAction::CloseActive))
    );
    assert_eq!(
        parse_action("kill this"),
        Some(Action::Window(WindowAction::CloseActive))
    );
    assert_eq!(
        parse_action("kill window"),
        Some(Action::Window(WindowAction::CloseActive))
    );
    assert_eq!(
        parse_action("close active"),
        Some(Action::Window(WindowAction::CloseActive))
    );
    assert_eq!(
        parse_action("fullscreen"),
        Some(Action::Window(WindowAction::ToggleFullscreen))
    );
    assert_eq!(
        parse_action("toggle fullscreen"),
        Some(Action::Window(WindowAction::ToggleFullscreen))
    );
    assert_eq!(
        parse_action("focus left"),
        Some(Action::Window(WindowAction::FocusDirection(
            Direction::Left
        )))
    );
    assert_eq!(
        parse_action("focus right"),
        Some(Action::Window(WindowAction::FocusDirection(
            Direction::Right
        )))
    );
    assert_eq!(
        parse_action("focus top"),
        Some(Action::Window(WindowAction::FocusDirection(Direction::Up)))
    );
    assert_eq!(
        parse_action("focus down"),
        Some(Action::Window(WindowAction::FocusDirection(
            Direction::Down
        )))
    );
    assert_eq!(
        parse_action("focus bottom"),
        Some(Action::Window(WindowAction::FocusDirection(
            Direction::Down
        )))
    );
    assert_eq!(
        parse_action("move window left"),
        Some(Action::Window(WindowAction::MoveDirection(Direction::Left)))
    );
    assert_eq!(
        parse_action("move window up"),
        Some(Action::Window(WindowAction::MoveDirection(Direction::Up)))
    );
    assert_eq!(
        parse_action("move window top"),
        Some(Action::Window(WindowAction::MoveDirection(Direction::Up)))
    );
    assert_eq!(
        parse_action("move left"),
        Some(Action::Window(WindowAction::MoveDirection(Direction::Left)))
    );
    assert_eq!(
        parse_action("move right"),
        Some(Action::Window(WindowAction::MoveDirection(
            Direction::Right
        )))
    );
    assert_eq!(
        parse_action("move top"),
        Some(Action::Window(WindowAction::MoveDirection(Direction::Up)))
    );
    assert_eq!(
        parse_action("move down"),
        Some(Action::Window(WindowAction::MoveDirection(Direction::Down)))
    );
    assert_eq!(
        parse_action("move bottom"),
        Some(Action::Window(WindowAction::MoveDirection(Direction::Down)))
    );
    assert_eq!(
        parse_action("swap left"),
        Some(Action::Window(WindowAction::MoveDirection(Direction::Left)))
    );
    assert_eq!(
        parse_action("swap right"),
        Some(Action::Window(WindowAction::MoveDirection(
            Direction::Right
        )))
    );
    assert_eq!(
        parse_action("swap top"),
        Some(Action::Window(WindowAction::MoveDirection(Direction::Up)))
    );
    assert_eq!(
        parse_action("swap down"),
        Some(Action::Window(WindowAction::MoveDirection(Direction::Down)))
    );
    assert_eq!(
        parse_action("toggle floating"),
        Some(Action::Window(WindowAction::ToggleFloating))
    );
    assert_eq!(
        parse_action("float window"),
        Some(Action::Window(WindowAction::ToggleFloating))
    );
    assert_eq!(
        parse_action("float it"),
        Some(Action::Window(WindowAction::ToggleFloating))
    );
    assert_eq!(
        parse_action("toggle plot"),
        Some(Action::Window(WindowAction::ToggleFloating))
    );
    assert_eq!(
        parse_action("toggle flot"),
        Some(Action::Window(WindowAction::ToggleFloating))
    );
    assert_eq!(
        parse_action("plot it"),
        Some(Action::Window(WindowAction::ToggleFloating))
    );
    assert_eq!(
        parse_action("flot window"),
        Some(Action::Window(WindowAction::ToggleFloating))
    );
    assert_eq!(
        parse_action("plot"),
        Some(Action::Window(WindowAction::ToggleFloating))
    );
    assert_eq!(
        parse_action("flot"),
        Some(Action::Window(WindowAction::ToggleFloating))
    );
    assert_eq!(
        parse_action("pin window"),
        Some(Action::Window(WindowAction::TogglePin))
    );
    assert_eq!(
        parse_action("pin it"),
        Some(Action::Window(WindowAction::TogglePin))
    );
    assert_eq!(
        parse_action("toggle pin"),
        Some(Action::Window(WindowAction::TogglePin))
    );
    assert_eq!(
        parse_action("center"),
        Some(Action::Window(WindowAction::CenterWindow))
    );
    assert_eq!(
        parse_action("center window"),
        Some(Action::Window(WindowAction::CenterWindow))
    );
    assert_eq!(
        parse_action("toggle split"),
        Some(Action::Window(WindowAction::ToggleSplit))
    );
    assert_eq!(
        parse_action("change split"),
        Some(Action::Window(WindowAction::ToggleSplit))
    );
}

#[test]
fn test_screenshot_commands() {
    assert_eq!(
        parse_action("screenshot"),
        Some(Action::Screenshot(ScreenshotAction::ActiveWindow))
    );
    assert_eq!(
        parse_action("screenshot this"),
        Some(Action::Screenshot(ScreenshotAction::ActiveWindow))
    );
    assert_eq!(
        parse_action("screenshot window"),
        Some(Action::Screenshot(ScreenshotAction::ActiveWindow))
    );
    assert_eq!(
        parse_action("screenshot this window"),
        Some(Action::Screenshot(ScreenshotAction::ActiveWindow))
    );
    assert_eq!(
        parse_action("screenshot screen"),
        Some(Action::Screenshot(ScreenshotAction::FullScreen))
    );
    assert_eq!(
        parse_action("capture screen"),
        Some(Action::Screenshot(ScreenshotAction::FullScreen))
    );
    assert_eq!(
        parse_action("take screenshot"),
        Some(Action::Screenshot(ScreenshotAction::FullScreen))
    );
    assert_eq!(
        parse_action("recapture"),
        Some(Action::Screenshot(ScreenshotAction::RecaptureLast))
    );
    assert_eq!(
        parse_action("snip"),
        Some(Action::Screenshot(ScreenshotAction::InteractiveSnip))
    );
    assert_eq!(
        parse_action("custom selection"),
        Some(Action::Screenshot(ScreenshotAction::InteractiveSnip))
    );
    assert_eq!(
        parse_action("screenshot pane left"),
        Some(Action::Screenshot(ScreenshotAction::DirectionalPane(
            Direction::Left
        )))
    );
}

#[test]
fn test_clipboard_commands() {
    assert_eq!(
        parse_action("type clipboard"),
        Some(Action::Clipboard(ClipboardAction::TypeClipboard))
    );
    assert_eq!(
        parse_action("type it"),
        Some(Action::Clipboard(ClipboardAction::TypeClipboard))
    );
    assert_eq!(
        parse_action("paste typing"),
        Some(Action::Clipboard(ClipboardAction::TypeClipboard))
    );
    assert_eq!(
        parse_action("clear clipboard"),
        Some(Action::Clipboard(ClipboardAction::ClearClipboard))
    );
    assert_eq!(
        parse_action("empty clipboard"),
        Some(Action::Clipboard(ClipboardAction::ClearClipboard))
    );
    assert_eq!(
        parse_action("strip clipboard"),
        Some(Action::Clipboard(ClipboardAction::StripFormatting))
    );
    assert_eq!(
        parse_action("strip clipboard formatting"),
        Some(Action::Clipboard(ClipboardAction::StripFormatting))
    );
}

#[test]
fn test_confirmation_and_app_lifecycle() {
    assert_eq!(parse_action("confirm"), Some(Action::Confirm));
    assert_eq!(parse_action("confirmed"), Some(Action::Confirm));
    assert_eq!(parse_action("confirm it"), Some(Action::Confirm));
    assert_eq!(
        parse_action("open brave"),
        Some(Action::Open("brave".to_string()))
    );
    assert_eq!(
        parse_action("open youtube"),
        Some(Action::Open("youtube".to_string()))
    );
    assert_eq!(
        parse_action("open up youtube"),
        Some(Action::Open("youtube".to_string()))
    );
    assert_eq!(
        parse_action("open the youtube"),
        Some(Action::Open("youtube".to_string()))
    );
    assert_eq!(
        parse_action("focus youtube"),
        Some(Action::Open("youtube".to_string()))
    );
    assert_eq!(
        parse_action("switch to youtube"),
        Some(Action::Open("youtube".to_string()))
    );
    assert_eq!(
        parse_action("go to youtube"),
        Some(Action::Open("youtube".to_string()))
    );
    assert_eq!(
        parse_action("launch code"),
        Some(Action::Open("code".to_string()))
    );
    assert_eq!(
        parse_action("close spotify"),
        Some(Action::Close("spotify".to_string()))
    );
    assert_eq!(
        parse_action("kill spotify"),
        Some(Action::Close("spotify".to_string()))
    );
}

#[test]
fn test_gui_computer_use_commands() {
    assert_eq!(
        parse_action("scroll"),
        Some(Action::Gui(GuiAction::Scroll {
            direction: GuiScrollDirection::Down,
            amount: 4,
        }))
    );
    assert_eq!(
        parse_action("scroll down"),
        Some(Action::Gui(GuiAction::Scroll {
            direction: GuiScrollDirection::Down,
            amount: 4,
        }))
    );
    assert_eq!(
        parse_action("scroll up"),
        Some(Action::Gui(GuiAction::Scroll {
            direction: GuiScrollDirection::Up,
            amount: 4,
        }))
    );
    assert_eq!(
        parse_action("scroll down 5"),
        Some(Action::Gui(GuiAction::Scroll {
            direction: GuiScrollDirection::Down,
            amount: 5,
        }))
    );
    assert_eq!(
        parse_action("scroll up 10 lines"),
        Some(Action::Gui(GuiAction::Scroll {
            direction: GuiScrollDirection::Up,
            amount: 10,
        }))
    );
    assert_eq!(
        parse_action("scroll left"),
        Some(Action::Gui(GuiAction::Scroll {
            direction: GuiScrollDirection::Left,
            amount: 4,
        }))
    );
    assert_eq!(
        parse_action("scroll right"),
        Some(Action::Gui(GuiAction::Scroll {
            direction: GuiScrollDirection::Right,
            amount: 4,
        }))
    );
    assert_eq!(
        parse_action("click submit"),
        Some(Action::Gui(GuiAction::Click {
            target: "submit".to_string(),
        }))
    );
    assert_eq!(
        parse_action("click on next"),
        Some(Action::Gui(GuiAction::Click {
            target: "next".to_string(),
        }))
    );
    assert_eq!(
        parse_action("tap continue"),
        Some(Action::Gui(GuiAction::Click {
            target: "continue".to_string(),
        }))
    );
    assert_eq!(
        parse_action("focus search"),
        Some(Action::Gui(GuiAction::Focus {
            target: "search".to_string(),
        }))
    );
    assert_eq!(
        parse_action("focus email"),
        Some(Action::Gui(GuiAction::Focus {
            target: "email".to_string(),
        }))
    );
    assert_eq!(
        parse_action("focus input username"),
        Some(Action::Gui(GuiAction::Focus {
            target: "username".to_string(),
        }))
    );
    assert_eq!(
        parse_action("press enter"),
        Some(Action::Gui(GuiAction::PressKey {
            combo: "enter".to_string(),
        }))
    );
    assert_eq!(
        parse_action("press escape"),
        Some(Action::Gui(GuiAction::PressKey {
            combo: "escape".to_string(),
        }))
    );
    assert_eq!(
        parse_action("type hello world"),
        Some(Action::Gui(GuiAction::SetValue {
            target: "focused".to_string(),
            value: "hello world".to_string(),
        }))
    );
}

#[test]
fn test_file_archive_commands() {
    assert_eq!(
        parse_action("zip selected"),
        Some(Action::File(FileAction::ZipSelected))
    );
    assert_eq!(
        parse_action("zip selected file"),
        Some(Action::File(FileAction::ZipSelected))
    );
    assert_eq!(
        parse_action("zip this"),
        Some(Action::File(FileAction::ZipSelected))
    );
    assert_eq!(
        parse_action("compress selected"),
        Some(Action::File(FileAction::ZipSelected))
    );
    assert_eq!(
        parse_action("compress this"),
        Some(Action::File(FileAction::ZipSelected))
    );
    assert_eq!(
        parse_action("unzip selected"),
        Some(Action::File(FileAction::UnzipSelected {
            into_subfolder: true
        }))
    );
    assert_eq!(
        parse_action("unzip this"),
        Some(Action::File(FileAction::UnzipSelected {
            into_subfolder: true
        }))
    );
    assert_eq!(
        parse_action("extract selected"),
        Some(Action::File(FileAction::UnzipSelected {
            into_subfolder: true
        }))
    );
    assert_eq!(
        parse_action("extract here"),
        Some(Action::File(FileAction::UnzipSelected {
            into_subfolder: false
        }))
    );
    assert_eq!(
        parse_action("zip"),
        Some(Action::File(FileAction::ZipSelected))
    );
    assert_eq!(
        parse_action("zip them"),
        Some(Action::File(FileAction::ZipSelected))
    );
    assert_eq!(
        parse_action("jip them"),
        Some(Action::File(FileAction::ZipSelected))
    );
    assert_eq!(
        parse_action("unzip"),
        Some(Action::File(FileAction::UnzipSelected {
            into_subfolder: true
        }))
    );
    assert_eq!(
        parse_action("unzip all"),
        Some(Action::File(FileAction::UnzipSelected {
            into_subfolder: true
        }))
    );
    assert_eq!(
        parse_action("unchip all"),
        Some(Action::File(FileAction::UnzipSelected {
            into_subfolder: true
        }))
    );
    assert_eq!(
        parse_action("anjip"),
        Some(Action::File(FileAction::UnzipSelected {
            into_subfolder: true
        }))
    );
    assert_eq!(
        parse_action("unjip them all"),
        Some(Action::File(FileAction::UnzipSelected {
            into_subfolder: true
        }))
    );
}

#[test]
fn test_unrecognized_returns_none() {
    assert_eq!(
        parse_action("find the latest receipt in Downloads and attach it in Slack"),
        None
    );
    assert_eq!(parse_action("what is the weather in Tokyo"), None);
    assert_eq!(parse_action("hello computer"), None);
    assert_eq!(parse_action(""), None);
}

#[test]
fn test_fuzz_parser_no_panic() {
    use proptest::prelude::*;
    proptest!(|(s in "\\PC{0,100}")| {
        let _ = parse_action(&s);
    });
}
