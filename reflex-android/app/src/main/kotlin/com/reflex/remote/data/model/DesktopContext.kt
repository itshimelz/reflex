package com.reflex.remote.data.model

data class ActiveWindowContext(
    val className: String = "",
    val title: String = "",
    val appName: String = "",
    val workspace: Int? = null
)

data class DesktopWindow(
    val address: String = "",
    val className: String = "",
    val title: String = "",
    val appName: String = "",
    val workspace: Int = 1,
    val isActive: Boolean = false
)

data class WorkspaceState(
    val id: Int = 1,
    val name: String = "1",
    val isActive: Boolean = false,
    val windowCount: Int = 0,
    val windows: List<DesktopWindow> = emptyList()
)

data class MediaContext(
    val player: String = "",
    val title: String = "",
    val artist: String = "",
    val status: String = "Stopped",
    val positionSeconds: Float? = null,
    val durationSeconds: Float? = null,
    val artUrl: String? = null
) {
    val isPlaying: Boolean
        get() = status.equals("Playing", ignoreCase = true)
}

data class VolumeContext(
    val percent: Int = 50,
    val isMuted: Boolean = false
)

data class DesktopContext(
    val activeWindow: ActiveWindowContext = ActiveWindowContext(),
    val media: MediaContext = MediaContext(),
    val volume: VolumeContext = VolumeContext(),
    val activeWorkspace: Int = activeWindow.workspace ?: 1,
    val workspaces: List<WorkspaceState> = emptyList(),
    val openWindows: List<DesktopWindow> = emptyList()
)
