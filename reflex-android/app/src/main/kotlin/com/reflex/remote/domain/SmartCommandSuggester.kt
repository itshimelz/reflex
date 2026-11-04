package com.reflex.remote.domain

import com.reflex.remote.data.model.DesktopContext
import com.reflex.remote.data.model.ReflexCommandItem

data class ContextSuggestionResult(
    val contextBadge: String,
    val contextDescription: String,
    val commands: List<ReflexCommandItem>
)

object SmartCommandSuggester {

    fun suggestCommands(
        context: DesktopContext?,
        allCommands: List<ReflexCommandItem>
    ): ContextSuggestionResult {
        if (allCommands.isEmpty()) {
            return ContextSuggestionResult(
                contextBadge = "STANDBY",
                contextDescription = "Awaiting desktop telemetry",
                commands = emptyList()
            )
        }

        val commandsById = allCommands.associateBy { it.id }

        if (context == null) {
            val defaultIds = listOf(
                "volume_up", "volume_down", "toggle_mute", "media_toggle",
                "screenshot_window", "type_clipboard", "zip_selected", "open_terminal"
            )
            return ContextSuggestionResult(
                contextBadge = "DEFAULT SHORTCUTS",
                contextDescription = "General quick desktop actions",
                commands = defaultIds.mapNotNull { commandsById[it] }
            )
        }

        val media = context.media
        val window = context.activeWindow
        val appName = window.appName.lowercase()
        val windowTitle = window.title.lowercase()
        val windowClass = window.className.lowercase()

        // 1. Is media actively playing or has track loaded?
        if (media.isPlaying) {
            val playerLabel = cleanPlayerLabel(media.player)
            val trackInfo = if (media.title.isNotBlank()) {
                if (media.artist.isNotBlank()) "${media.title} • ${media.artist}" else media.title
            } else "Playback Active"

            val ids = mutableListOf(
                "media_toggle",
                "media_next",
                "media_prev",
                "media_seek_fwd",
                "volume_up",
                "volume_down"
            )
            if (media.player.contains("brave", ignoreCase = true) || appName == "brave") {
                ids.add("mute_brave")
            } else if (media.player.contains("spotify", ignoreCase = true) || appName == "spotify") {
                ids.add("mute_spotify")
            }
            ids.add("toggle_mute")

            return ContextSuggestionResult(
                contextBadge = "PLAYING • $playerLabel",
                contextDescription = trackInfo,
                commands = ids.mapNotNull { commandsById[it] }
            )
        }

        // 2. Is a File Manager active?
        if (appName == "dolphin" || windowClass.contains("dolphin") || appName in listOf("nautilus", "thunar", "nemo", "pcmanfm")) {
            val ids = listOf(
                "zip_selected",
                "unzip_selected",
                "duplicate_selected",
                "clean_filename",
                "timestamp_filename",
                "convert_webp",
                "convert_mp3",
                "copy_full_path",
                "sha256_checksum",
                "trash_selected",
                "open_terminal"
            )
            return ContextSuggestionResult(
                contextBadge = "FILES • DOLPHIN",
                contextDescription = if (window.title.isNotBlank()) window.title else "Selected file operations active",
                commands = ids.mapNotNull { commandsById[it] }
            )
        }

        // 3. Is a Terminal active?
        if (appName in listOf("kitty", "terminal", "alacritty", "wezterm", "foot", "konsole") || windowClass.contains("kitty")) {
            val ids = listOf(
                "type_clipboard",
                "strip_clipboard",
                "clear_clipboard",
                "screenshot_window",
                "confirm_action",
                "close_window",
                "toggle_fullscreen"
            )
            val termName = if (appName.isNotBlank()) appName.uppercase().take(8) else "TERMINAL"
            return ContextSuggestionResult(
                contextBadge = "TERMINAL • $termName",
                contextDescription = if (window.title.isNotBlank()) window.title else "Terminal & clipboard injection ready",
                commands = ids.mapNotNull { commandsById[it] }
            )
        }

        // 4. Is an IDE or Code Editor active?
        if (appName in listOf("code", "editor", "nvim", "neovim", "antigravity", "idea", "studio") || windowTitle.contains("code") || windowTitle.contains("antigravity")) {
            val ids = listOf(
                "type_clipboard",
                "screenshot_window",
                "toggle_floating",
                "toggle_fullscreen",
                "toggle_split",
                "focus_direction",
                "close_window"
            )
            val editorName = if (appName.isNotBlank()) appName.uppercase().take(8) else "EDITOR"
            return ContextSuggestionResult(
                contextBadge = "EDITOR • $editorName",
                contextDescription = if (window.title.isNotBlank()) window.title else "Code editing & layout focus",
                commands = ids.mapNotNull { commandsById[it] }
            )
        }

        // 5. Is a Web Browser active?
        if (appName in listOf("brave", "browser", "chromium", "chrome", "firefox") || windowTitle.contains("youtube") || windowTitle.contains("brave")) {
            val ids = listOf(
                "media_toggle",
                "scroll_down",
                "scroll_up",
                "focus_search",
                "screenshot_window",
                "mute_brave",
                "volume_up",
                "volume_down"
            )
            val browserName = if (appName.isNotBlank()) appName.uppercase().take(8) else "BROWSER"
            return ContextSuggestionResult(
                contextBadge = "BROWSER • $browserName",
                contextDescription = if (window.title.isNotBlank()) window.title else "Web navigation & media ready",
                commands = ids.mapNotNull { commandsById[it] }
            )
        }

        // 6. Default fallback
        val defaultIds = listOf(
            "volume_up",
            "volume_down",
            "toggle_mute",
            "media_toggle",
            "screenshot_window",
            "type_clipboard",
            "zip_selected",
            "open_terminal"
        )
        val appLabel = if (appName.isNotBlank()) appName.uppercase().take(8) else "DESKTOP"
        return ContextSuggestionResult(
            contextBadge = "FOCUS • $appLabel",
            contextDescription = if (window.title.isNotBlank()) window.title else "Ready for voice and quick commands",
            commands = defaultIds.mapNotNull { commandsById[it] }
        )
    }

    private fun cleanPlayerLabel(player: String): String {
        val lower = player.lowercase().trim()
        return when {
            lower.contains("plasma-browser") || lower.contains("browser") -> "BROWSER"
            lower.contains("spotify") -> "SPOTIFY"
            lower.contains("chrome") -> "CHROME"
            lower.contains("firefox") -> "FIREFOX"
            lower.contains("brave") -> "BRAVE"
            lower.contains("vlc") -> "VLC"
            lower.contains("mpv") -> "MPV"
            lower.isBlank() -> "MEDIA"
            else -> player.uppercase().take(8)
        }
    }
}
