package com.reflex.remote.data.model

enum class CommandCategory(val label: String, val shortName: String, val jsonFileName: String) {
    ALL("All Commands", "All", ""),
    AUDIO("Audio & Volume", "Audio", "audio.json"),
    MEDIA("Media Playback", "Media", "media.json"),
    FILES("File Operations", "Files", "files.json"),
    CONVERT("Format & Media Convert", "Convert", "convert.json"),
    METADATA("Metadata & Checksums", "Metadata", "metadata.json"),
    WINDOW("Window & Workspaces", "Windows", "window.json"),
    SYSTEM("System & Desktop", "System", "system.json")
}

data class ReflexCommandItem(
    val id: String,
    val title: String,
    val command: String,
    val category: CommandCategory,
    val description: String,
    val aliases: List<String> = emptyList(),
    val iconRes: String = "file"
)
