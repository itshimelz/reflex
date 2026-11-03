# Reflex: Commands and Capabilities Reference Manual

**Reflex** is a sub-5ms, deterministic, low-latency desktop automation engine written in Rust for Arch Linux running Hyprland, PipeWire/WirePlumber, and QuickShell.

This document serves as the exhaustive, single-source-of-truth reference for all supported voice/CLI commands, natural language variants, 4-tier application resolution, safety guardrails, synthesized earcons, and system integration.

---

## 1. Quick Start & Execution Modes

### Push-to-Talk Keybinds (`~/.config/hypr/custom/keybinds.lua`)
- **Hold `F7`**: Speak any desktop command or natural request, release key to execute:
  - If the command matches Reflex deterministic grammar $\rightarrow$ executes instantly (<1ms) with audible earcon feedback.
  - If the command is an open-ended goal or outside deterministic grammar $\rightarrow$ Reflex exits with code `2` (`PassThrough`), instantly handing off the transcript to headless `agy` (`Google Antigravity` + `agent-desktop`).
- **Hold `F8`**: Hex continuous speech dictation (types spoken words directly into the active window via Wayland virtual keyboard).

### Direct CLI Invocations
```bash
# Deterministic command execution
reflex "volume up"
reflex "move YouTube to workspace 4"
reflex "screenshot window"

# Check exit code
reflex "set volume 100" || echo "Exited with code $?"
```

---

## 2. Command Grammar & Capabilities Inventory

All commands support natural phrasing, word or digit numbers (0–100), optional prepositions (`to`, `by`, `in`), and conversational preambles (`please`, `can you`, `could you`).

### 2.1 Audio & Volume Management
Direct control over WirePlumber audio sinks and sources without spawning blocking shell processes.

| Capability | Spoken Variants / Synonyms | Action / AST | Details & Guardrails |
| :--- | :--- | :--- | :--- |
| **Volume Up (Default)** | `"volume up"`, `"sound up"`, `"louder"`, `"turn it up"`, `"turn sound up"` | `AudioAction::VolumeStep(+5)` | Steps volume up by +5%. |
| **Volume Down (Default)** | `"volume down"`, `"sound down"`, `"quieter"`, `"softer"`, `"turn it down"`, `"turn sound down"` | `AudioAction::VolumeStep(-5)` | Steps volume down by -5%. |
| **Volume Up by N** | `"volume up by 10"`, `"sound up by 10"`, `"volume up 15%"`, `"louder by ten"` | `AudioAction::VolumeStep(+N)` | Steps volume by specified percentage $N$. |
| **Volume Down by N** | `"volume down by 10"`, `"sound down by 10"`, `"volume down 20%"`, `"quieter by five"` | `AudioAction::VolumeStep(-N)` | Decrements volume by specified percentage $N$. |
| **Set Absolute Volume / Sound** | `"sound 50"`, `"sound 55"`, `"sound sound 50"`, `"set sound to 50"`, `"volume 50"`, `"volume 55"`, `"volume volume 50"`, `"set volume to 60"` | `AudioAction::VolumeSet(N)` | Sets volume directly to $N$%. Direct sets are unclamped when `clamp_volume = false` in `config.toml`. |
| **Mute Output** | `"mute"`, `"mute audio"`, `"silence"` | `AudioAction::Mute(true)` | Mutes default audio sink. |
| **Unmute Output** | `"unmute"`, `"unmute audio"` | `AudioAction::Mute(false)` | Unmutes default audio sink. |
| **Toggle Output Mute** | `"toggle mute"` | `AudioAction::MuteToggle` | Toggles output sink mute state. |
| **Toggle Microphone Mute** | `"toggle mic"`, `"mute mic"`, `"unmute mic"` | `AudioAction::MicMuteToggle` | Toggles default microphone source mute state. |
| **Switch Audio Port** | `"switch audio"`, `"switch speaker"`, `"toggle speaker"`, `"toggle audio"`, `"change speaker"`, `"swap audio"`, `"headphones"`, `"switch to headphones"`, `"switch to speaker"` | `AudioAction::SwitchPort` | Cycles physical output port (e.g., Headphones $\leftrightarrow$ Speakers). |
| **Mute Named App** | `"mute brave"`, `"mute spotify"`, `"silence discord"`, `"mute youtube"`, `"mute mpv"` | `AudioAction::AppAudio { target, mode: Mute(true) }` | Discovers PipeWire node ID for app via `pw-dump` and sets mute without stealing window focus (<5ms). |
| **Unmute Named App** | `"unmute brave"`, `"unmute spotify"`, `"unmute discord"` | `AudioAction::AppAudio { target, mode: Mute(false) }` | Unmutes background stream for target app node. |
| **Toggle App Mute** | `"toggle mute brave"`, `"toggle brave audio"` | `AudioAction::AppAudio { target, mode: ToggleMute }` | Toggles mute state of target application audio stream. |
| **Set App Volume** | `"set brave volume 50"`, `"brave volume 70"`, `"volume brave 40"`, `"brave volume to 60"` | `AudioAction::AppAudio { target, mode: VolumeSet(N) }` | Sets target application stream volume directly. |
| **Step App Volume** | `"turn up brave"`, `"brave volume up"`, `"turn down brave"`, `"brave volume down"` | `AudioAction::AppAudio { target, mode: VolumeStep(±5) }` | Increments/decrements target stream volume by ±5%. |

---

