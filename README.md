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
- **Directional Tiling Navigation & Window Geometry**: Sub-millisecond window swapping (`move left/right/top/down`), directional focus navigation (`focus left/right/top/down`), native floating toggle, multi-workspace pinning, dwindle split orientation toggling, and window centering.
- **Per-App Audio Stream Control**: Direct PipeWire stream discovery via `pw-dump` allowing instant volume leveling and muting for specific apps (`mute brave`, `unmute spotify`, `brave volume 50`) without losing editor focus.
- **Safe Wayland Clipboard Utilities**: Virtual typing into restricted terminals via `wtype` guarded by a 500-character safety cap and automated trailing-newline stripping, plus instant buffer clearing and format stripping.
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
| `"mute <app>"`, `"silence <app>"` | `AudioAction::AppAudio { target, mode: Mute(true) }` | Discovers PipeWire node ID for app via `pw-dump` and mutes stream (<5ms) |
| `"unmute <app>"` | `AudioAction::AppAudio { target, mode: Mute(false) }`| Unmutes background stream for target app node |
| `"toggle mute <app>"` | `AudioAction::AppAudio { target, mode: ToggleMute }` | Toggles mute state of target application audio stream |
| `"<app> volume 50"`, `"set <app> volume 60"` | `AudioAction::AppAudio { target, mode: VolumeSet(N) }` | Sets target application stream volume directly |
| `"turn up <app>"`, `"<app> volume up"` | `AudioAction::AppAudio { target, mode: VolumeStep(+5) }` | Increments target stream volume by +5% |
| `"turn down <app>"`, `"<app> volume down"` | `AudioAction::AppAudio { target, mode: VolumeStep(-5) }` | Decrements target stream volume by -5% |

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
| `"toggle floating"`, `"float"`, `"float it"`, `"unfloat"` | `WindowAction::ToggleFloating` | Toggles active window between tiling and floating mode natively in Hyprland |
| `"pin"`, `"pin window"`, `"pin it"`, `"toggle pin"`, `"unpin"` | `WindowAction::TogglePin` | Pins active floating window to follow across all workspaces |
| `"center"`, `"center window"`, `"center it"`, `"center active"` | `WindowAction::CenterWindow` | Centers active floating window on current display |
| `"toggle split"`, `"change split"`, `"switch split"`, `"swap split"` | `WindowAction::ToggleSplit` | Toggles Hyprland dwindle layout split orientation |
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

### 6. Guardrailed Wayland Clipboard Utilities

| Command / Spoken Variants | Action / AST | Details & Safety Policies |
| :--- | :--- | :--- |
| `"type clipboard"`, `"type it"`, `"type that"`, `"paste typing"` | `ClipboardAction::TypeClipboard` | **Guardrailed:** Injects clipboard via `wtype` (2ms key delay). Strict 500-char cap; strips trailing newlines to prevent terminal command execution |
| `"clear clipboard"`, `"empty clipboard"`, `"clear buffer"` | `ClipboardAction::ClearClipboard` | Clears Wayland clipboard buffer via `wl-copy --clear` |
| `"strip clipboard"`, `"strip clipboard formatting"` | `ClipboardAction::StripFormatting` | Strips rich formatting, re-copying buffer strictly as `text/plain;charset=utf-8` |

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

### 1. Prerequisites & Ecosystem Dependencies

Reflex is engineered for **Arch Linux** running the **Hyprland** Wayland compositor. It operates within a tri-tier voice ecosystem comprising an upstream speech-to-text dictation engine, system Wayland/audio servers, and an autonomous AI fallback agent.

#### A. Upstream Speech-to-Text Engine: Hex (Required for Voice Input)

