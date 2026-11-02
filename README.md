# ⚡ Reflex

> **Sub-Millisecond Deterministic Desktop Automation & Voice Operator for Arch Linux & Hyprland**

[![Language](https://img.shields.io/badge/Language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Compositor](https://img.shields.io/badge/Compositor-Hyprland%200.57+-blue.svg)](https://hyprland.org/)
[![Audio](https://img.shields.io/badge/Audio-PipeWire%20%2F%20WirePlumber-green.svg)](https://pipewire.org/)
[![License](https://img.shields.io/badge/License-MIT%20%2F%20Apache--2.0-blue.svg)](LICENSE)

**Reflex** is a high-speed resident automation daemon (`reflexd`) and CLI client (`reflex`) engineered in pure Rust. It delivers sub-5ms deterministic command dispatch, single-instance application focus/launch, safety guardrails, procedural earcon synthesis, and seamless autonomous AI fallback for power users on Wayland / Hyprland.

---

## 🚀 Key Highlights & Architecture

- **Sub-Millisecond Pure Rust Hot Path**: Zero shell subprocesses (`hyprctl`, `wpctl`, and `playerctl` are strictly eliminated from the execution path). Communicates directly over local Unix domain sockets and D-Bus.
- **Hyprland Dual-Socket Protocol**: Direct connection to Hyprland's `.socket.sock` for zero-latency command evaluation, paired with asynchronous decoupled verification on `.socket2.sock`.
- **4-Tier Application Registry**: Fast hybrid resolver for desktop apps and Chromium/Brave PWAs:
  - **Tier 1**: Exact `StartupWMClass`, `.desktop` file ID, and compositor class match.
  - **Tier 2**: User-defined aliases configured in `config.toml`.
  - **Tier 3**: Isolated title token extraction (splitting on `-`, `|`, `•`, `—`, `:`).
  - **Tier 4**: Bounded Levenshtein fuzzy matching ($\text{distance} \le 1$).
- **Smart Single-Instance Lifecycle**:
  - If the requested app/PWA is already running (e.g., YouTube PWA), switches workspace and brings focus to that exact window address in `<1ms`.
  - If the app is closed, sanitizes the desktop entry `Exec` line (stripping parameter flags like `%U`/`%u`) and launches it detached.
- **Hardened Safety Guardrails**:
  - **Volume Ceiling Enforcement**: Clamps maximum volume to 80% (0 dB limit).
  - **Velocity Throttling**: Restricts sudden volume jumps to +15% per 500ms sliding window.
  - **Protected Window Confirmation**: Attempts to close critical windows (`kitty`, `brave`, `nvim`, `code`, `antigravity`) pin the window address and require a spoken `"confirm"` within 4.0 seconds.
- **Directional Tiling Navigation**: Sub-millisecond window swapping (`move left/right/top/down`, `swap left/right/top/down`) and directional focus navigation (`focus left/right/top/down`).
- **Procedural In-Memory Earcons**: Zero-file-load SIMD sine synthesis generating crisp audible feedback for command success, warnings, errors, camera shutter, and confirmation chimes.
- **Autonomous AI Fallback (`reflex-agent-fallback`)**: When spoken requests fall outside deterministic grammar, Reflex exits with code `2` (`PassThrough`), instantly routing the transcript to headless `Google Antigravity` (`agy`) with in-place synchronized QuickShell OSD notifications and neural TTS playback.

---

## 📋 Command Reference & Grammar

Reflex normalizes spoken transcripts in memory (stripping conversational filler like *"can you please"*, *"hey"*, punctuation, and phonetic homophones).

### 1. Audio & Volume Control (PipeWire / WirePlumber)

| Command / Spoken Variants | Action / AST | Details & Guardrails |
| :--- | :--- | :--- |
| `"volume up"`, `"louder"`, `"turn it up"` | `AudioAction::VolumeStep(+5)` | Steps volume up by +5% (velocity checked) |
| `"volume down"`, `"quieter"`, `"softer"` | `AudioAction::VolumeStep(-5)` | Steps volume down by -5% |
| `"volume up by 15"`, `"louder by 10%"` | `AudioAction::VolumeStep(+N)` | Steps volume up by $N$% |
| `"volume down by 20"`, `"quieter by 10%"`| `AudioAction::VolumeStep(-N)` | Steps volume down by $N$% |
| `"set volume to 60"`, `"volume 40%"` | `AudioAction::VolumeSet(N)` | Sets absolute volume clamped to 80% ceiling |
| `"mute"`, `"silence"` | `AudioAction::Mute(true)` | Mutes default audio sink |
| `"unmute"` | `AudioAction::Mute(false)` | Unmutes default audio sink |
| `"toggle mute"` | `AudioAction::MuteToggle` | Toggles output mute state |
| `"toggle mic"`, `"mute mic"`, `"unmute mic"`| `AudioAction::MicMuteToggle`| Toggles default microphone input mute |
| `"switch audio"`, `"switch speaker"`, `"headphones"`, `"switch to headphones"` | `AudioAction::SwitchPort` | Cycles physical output port (Speakers $\leftrightarrow$ Headphones) |

### 2. Media Playback (MPRIS via D-Bus)

| Command / Spoken Variants | Action / AST | Details |
| :--- | :--- | :--- |
| `"play"`, `"resume"`, `"unpause"` | `MediaAction::Play` | Sends `Play` to active media players |
| `"pause"`, `"stop"`, `"stop music"`, `"pause video"`, `"stop playback"` | `MediaAction::Pause` | Broadcasts asynchronous `Pause` over D-Bus (<2ms) |
| `"toggle media"`, `"play pause"` | `MediaAction::Toggle` | Calls `PlayPause` on MPRIS interface |
| `"next"`, `"skip"`, `"next track"` | `MediaAction::Next` | Skips to next track |
| `"previous"`, `"prev"`, `"last song"` | `MediaAction::Previous` | Rewinds to previous track |
| `"replay"`, `"restart track"` | `MediaAction::Replay` | Restarts current track from beginning |
| `"seek forward 10 seconds"` | `MediaAction::SeekForward(N)` | Jumps forward by $N$ seconds |
| `"seek backward 30 seconds"` | `MediaAction::SeekBackward(N)` | Jumps backward by $N$ seconds |

### 3. Workspace Navigation & Window Relocation

| Command / Spoken Variants | Action / AST | Details |
| :--- | :--- | :--- |
| `"workspace 3"`, `"go to workspace 5"`, `"switch to 2"` | `WorkspaceAction::Focus(N)` | Switches to workspace $N$ (<1ms) |
| `"move window to workspace 6"`, `"move to 4"` | `WorkspaceAction::MoveWindow { workspace: N, follow: false }` | Relocates active window without moving focus |
| `"move to 6 and follow"`, `"move window 3 follow"` | `WorkspaceAction::MoveWindow { workspace: N, follow: true }` | Relocates active window and follows focus |
| `"move YouTube to 4"`, `"move Spotify to 2"` | `Action::MoveNamed { app, workspace, follow }` | Resolves app via 4-tier registry and moves it |

### 4. Window Directional Navigation & Lifecycle

| Command / Spoken Variants | Action / AST | Details |
| :--- | :--- | :--- |
| `"focus left"`, `"focus right"`, `"focus top"`, `"focus up"`, `"focus down"`, `"focus bottom"` | `WindowAction::FocusDirection(Dir)` | Switches focus to adjacent tiled window |
| `"move left"`, `"move right"`, `"move top"`, `"move up"`, `"move down"`, `"swap left"`, `"swap right"` | `WindowAction::MoveDirection(Dir)` | Swaps active window position in tiling layout (<1ms) |
| `"fullscreen"`, `"toggle fullscreen"` | `WindowAction::ToggleFullscreen` | Toggles fullscreen mode |
| `"close"`, `"close it"`, `"close this"`, `"close window"`, `"kill it"` | `WindowAction::CloseActive` | **Guardrailed:** Closes active window. Protected classes require `"confirm"` |
| `"confirm"` | `Action::Confirm` | Executes pending close on pinned window address |
| `"open <app>"`, `"launch <app>"`, `"focus <app>"`, `"switch to <app>"` | `Action::Open(App)` | **Single-Instance Aware:** Focuses existing window or launches via `.desktop` |
| `"close <app>"`, `"kill <app>"` | `Action::Close(App)` | **Guardrailed:** Resolves and closes specific application |

### 5. Screenshots (`grim` + `wl-copy`)

| Command / Spoken Variants | Action / AST | Details |
| :--- | :--- | :--- |
| `"screenshot"`, `"screenshot window"`, `"screenshot this"` | `ScreenshotAction::ActiveWindow` | Captures focused window geometry directly |
| `"screenshot screen"`, `"capture screen"`, `"full screenshot"` | `ScreenshotAction::FullScreen` | Captures entire output canvas |
| `"snip"`, `"custom selection"`, `"select area"` | `ScreenshotAction::InteractiveSnip`| Launches interactive selection snip |
| `"screenshot pane left/right/up/down"` | `ScreenshotAction::DirectionalPane(Dir)`| Captures adjacent tiled pane |

---

## 🛠️ Project Structure

```
reflex/
├── Cargo.toml                    # Root workspace definition
├── config/
│   └── default_config.toml       # Reference configuration template
├── crates/
│   ├── reflex-core/              # AST, parser, normalizer, registry, guardrails
│   ├── reflexd/                  # High-performance resident daemon & drivers
│   └── reflex/                   # Thin sub-millisecond CLI client
├── scripts/
│   └── reflex-agent-fallback     # OSD-synchronized cognitive AI fallback runner
├── systemd/
│   └── reflexd.service           # Systemd user service unit
└── README.md                     # Project documentation
```

---

## 📦 Installation & Setup

### ⚡ Quick Install (Pre-compiled Binary)

Install the latest release bundle (`reflex`, `reflexd`, `reflex-agent-fallback`, systemd service, and default config) in seconds:

```bash
curl -sSL https://raw.githubusercontent.com/itshimelz/reflex/main/install.sh | bash
```

Custom options (version, destination directory, or skipping service/config):
```bash
# Install specific version to ~/.local/bin
REFLEX_VERSION=v0.1.0 INSTALL_DIR=~/.local/bin curl -sSL https://raw.githubusercontent.com/itshimelz/reflex/main/install.sh | bash
```

---

### 1. Prerequisites

Reflex is designed for **Arch Linux** running **Hyprland**. Ensure the following packages are installed:

```bash
sudo pacman -S --needed \
    hyprland \
    pipewire \
    wireplumber \
    libnotify \
    grim \
    slurp \
    wl-clipboard \
    cargo
```

### 2. Build and Install

```bash
git clone https://github.com/itshimelz/reflex.git
cd reflex

# Build optimized release binaries
cargo build --release

# Copy binaries to your PATH
cp target/release/reflex target/release/reflexd ~/.cargo/bin/

# Install the fallback script
cp scripts/reflex-agent-fallback ~/.local/bin/
chmod +x ~/.local/bin/reflex-agent-fallback
```

### 3. User Configuration (`~/.config/reflex/config.toml`)

Create `~/.config/reflex/config.toml` to customize volume safety boundaries and PWA aliases:

```toml
[volume]
ceiling = 80              # Hard maximum volume (0 dB gain limit)
max_step_velocity = 15    # Maximum permitted jump in volume per 500ms sliding window

[guardrails]
confirmation_timeout_secs = 4.0
unkillable_classes = [
    "kitty",
    "neovim",
    "nvim",
    "brave",
    "brave-browser",
    "code",
    "vscode",
    "antigravity"
]

[aliases]
# Custom user aliases mapping spoken names to cryptic PWA or desktop window classes
youtube   = "brave-agimnkijcaahngcdmfeangaknmldooml-Default"
whatsapp  = "brave-hnpfjngllnobngcgfapefoaidbinmjnm-Default"
instagram = "brave-maonjhobmgipjnlalmgmflffnflfljbl-Default"
studio    = "jetbrains-studio"
idea      = "jetbrains-idea"
```

### 4. Enable Systemd User Service

```bash
# Copy and enable systemd user service
mkdir -p ~/.config/systemd/user
cp systemd/reflexd.service ~/.config/systemd/user/

systemctl --user daemon-reload
systemctl --user enable --now reflexd.service

# Verify service is running
systemctl --user status reflexd.service
```

### 5. Hyprland Keybind Integration (`~/.config/hypr/custom/keybinds.lua`)

Wire push-to-talk into your Hyprland configuration:

```lua
-- Push-to-Talk (Hold F7): Speak command -> Release to execute
-- Exit code 0: Deterministic command executed successfully
-- Exit code 2: Fallback triggered -> routes to reflex-agent-fallback
bind = {
    {
        key = "F7",
        event = "press",
        exec = "echo start > /tmp/reflex_voice_trigger"
    },
    {
        key = "F7",
        event = "release",
        exec = "TRANSCRIPT=$(hex transcribe) && (reflex \"$TRANSCRIPT\" || [ $? -eq 2 ] && reflex-agent-fallback \"$TRANSCRIPT\")"
    }
}
```

---

## 🧪 Testing & Verification

Run the comprehensive unit, property, and integration test suite:

```bash
cargo test --workspace
```

Test commands directly from your terminal:

```bash
# Volume control
reflex "volume up"
reflex "volume 50"

# Application lifecycle
reflex "open youtube"
reflex "open dolphin"
reflex "close it"

# Tiling navigation
reflex "move left"
reflex "focus top"

# Safe closing of protected windows (triggers confirmation chime)
reflex "close youtube"
reflex "confirm"
```

---

## ⚡ Benchmarks

| Operation | Latency | Mechanism |
| :--- | :--- | :--- |
| **CLI IPC Round-Trip** | **< 1.0 ms** | Unix Domain Socket (`/run/user/1000/reflex.sock`) |
| **Hyprland Dispatch** | **< 0.5 ms** | Direct native `.socket.sock` evaluation |
| **MPRIS Media Pause** | **< 1.8 ms** | Non-blocking asynchronous D-Bus `zbus` call |
| **Window Close Execution** | **< 0.4 ms** | Compositor Lua table window dispatch |
| **Single-Instance Focus** | **< 1.5 ms** | Workspace switch + address-pinned focus |
| **Daemon Memory Footprint** | **~2.8 MB** | Zero JVM/Node runtime, pure compiled Rust binary |

---

## 📄 License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