### 2.2 Media Playback (MPRIS via D-Bus)
Direct control over active media players (Chromium, Brave, Spotify, VLC, MPV) via asynchronous `zbus` D-Bus calls.

| Capability | Spoken Variants / Synonyms | Action / AST | Details |
| :--- | :--- | :--- | :--- |
| **Play / Resume** | `"play"`, `"resume"`, `"unpause"`, `"play music"`, `"play song"` | `MediaAction::Play` | Sends `Play` method call to all active MPRIS players. |
| **Pause / Stop Media** | `"stop"`, `"pause"`, `"stop music"`, `"pause music"`, `"stop video"`, `"pause video"`, `"stop song"`, `"stop playback"`, `"stop audio"` *(homophone: `"paws"`)* | `MediaAction::Pause` | Sends immediate asynchronous `Pause` method call over D-Bus to all active MPRIS players (<2ms). |
| **Toggle Play / Pause** | `"toggle media"`, `"toggle play"`, `"play pause"`, `"toggle playback"` | `MediaAction::Toggle` | Calls `PlayPause` on MPRIS interface. |
| **Next Track** | `"next"`, `"skip"`, `"next song"`, `"next track"` | `MediaAction::Next` | Skips to next track. |
| **Previous Track** | `"previous"`, `"prev"`, `"last song"`, `"back song"`, `"previous track"` | `MediaAction::Previous` | Rewinds to previous track. |
| **Replay Track** | `"replay"`, `"restart song"`, `"restart track"` | `MediaAction::Replay` | Re-triggers previous track or seeks to 0. |
| **Seek Forward** | `"seek forward 10 seconds"`, `"seek forward five"` | `MediaAction::SeekForward(N)` | Offsets playback position forward by $N$ microseconds. |
| **Seek Backward** | `"seek backward 30 seconds"`, `"seek backward ten"` | `MediaAction::SeekBackward(N)` | Offsets playback position backward by $N$ microseconds. |

---

### 2.3 Workspace Navigation & Window Relocation
Direct compositor control over Hyprland `.socket.sock` with decoupled verification on `.socket2.sock`.

| Capability | Spoken Variants / Synonyms | Action / AST | Details |
| :--- | :--- | :--- | :--- |
| **Focus Workspace** | `"workspace 3"`, `"go to workspace 5"`, `"switch to workspace 2"`, `"switch to 4"`, `"go to 1"` | `WorkspaceAction::Focus(N)` | Dispatches `workspace <N>`. |
| **Move Active Window** | `"move window to workspace 6"`, `"move this window workspace 6"`, `"move to 4"`, `"move this to workspace 2"` | `WorkspaceAction::MoveWindow { workspace: N, follow: false }` | Moves active window to workspace $N$ without shifting user focus. |
| **Move Active Window & Follow** | `"move to 6 and follow"`, `"move window to workspace 3 follow"` | `WorkspaceAction::MoveWindow { workspace: N, follow: true }` | Moves active window to workspace $N$ and switches focus to workspace $N$. |
| **Move Named App to Workspace** | `"move YouTube to 4"`, `"move Chrome to workspace 6"`, `"move Spotify 2"`, `"move WhatsApp to 5 and follow"` | `Action::MoveNamed { app, workspace, follow }` | Uses 4-Tier `AppRegistry` to locate target window by class/alias/title, moves it to workspace $N$. |

---

### 2.4 Window Management & Directional Navigation

| Capability | Spoken Variants / Synonyms | Action / AST | Details & Safety Policies |
| :--- | :--- | :--- | :--- |
| **Focus Direction** | `"focus left"`, `"focus right"`, `"focus up"`, `"focus top"`, `"focus down"`, `"focus bottom"`, `"focus window left/right/up/down"` | `WindowAction::FocusDirection(Dir)` | Switches focus to adjacent window in tiling layout. Supports both `up`/`down` and `top`/`bottom`. |
| **Swap / Move Window Direction** | `"move left"`, `"move right"`, `"move up"`, `"move top"`, `"move down"`, `"move bottom"`, `"move window left/right/top/down"`, `"swap left/right/top/down"` | `WindowAction::MoveDirection(Dir)` | Swaps active window position in tiling layout in the specified direction (<1ms). |
| **Toggle Floating** | `"toggle floating"`, `"float"`, `"float window"`, `"float it"`, `"unfloat"`, `"unfloat window"`, `"toggle float"` *(phonetic misrecognitions: `"toggle plot"`, `"toggle flot"`, `"plot window"`, `"flot window"`, `"plot it"`, `"flot it"`, `"unplot"`, `"unflot"`)* | `WindowAction::ToggleFloating` | Toggles active window between tiling and floating mode natively in Hyprland (<0.5ms). |
| **Toggle Pin Window** | `"pin"`, `"pin window"`, `"pin it"`, `"toggle pin"`, `"unpin"` | `WindowAction::TogglePin` | Pins active floating window to follow across all workspaces. |
| **Center Window** | `"center"`, `"center window"`, `"center it"`, `"center active"` | `WindowAction::CenterWindow` | Centers active floating window on current display. |
| **Toggle Dwindle Split** | `"toggle split"`, `"change split"`, `"switch split"`, `"swap split"` | `WindowAction::ToggleSplit` | Toggles Hyprland dwindle layout horizontal/vertical split orientation. |
| **Toggle Fullscreen** | `"fullscreen"`, `"toggle fullscreen"` | `WindowAction::ToggleFullscreen` | Toggles Hyprland fullscreen mode. |
| **Close Active Window** | `"close"`, `"close it"`, `"close this"`, `"close window"`, `"close the window"`, `"close this window"`, `"close active"`, `"close current"`, `"kill it"`, `"kill this"`, `"kill window"` | `WindowAction::CloseActive` | **Guardrailed:** Immediately closes the active window (<1ms). If active window is in the protected allowlist (`kitty`, `brave`, `nvim`, `code`, `antigravity`), pins window address and demands spoken `"confirm"` within 4.0s. |
| **Confirm Action** | `"confirm"`, `"confirmed"`, `"confirm it"` | `Action::Confirm` | Executes pending destructive action strictly on the pinned window address. |
| **Launch / Focus Application** | `"open youtube"`, `"open up youtube"`, `"open the youtube"`, `"launch code"`, `"focus youtube"`, `"switch to youtube"`, `"go to antigravity"`, `"open dolphin"` | `Action::Open(App)` | **Single-Instance Aware:** If the application is already open, switches to its workspace and focuses the window address (<1ms). If not running, resolves the `.desktop` file's `Exec` command via `AppRegistry` and launches it detached. |
| **Close Named Application** | `"close discord"`, `"close spotify"`, `"close dolphin"`, `"kill youtube"` | `Action::Close(App)` | Resolves target window via 4-Tier `AppRegistry`. **Guardrailed:** If the app is protected (`brave`, `kitty`, `antigravity`, `code`, etc.), pins address and demands spoken `"confirm"` within 4.0s before closing. |