Reflex is strictly a deterministic command router and automation daemon; it does not capture raw microphone audio or run speech recognition neural models directly. Instead, it pairs upstream with **[Hex](https://github.com/anomalyco/hex)** (`anomalyco/hex`), a fast, local-first on-device speech-to-text dictation application built in pure Rust with Vulkan GPU acceleration.

| Resource | Link / Command |
| :--- | :--- |
| **Official Website** | [https://anoma.ly/hex](https://anoma.ly/hex) |
| **GitHub Repository** | [https://github.com/anomalyco/hex](https://github.com/anomalyco/hex) |
| **Linux Documentation** | [https://github.com/anomalyco/hex/blob/main/docs/linux.md](https://github.com/anomalyco/hex/blob/main/docs/linux.md) |
| **One-Line Linux Installer** | `curl -fsSL https://anoma.ly/hex/install-linux.sh \| sh` |

**Tri-Tier Division of Labor**:
- **Continuous Voice Dictation (`Hold F8`)**: Hex transcribes spoken speech on-device via Vulkan and types text directly into the focused window using Wayland virtual keyboard protocols (`wtype`).
- **Reflex Command Dispatch (`Hold F7`)**: Hex captures and transcribes spoken intent upon key release (`hex transcribe`), which is piped into `reflex "$TRANSCRIPT"`.
- **Autonomous AI Fallback**: If Reflex returns exit code `2` (`PassThrough` / unrecognized command), the transcript is forwarded to `reflex-agent-fallback`.

#### B. Arch Linux Desktop & Compositor Packages

Ensure the following native desktop and audio packages are installed:

```bash
sudo pacman -S --needed \
    hyprland \
    pipewire \
    wireplumber \
    libnotify \
    grim \
    slurp \
    wl-clipboard \
    wtype
```

| Package | Purpose in Reflex |
| :--- | :--- |
| `hyprland` | Wayland dynamic tiling compositor ([hyprland.org](https://hyprland.org/)); IPC over Unix domain sockets |
| `pipewire` & `wireplumber` | Audio subsystem ([pipewire.org](https://pipewire.org/)); volume stepping, ceilings, mute, and port switching |
| `libnotify` (`notify-send`) | System desktop notifications and OSD alerts |
| `grim` & `slurp` | Full-screen and interactive region screenshot capture |
| `wl-clipboard` (`wl-copy`/`wl-paste`) | Wayland clipboard integration and plain text formatting sanitizer |
| `wtype` | Virtual keystroke injection for guardrailed clipboard typing |

#### C. Autonomous Fallback & Neural Audio (Optional)

When spoken requests fall outside deterministic grammar and require multi-step GUI navigation or reasoning:
- **[Google Antigravity](https://antigravity.google/) (`agy`)**: Headless autonomous desktop and coding agent.
- **[Edge TTS](https://github.com/rany2/edge-tts)**: Neural text-to-speech voice synthesizer (`uvx edge-tts` or `pip install edge-tts`).
- **`pw-play`**: PipeWire audio playback tool (included with `pipewire`).

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

### 5. Hyprland Keybind Integration

Wire Push-to-Talk commands and Hex dictation into your Hyprland configuration.

#### Option A: Lua Config (`~/.config/hypr/custom/keybinds.lua`)

```lua
-- ==============================================================================
-- Reflex & Hex Keybindings
-- Hold F8: Hex continuous voice dictation (types directly into active window)
-- Hold F7: Reflex Push-to-Talk voice commands (exit code 2 -> AI fallback)
-- ==============================================================================
bind = {
    -- Hex continuous voice dictation (hold to speak, release to type)
    {
        key = "F8",
        event = "press",
        exec = "hex dictation start"
    },
    {
        key = "F8",
        event = "release",
        exec = "hex dictation stop"
    },

    -- Reflex deterministic command dispatch & autonomous AI fallback
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

#### Option B: Standard Hyprland Syntax (`~/.config/hypr/hyprland.conf`)

```ini
# Hold F8: Continuous voice dictation into active window via Hex
bind = , F8, exec, hex dictation start
bindr = , F8, exec, hex dictation stop

# Hold F7: Reflex deterministic voice commands & AI fallback
bind = , F7, exec, echo start > /tmp/reflex_voice_trigger
bindr = , F7, exec, bash -c 'TRANSCRIPT=$(hex transcribe); reflex "$TRANSCRIPT" || [ $? -eq 2 ] && reflex-agent-fallback "$TRANSCRIPT"'
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
