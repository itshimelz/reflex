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
fn test_confirmation_and_app_lifecycle() {
    assert_eq!(parse_action("confirm"), Some(Action::Confirm));
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