---

### 2.5 Zero-Drag Screenshots (`grim` + `wl-copy`)
Instant asynchronous frame capture without manual drag selection. All captures are copied directly to the Wayland clipboard buffer and optionally saved.

| Capability | Spoken Variants / Synonyms | Action / AST | Details |
| :--- | :--- | :--- | :--- |
| **Active Window Snip** | `"screenshot"`, `"screenshot this"`, `"screenshot window"`, `"screenshot this window"` | `ScreenshotAction::ActiveWindow` | Queries cached geometry `{x, y, w, h}` of the focused window from Hyprland and captures only that box. |
| **Full Screen Snip** | `"screenshot screen"`, `"capture screen"`, `"take screenshot"`, `"full screenshot"` | `ScreenshotAction::FullScreen` | Captures entire output canvas. |
| **Directional Pane Snip** | `"screenshot pane left"`, `"screenshot pane right"` | `ScreenshotAction::DirectionalPane(Dir)` | Captures adjacent tiled pane based on layout tree. |
| **Recapture Last Area** | `"recapture"`, `"screenshot again"` | `ScreenshotAction::RecaptureLast` | Re-executes capture using the most recent bounding box. |
| **Interactive Selection** | `"snip"`, `"custom selection"`, `"select area"` | `ScreenshotAction::InteractiveSnip` | Launches `slurp` for manual box selection. |

---

### 2.6 Guardrailed Wayland Clipboard Utilities
Safe clipboard inspection, virtual keystroke injection via `wtype`, and sanitization.

| Capability | Spoken Variants / Synonyms | Action / AST | Details & Safety Policies |
| :--- | :--- | :--- | :--- |
| **Type Clipboard Buffer** | `"type clipboard"`, `"type it"`, `"type that"`, `"type buffer"`, `"paste typing"`, `"paste as typing"`, `"type"` | `ClipboardAction::TypeClipboard` | **Guardrailed:** Injects clipboard contents directly into active window using `wtype` (2ms key delay). Strict 500-char maximum ceiling to prevent compositor stalls; automatically strips trailing `\r` and `\n` to prevent accidental immediate shell execution in terminals. |
| **Clear Clipboard** | `"clear clipboard"`, `"empty clipboard"`, `"clear buffer"` | `ClipboardAction::ClearClipboard` | Empties the Wayland clipboard buffer via `wl-copy --clear`. |
| **Strip Formatting** | `"strip clipboard"`, `"strip clipboard formatting"`, `"clean clipboard"`, `"plain clipboard"` | `ClipboardAction::StripFormatting` | Strips rich HTML / RTF MIME types from clipboard, recopying buffer strictly as `text/plain;charset=utf-8`. |

---

### 2.7 Desktop Context & Computer-Use Primitives (Zero-LLM)
Semantic accessibility inspection via `agent-desktop` paired with Wayland virtual keystroke dispatch. Operates 100% locally on CPU without cloud LLMs or agent fallback overhead.

