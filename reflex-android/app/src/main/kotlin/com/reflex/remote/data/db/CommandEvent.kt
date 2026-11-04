package com.reflex.remote.data.db

data class CommandEvent(
    val id: Long = 0,
    val commandText: String,
    val commandId: String = "",
    val appName: String = "",
    val windowTitle: String = "",
    val windowClass: String = "",
    val workspace: String = "",
    val isMediaPlaying: Boolean = false,
    val mediaPlayer: String = "",
    val mediaTitle: String = "",
    val previousCommand: String = "",
    val timestamp: Long = System.currentTimeMillis(),
    val source: String = "android"
)
