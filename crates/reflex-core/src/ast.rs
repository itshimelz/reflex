//! AST definitions for Reflex actions.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaAction {
    Play,
    Pause,
    Toggle,
    Next,
    Previous,
    Replay,
    SeekForward(u32),
    SeekBackward(u32),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioAction {
    VolumeStep(i8),
    VolumeSet(u8),
    MuteToggle,
    Mute(bool),
    MicMuteToggle,
    SwitchPort,
    AppAudio { target: String, mode: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkspaceAction {
    Focus(u8),
    MoveWindow { workspace: u8, follow: bool },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowAction {
    MoveDirection(Direction),
    FocusDirection(Direction),
    CloseActive,
    ToggleFullscreen,
    ToggleFloating,
    TogglePin,
    CenterWindow,
    ToggleSplit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScreenshotAction {
    ActiveWindow,
    NamedApp(String),
    DirectionalPane(Direction),
    FullScreen,
    RecaptureLast,
    InteractiveSnip,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    Media(MediaAction),
    Audio(AudioAction),
    Workspace(WorkspaceAction),
    Window(WindowAction),
    Screenshot(ScreenshotAction),
    Open(String),
    Close(String),
    MoveNamed {
        app: String,
        workspace: u8,
        follow: bool,
    },
    Confirm,
}