| Capability | Spoken Variants / Synonyms | Action / AST | Details & Execution Target |
| :--- | :--- | :--- | :--- |
| **Scroll Down (Default)** | `"scroll"`, `"scroll down"`, `"page down"` | `GuiAction::Scroll { direction: Down, amount: 4 }` | Dispatches downward page/container scrolling to the active window (<15ms). |
| **Scroll Up (Default)** | `"scroll up"`, `"page up"` | `GuiAction::Scroll { direction: Up, amount: 4 }` | Dispatches upward page/container scrolling to the active window (<15ms). |
| **Scroll by N Lines** | `"scroll down 5"`, `"scroll down by 10"`, `"scroll up five lines"` | `GuiAction::Scroll { direction, amount: N }` | Steps scrolling in specified direction by $N$ units. |
| **Scroll Horizontal** | `"scroll left"`, `"scroll right"` | `GuiAction::Scroll { direction: Left/Right, amount: 4 }` | Horizontal scrolling within wide containers or documents. |
| **Focus Form Field** | `"focus search"`, `"focus search bar"`, `"focus email"`, `"focus input username"`, `"focus field comment"` | `GuiAction::Focus { target }` | Queries active window AT-SPI accessibility tree, fuzzy-grounds the target name, and focuses the matching input field (<15ms). |
| **Search For Query (One-Shot)** | `"search for HSC result"`, `"search Adaalat"`, `"search lo fi music"` | `GuiAction::Search { query }` | One-shot macro: focuses search bar, replaces old text, types query, and executes Return in <200ms without LLM. |
| **Click Button / Element** | `"click submit"`, `"click on next"`, `"tap continue"`, `"click checkout"` | `GuiAction::Click { target }` | Resolves target button or clickable link on screen via fuzzy matching on name/description and dispatches semantic click. Falls through to Tier 3 agent if ungrounded. |
| **Direct Text Insertion** | `"type hello world"` | `GuiAction::SetValue { target: "focused", value }` | Synthesizes virtual typing into the currently focused input field. |
| **Send Key Combo** | `"press enter"`, `"press escape"`, `"press tab"`, `"press backspace"` | `GuiAction::PressKey { combo }` | Dispatches virtual key event directly into the active window. |

---

### 2.8 File & Archive Management (Zero-LLM)
Instant file manipulation, compression, and extraction for highlighted or selected files in Dolphin, desktop file managers, or terminals. Detects selected items via Wayland clipboard synchronization (`wtype` Ctrl+C + `wl-paste -t text/uri-list`) and executes native Linux tools and system calls in <100ms with zero AI tokens or latency.

> [!NOTE]
> **Full Multi-Selection & Batch Processing**: Every file and media action fully supports both single selected items and batch multi-selection. When multiple files are highlighted in Dolphin, Nautilus, or terminal:
> - **Aggregation actions** (e.g. `zip`, `file info`, `checksum`, `copy path`, `copy contents`) automatically combine, aggregate, or bundle the selected items.
> - **Batch conversion/mutation actions** (e.g. `unzip`, `convert image`, `extract audio`, `duplicate`, `trash`, `move`, `timestamp`, `clean`, `strip metadata`, `rename`) execute over each selected item concurrently or sequentially with automated conflict resolution.

| Capability | Spoken Variants / Synonyms | Action / AST | Details & Multi-Selection Support |
| :--- | :--- | :--- | :--- |
| **Zip Selected (Archive)** | `"zip"`, `"zip selected"`, `"zip this"`, `"zip file"`, `"compress selected"`, `"compress this"`, `"zip them"`, `"zip all"` *(ASR homophones: `"jip"`, `"jip them"`, `"jip all"`)* | `FileAction::ZipSelected` | **Multi-selection:** Bundles all selected items into `archive.zip` (or `archive_N.zip`). For single item, produces `<name>.zip`. |
| **Unzip Selected (Subfolder)** | `"unzip"`, `"unzip selected"`, `"unzip this"`, `"unzip file"`, `"extract selected"`, `"extract this"`, `"unzip them all"` *(ASR homophones: `"unjip"`, `"anjip"`, `"unchip"`)* | `FileAction::UnzipSelected { into_subfolder: true }` | **Multi-selection:** Extracts all selected archives into individual subfolders (`<archive>/`). Supports `.zip`, `.tar.gz`, `.tgz`, `.tar.xz`, `.tar.bz2`, `.tar`. |
| **Unzip / Extract In-Place** | `"extract here"`, `"unzip here"`, `"extract in place"`, `"unzip in place"` | `FileAction::UnzipSelected { into_subfolder: false }` | **Multi-selection:** Extracts all selected archives directly in-place in current folder. |
| **Copy Full Path** | `"copy path"`, `"copy the path"`, `"copy full path"`, `"copy file path"`, `"copy location"`, `"full path"`, `"file path"` *(homophones: `"copy pass"`, `"copy pat"`)* | `FileAction::CopyPath { full: true }` | **Multi-selection:** Joins absolute filesystem paths of all selected items with newlines (`\n`) into the Wayland clipboard. |
| **Copy Filename** | `"copy filename"`, `"copy file name"`, `"copy name"`, `"copy short path"`, `"filename"` | `FileAction::CopyPath { full: false }` | **Multi-selection:** Joins base filenames of all selected items with newlines (`\n`) into clipboard. |
| **Copy File Contents** | `"copy content"`, `"copy contents"`, `"copy text"`, `"copy code"`, `"copy file text"` | `FileAction::CopyContents` | **Multi-selection:** Reads UTF-8 text/code of selected files (<10MB ceiling) separated by header dividers (`// === filename ===`). |
| **Copy Image to Clipboard** | `"copy image"`, `"copy picture"`, `"copy photo"`, `"copy image to clipboard"` | `FileAction::CopyImage` | Reads selected image and streams PNG bitmap into `wl-copy -t image/png` via ImageMagick. |
| **Save Clipboard Here** | `"save clipboard"`, `"save clipboard here"`, `"save buffer"`, `"paste to file"`, `"paste clipboard here"` | `FileAction::SaveClipboardHere` | Detects clipboard type: writes bitmap images to `clipboard_<timestamp>.png` or text to `clipboard_<timestamp>.txt` in active folder. |
| **Duplicate Selected** | `"duplicate"`, `"duplicate file"`, `"duplicate this"`, `"duplicate selected"`, `"clone file"`, `"make copy"`, `"copy this"`, `"copy this file"` | `FileAction::DuplicateSelected` | **Multi-selection:** Loops over all selected files/directories, duplicating each into `<name>_copy.<ext>` with auto-incrementing collision avoidance. |
| **Trash Selected** | `"trash"`, `"trash selected"`, `"trash this"`, `"trash file"`, `"move to trash"`, `"send to trash"`, `"delete selected"` | `FileAction::TrashSelected` | **Multi-selection:** Moves all selected files to `~/.local/share/Trash/files/` and writes corresponding `.trashinfo` metadata for full Freedesktop restore compatibility. |
| **Move to Directory** | `"move to downloads"`, `"downloads"`, `"send to documents"`, `"documents"`, `"move to desktop"`, `"desktop"`, `"pictures"`, `"videos"`, `"music"` | `FileAction::MoveToDirectory(Dir)` | **Multi-selection:** Moves all selected files into the target XDG directory with auto-renaming on collision. |
| **Create New Folder** | `"new folder"`, `"new folder projects"`, `"create folder Assets"`, `"make directory src"` | `FileAction::NewFolder { name }` | Creates a new directory in the active location (defaults to `"New Folder"` if unspecified). |
| **Rename File** | `"rename to final_report"`, `"rename this to resume"`, `"rename file to backup"` | `FileAction::RenameSelected { new_name }` | **Multi-selection:** Single file renames directly; multiple files are renamed sequentially with indexed suffixes (`<name>_1.<ext>`, `<name>_2.<ext>`). |
| **Timestamp Filename** | `"timestamp"`, `"timestamp this"`, `"timestamp file"`, `"add timestamp"`, `"datestamp this"` | `FileAction::TimestampSelected` | **Multi-selection:** Prepends ISO date (`YYYY-MM-DD_<name>.<ext>`) to every selected item. |
| **Clean / Sanitize Filename** | `"clean filename"`, `"clean name"`, `"clean"`, `"sanitize filename"`, `"slugify filename"`, `"normalize filename"` | `FileAction::CleanFilename` | **Multi-selection:** Sanitizes names of all selected files (lowercasing, symbol-to-hyphen conversion, trimming). |
| **Strip EXIF / Metadata** | `"strip metadata"`, `"clean metadata"`, `"metadata"`, `"strip exif"`, `"remove metadata"`, `"strip tags"` | `FileAction::StripMetadata` | **Multi-selection:** Privacy cleaner for all selected files: strips EXIF GPS/camera data from images via ImageMagick, and cleans ID3/metadata from media via FFmpeg. |

---

### 2.9 Media Conversion & Power Tools (Zero-LLM)

| Capability | Spoken Variants / Synonyms | Action / AST | Details & Multi-Selection Support |
| :--- | :--- | :--- | :--- |
| **Extract Audio Track** | `"extract audio"`, `"rip audio"`, `"convert to mp3"`, `"extract mp3"`, `"mp3"`, `"convert to wav"`, `"convert to flac"`, `"convert to m4a"` | `FileAction::ExtractAudio { format }` | **Multi-selection:** Batch converts and extracts audio tracks from all selected video/audio items via FFmpeg (`libmp3lame`, `pcm_s16le`, `flac`, `aac`). |
| **Convert Image Format** | `"convert to png"`, `"png"`, `"convert to jpg"`, `"jpg"`, `"convert to webp"`, `"webp"`, `"make webp"`, `"convert to ico"`, `"save as png"` | `FileAction::ConvertImage { format }` | **Multi-selection:** Batch converts all selected images via ImageMagick preserving transparency and quality. |
| **Generate Animated GIF** | `"make gif"`, `"make a gif"`, `"create gif"`, `"convert to gif"`, `"turn into gif"` | `FileAction::MakeGif` | **Multi-selection:** Generates 15fps palette-optimized animated GIFs for each selected video container via FFmpeg. |
| **Render PDF to Images** | `"pdf to images"`, `"convert pdf to images"`, `"extract pdf pages"`, `"pdf pages to images"` | `FileAction::PdfToImages` | **Multi-selection:** Batch extracts each page of all selected PDF documents into 150 DPI PNG images via `pdftoppm`. |
| **File Information** | `"file info"`, `"file size"`, `"inspect file"`, `"file details"`, `"file properties"` | `FileAction::FileInfo` | **Multi-selection:** For single file, reports formatted size and permissions. For multiple items, calculates total combined size and counts (`N items: X MB total (A dirs, B files)`). |
| **Compute SHA-256 Checksum** | `"checksum"`, `"checksum this"`, `"sha256"`, `"sha 256"`, `"calculate checksum"`, `"hash file"` | `FileAction::ChecksumSelected` | **Multi-selection:** Calculates SHA-256 for all selected items, joins them into `<hash>  <filename>` format, and copies the complete list to Wayland clipboard. |
| **Open Folder in Terminal** | `"open in terminal"`, `"open terminal here"`, `"terminal here"`, `"open folder in terminal"`, `"terminal"` | `FileAction::OpenInTerminal` | Spawns `kitty` terminal directly in the parent directory of the selected file(s) or active folder. |

---

## 3. Phonetic Normalizer & Preambles

Reflex runs all input through an in-memory lexical normalizer (`reflex_core::normalizer`) before grammar parsing:

1. **Punctuation Trimming:** Strips trailing periods, question marks, exclamation marks, and semicolons emitted by ASR.
2. **Filler Stripping:** Automatically removes conversational preambles:
   - `"can you please"`, `"could you please"`, `"would you please"`, `"take a"`, `"please"`, `"can you"`, `"could you"`, `"would you"`, `"now"`, `"just"`, `"hey"`
3. **Phonetic & Accent Normalization:**
   - Homophones: `"paws"` $\rightarrow$ `"pause"`, `"mike"` $\rightarrow$ `"mic"`
   - Archive / Compression mispronunciations:
     - `"jip"`, `"jeep"`, `"gyp"`, `"zyp"`, `"zep"`, `"zipp"`, `"jib"` $\rightarrow$ `"zip"`
     - `"unjip"`, `"anjip"`, `"onjip"`, `"unchip"`, `"anchip"`, `"onchip"`, `"unjeep"`, `"anjeep"`, `"onzip"` $\rightarrow$ `"unzip"`
     - `"extrac"`, `"extrack"` $\rightarrow$ `"extract"`, `"compres"` $\rightarrow$ `"compress"`
   - File & Clipboard operations:
     - `"coppy"`, `"coppi"`, `"kopy"`, `"copee"` $\rightarrow$ `"copy"`
     - `"pass"`, `"pat"`, `"pash"`, `"paath"` $\rightarrow$ `"path"` (`"full pass"` $\rightarrow$ `"full path"`)
     - `"clip board"`, `"lib board"`, `"lip board"`, `"libboard"`, `"clip bord"` $\rightarrow$ `"clipboard"`
     - `"duplicat"`, `"dublicate"`, `"doop"`, `"dupe"`, `"dupe it"` $\rightarrow$ `"duplicate this"`
     - `"tresh"`, `"trush"`, `"treash"`, `"trash it"` $\rightarrow$ `"trash this"`
     - `"delit"`, `"dilit"`, `"delet"` $\rightarrow$ `"delete"`
     - `"polder"`, `"pholder"`, `"foldar"` $\rightarrow$ `"folder"`
     - `"klin"`, `"klean"`, `"clean it"`, `"sanitize it"` $\rightarrow$ `"clean filename"`
     - `"clean file name clean"`, `"clean filename clean"` $\rightarrow$ `"clean filename"`
     - `"meta data"`, `"strip it"` $\rightarrow$ `"strip metadata"`
     - `"time stamp"`, `"date stamp"`, `"datestamp"` $\rightarrow$ `"timestamp"`
     - `"check sum"`, `"sha 256"`, `"shah 256"`, `"shah256"`, `"sha two fifty six"` $\rightarrow$ `"sha256"` / `"checksum"`
     - `"terminel"`, `"tarminal"`, `"termnl"` $\rightarrow$ `"terminal"`
   - Standard Directories:
     - `"dawnload"`, `"dawnloads"`, `"down loads"` $\rightarrow$ `"downloads"`
     - `"dokuments"`, `"documants"` $\rightarrow$ `"documents"`
     - `"deskop"`, `"desktp"` $\rightarrow$ `"desktop"`
     - `"pikchurs"`, `"piktures"`, `"photos"`, `"potos"` $\rightarrow$ `"pictures"`
     - `"vedio"`, `"vedios"`, `"vidio"` $\rightarrow$ `"videos"`
   - Media & Extension spellings:
     - `"em pee three"`, `"mp 3"` $\rightarrow$ `"mp3"`
     - `"pee en gee"`, `"p n g"` $\rightarrow$ `"png"`
     - `"jay peg"`, `"j p g"` $\rightarrow$ `"jpg"`
     - `"web p"`, `"weppy"`, `"webby"`, `"webi"`, `"wepy"` $\rightarrow$ `"webp"`
     - `"pee dee eff"`, `"p d f"` $\rightarrow$ `"pdf"`
     - `"jif"`, `"giph"`, `"geef"` $\rightarrow$ `"gif"`
     - `"adio"`, `"odiyo"`, `"audo"` $\rightarrow$ `"audio"`
   - Accent & Navigation variations:
     - `"togoli"`, `"togol"`, `"toggole"`, `"togle"`, `"togal"` $\rightarrow$ `"toggle"`
     - `"flot"`, `"flote"`, `"floot"`, `"plot"` $\rightarrow$ `"float"`
     - `"floting"`, `"ploting"`, `"plotting"` $\rightarrow$ `"floating"`
     - `"speeker"`, `"spkr"` $\rightarrow$ `"speaker"`
     - `"valum"`, `"valume"`, `"volum"`, `"bolume"` $\rightarrow$ `"volume"`
4. **Repetition Deduplication & Multi-Word Phrase Sanitization:**
   - Single-token stutters: `"zip, zip"`, `"zip zip"`, `"jip, jip"` $\rightarrow$ `"zip"`; `"unzip, unzip"` $\rightarrow$ `"unzip"`; `"sound sound"` $\rightarrow$ `"sound"`; `"copy copy"` $\rightarrow$ `"copy"`; `"trash trash"` $\rightarrow$ `"trash"`; `"duplicate duplicate"` $\rightarrow$ `"duplicate"`; `"move move"` $\rightarrow$ `"move"`; `"mp3 mp3"` $\rightarrow$ `"mp3"`; `"png png"` $\rightarrow$ `"png"`; `"jpg jpg"` $\rightarrow$ `"jpg"`; `"pdf pdf"` $\rightarrow$ `"pdf"`; `"Weppy, Webby, Webby, Webby"` $\rightarrow$ `"webp"`
   - Multi-token phrase stutters (2-word & 3-word repetitions):
     - `"full path full path"` $\rightarrow$ `"full path"`
     - `"copy this copy this"` $\rightarrow$ `"copy this"`
     - `"duplicate this duplicate this"` $\rightarrow$ `"duplicate this"`
     - `"trash this trash this"` $\rightarrow$ `"trash this"`
     - `"strip metadata strip metadata"` $\rightarrow$ `"strip metadata"`
     - `"clean filename clean filename"` $\rightarrow$ `"clean filename"`
   - Split-word prefixes: `"un jip"`, `"an jip"`, `"un chip"`, `"an chip"` $\rightarrow$ `"unzip"`; `"time stamp"` $\rightarrow$ `"timestamp"`; `"meta data"` $\rightarrow$ `"metadata"`; `"re name"` $\rightarrow$ `"rename"`; `"file name"` $\rightarrow$ `"filename"`
   - Pronoun targets: `"zip them"`, `"zip them all"`, `"zip all"`, `"unzip them"`, `"unzip them all"`, `"unzip all"`
   - Window & app merges: `"work space"` $\rightarrow$ `"workspace"`, `"you tube"` $\rightarrow$ `"youtube"`, `"whats app"` $\rightarrow$ `"whatsapp"`, `"screen shot"` $\rightarrow$ `"screenshot"`
5. **Natural Number & Ordinal Parsing:**
   - Parses digit numbers: `0` through `100`
   - Parses written numbers: `"zero"` through `"twenty"`, `"twenty one"`, ..., `"one hundred"`
   - Parses ordinals: `"first"` (1), `"second"` (2), `"third"` (3), ..., `"tenth"` (10)

---

## 4. 4-Tier Application & PWA Resolution Engine

When a command references a window or application by name (e.g. `"move YouTube to 4"`, `"close Spotify"`), `AppRegistry` searches active Hyprland clients using a 4-tier waterfall:

```mermaid
flowchart TD
    A["Spoken App Name"] --> T1{"Tier 1: Exact Match?<br/>StartupWMClass or Desktop ID"}
    T1 -->|Found| S["Resolved Client Address"]
    T1 -->|No| T2{"Tier 2: User Alias Match?<br/>config.toml [aliases]"}
    T2 -->|Found| S
    T2 -->|No| T3{"Tier 3: Title Token Match?<br/>Split on '-', '|', '•'"}
    T3 -->|Found| S
    T3 -->|No| T4{"Tier 4: Bounded Levenshtein?<br/>Length >= 5, Distance <= 1"}
    T4 -->|Found| S
    T4 -->|No| F["Client Not Found / No Match"]
```

1. **Tier 1 (Exact Class / Desktop ID):** Matches the client's `class` or `initialClass` directly against harvested `.desktop` entry IDs or `StartupWMClass`.
2. **Tier 2 (User Aliases):** Matches user mappings declared in `~/.config/reflex/config.toml` (e.g., `"youtube"` $\rightarrow$ `"brave-agimnkijcaahngcdmfeangaknmldooml-Default"`).
3. **Tier 3 (Isolated Title Tokens):** Splits window titles by delimiter tokens (`-`, `|`, `•`) and checks for exact token matches (e.g., title `"YouTube - Brave"` contains token `"youtube"`).
4. **Tier 4 (Bounded Fuzzy Levenshtein):** Tolerates 1 character typo or phoneme drift for words of length $\ge 5$ (edit distance $\le 1$).

---

## 5. In-Memory Safety Guardrails

Reflex implements deterministic in-memory safety state machines (`GuardrailEngine`) to prevent hearing damage and accidental data loss:

| Guardrail Policy | Threshold / Limit | Behavior when Exceeded | Feedback |
| :--- | :--- | :--- | :--- |
| **Volume Velocity Rate Limit** | `+15%` within any `500ms` window | Request is clamped to `current_volume + 15%`. Returns status `RejectedByPolicy`. | Audible `Warning` earcon + Amber `ShieldAlert` HUD pill. |
| **Volume Hard Ceiling** | `80%` (0 dB maximum gain) | Request exceeding 80% is clamped to 80%. Returns status `RejectedByPolicy`. | Audible `Warning` earcon + Amber `ShieldAlert` HUD pill. |
| **Protected Window Allowlist** | `kitty`, `neovim`, `nvim`, `brave`, `code`, `vscode`, `antigravity` | `"close window"` does **not** close window immediately. Window address is pinned and state transitions to `AwaitingConfirmation`. | Audible `ConfirmationChime` + Coral `Lock` HUD pill. |
| **Pinned Address Two-Stage Confirmation** | `4.0 seconds` expiration window | Saying `"confirm"` within 4.0s closes the **pinned** address (`0x...`), preventing misdirected closes if focus shifts. If 4.0s lapses, confirmation expires. | Audible `Success` click on confirm; `Error` buzz on expiration. |

---

## 6. Synthesized Low-Latency Audio Feedback (Earcons)

Reflex generates 16-bit 48kHz mono PCM audio samples entirely in-memory using mathematical wave functions, requiring zero external audio files, zero disk access, and zero codec decompression overhead.

| Earcon Kind | Waveform & Frequency | Duration | Perceptual Meaning |
| :--- | :--- | :--- | :--- |
| **Success** | 440 Hz sinusoidal click with exponential decay ($e^{-150t}$) | 15 ms | Command acknowledged and dispatched successfully. |
| **Warning** | Double 160 Hz pulse ($e^{-80t}$) with 10ms silence gap | 40 ms | Safety guardrail engaged (volume clamped to limit). |
| **Confirmation Chime** | Ascending two-tone chime: 523.25 Hz ($C_5$) $\rightarrow$ 659.25 Hz ($E_5$) | 100 ms | Protected window close requested; awaiting `"confirm"`. |
| **Shutter** | High-frequency 1200 Hz burst with fast decay | 30 ms | Screenshot captured and copied to clipboard. |
| **Error** | Low-frequency 120 Hz buzz with slow decay | 60 ms | Command failed or confirmation window expired. |

---

## 7. Real-Time Wayland HUD Pill Visual Feedback (`reflex-hud`)

Reflex features a dedicated, pure Rust Wayland layer-shell floating pill overlay matching Hex's obsidian glass aesthetics. It completely replaces OS desktop notifications with zero-latency visual feedback on `Layer::Overlay`:

### Visual Tone Palette & Vector Icons

| Tone | Hex Color | Semantic Meaning | Lucide React Vector Icons Used | Examples |
| :--- | :--- | :--- | :--- | :--- |
| **Emerald** | `#10B981` | Success / Deterministic Action | `Volume2`, `Volume1`, `Check`, `Camera`, `Clipboard`, `Headphones`, `Speaker` | Volume levels (`Volume 50%`), Unmute, Screenshots, Audio port switched |
| **Cyan** | `#38BDF8` | Navigation / Workspace / Window | `Layers`, `Move`, `Pin`, `Maximize2`, `Zap` | Workspace changes (`Workspace 3`), Focus moves (`Focus Left`), Window launches |
| **Amber** | `#F59E0B` | Warning / Guardrail Enforcement | `ShieldAlert`, `AlertTriangle` | Velocity clamped (`Volume Clamped`), Ceiling enforced (80% limit) |
| **Coral** | `#FB7185` | Safety Confirmation / Protected Target | `Lock`, `VolumeX`, `AlertTriangle` | Protected window close prompt (`Close Kitty? Say 'confirm'`), Audio Muted |
| **Purple** | `#C084FC` | Autonomous AI Fallback | `Sparkles` | AI reasoning (`Reflex AI: Thinking...`), AI speech summary delivery |

### Motion & Interaction Physics
- **Closed-Form Spring Propagator**: Uses exact analytical damped harmonic oscillator physics (`Spec::new(0.30, 0.15)` bounce and `Spec::new(0.30, 0.0)` fade) matching Hex's dynamic island.
- **In-Place Tactile Updates**: Consecutive actions (e.g. stepping volume 5 times) update the value in-place with a subtle tactile haptic pulse without bouncing off-screen.
- **100% Click-Through**: Empty input region ensures mouse clicks pass straight through to background windows.
- **Zero Idle CPU**: When not active, buffer clears to 100% transparency and render ticks sleep.

---

## 8. Tri-State Client Exit Contract & Fallback Architecture

The `reflex` CLI communicates with `reflexd` via UNIX domain socket (`$XDG_RUNTIME_DIR/reflex.sock`) using single-line NDJSON frames. It exits immediately with one of three deterministic status codes:

```mermaid
flowchart LR
    A["Voice Input (F7 Push-to-Talk)"] --> B["reflex '<transcript>'"]
    B -->|Exit 0| C["Done (Action Executed in <1ms)"]
    B -->|Exit 1| D["Done (Guardrail Blocked / Audio Warning)"]
    B -->|Exit 2| E["PassThrough -> reflex-agent-fallback"]
    E --> F["Autonomous AI Agent (agy + Reflex HUD Pill)"]
```

- **Exit `0` (`Success`):** Command was recognized, validated by safety policy, and executed.
- **Exit `1` (`RejectedByPolicy` / `Error`):** Command was blocked or clamped by guardrails, or execution failed.
- **Exit `2` (`PassThrough`):** Transcript was not recognized by deterministic grammar. Automatically routes to `reflex-agent-fallback`:
  1. **Instant Acknowledgment:** Emits a purple `Sparkles` HUD pill (*"Reflex AI: Thinking..."*) over `$XDG_RUNTIME_DIR/reflex-hud.sock`.
  2. **Headless Execution:** Runs `agy --effort low -c --print "<prompt>"` for autonomous reasoning.
  3. **Visual Result Delivery:** Updates the HUD pill in-place with the concise text answer.
  4. **Neural Voice Output:** Synthesizes and speaks the response via `edge-tts` and PipeWire `pw-play`.

---

## 9. Configuration File Reference (`~/.config/reflex/config.toml`)

```toml
# Reflex Configuration: ~/.config/reflex/config.toml

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
# Custom aliases mapping user-spoken terms to cryptic PWA or desktop window classes
youtube   = "brave-agimnkijcaahngcdmfeangaknmldooml-Default"
whatsapp  = "brave-hnpfjngllnobngcgfapefoaidbinmjnm-Default"
instagram = "brave-maonjhobmgipjnlalmgmflffnflfljbl-Default"
studio    = "jetbrains-studio"
idea      = "jetbrains-idea"
```

---

## 10. Service Management & System Diagnostics

```bash
# Check daemon & HUD overlay status
systemctl --user status reflexd.service reflex-hud.service

# View live daemon & HUD logs
journalctl --user -u reflexd.service -u reflex-hud.service -f

# Restart services (e.g. after updating configuration or binaries)
systemctl --user restart reflexd.service reflex-hud.service

# Live latency benchmark test
target/release/examples/latency_benchmark
```
