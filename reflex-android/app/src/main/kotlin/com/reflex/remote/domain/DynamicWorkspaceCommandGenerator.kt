package com.reflex.remote.domain

import com.reflex.remote.data.model.CommandCategory
import com.reflex.remote.data.model.DesktopContext
import com.reflex.remote.data.model.ReflexCommandItem

/**
 * Dynamic generator for contextual workspace and window commands.
 * Completely replaces static hardcoded workspace numbers or app names by
 * inspecting live Hyprland desktop context (active window, active workspace,
 * all open workspaces, and all open windows).
 */
object DynamicWorkspaceCommandGenerator {

    fun generateCommands(context: DesktopContext?): List<ReflexCommandItem> {
        if (context == null || context.workspaces.isEmpty()) {
            return emptyList()
        }

        val dynamicItems = mutableListOf<ReflexCommandItem>()
        val activeWsId = context.activeWorkspace
        val activeWindow = context.activeWindow
        val allWsIds = context.workspaces.map { it.id }.toSet()

        // 1. Switch to other active workspaces
        for (ws in context.workspaces) {
            if (!ws.isActive && ws.id != activeWsId) {
                val appSummary = if (ws.windows.isNotEmpty()) {
                    ws.windows.joinToString(", ") { it.appName }.take(22)
                } else {
                    "Empty"
                }
                dynamicItems.add(
                    ReflexCommandItem(
                        id = "dyn_switch_ws_${ws.id}",
                        title = "Switch to Workspace ${ws.id} ($appSummary)",
                        command = "workspace ${ws.id}",
                        category = CommandCategory.WINDOW,
                        description = "Switch active desktop focus to Workspace ${ws.id} ($appSummary)",
                        aliases = listOf(
                            "workspace ${ws.id}",
                            "go to workspace ${ws.id}",
                            "switch to ${ws.id}",
                            "switch workspace ${ws.id}",
                            "desktop ${ws.id}"
                        )
                    )
                )
            }
        }

        // 2. Focus specific background apps across all workspaces
        for (win in context.openWindows) {
            if (!win.isActive && win.appName.isNotBlank()) {
                val appClean = win.appName.replace(" ", "")
                val focusTarget = win.appName.lowercase().takeWhile { it != '(' }.trim()
                dynamicItems.add(
                    ReflexCommandItem(
                        id = "dyn_focus_${appClean.lowercase()}",
                        title = "Focus ${win.appName} (WS ${win.workspace})",
                        command = "focus $focusTarget",
                        category = CommandCategory.WINDOW,
                        description = "Bring ${win.appName} into focus on Workspace ${win.workspace}",
                        aliases = listOf(
                            "focus $focusTarget",
                            "switch to $focusTarget",
                            "open $focusTarget",
                            "show $focusTarget"
                        )
                    )
                )
            }
        }

        // 3. Move active window to other open workspaces or new workspace
        val activeAppName = activeWindow.appName.ifBlank {
            context.openWindows.firstOrNull { it.isActive }?.appName ?: ""
        }
        val winDisplay = if (activeAppName.isNotBlank()) {
            activeAppName.replaceFirstChar { it.uppercase() }
        } else {
            "Active Window"
        }

        for (ws in context.workspaces) {
            if (!ws.isActive && ws.id != activeWsId) {
                val targetSummary = if (ws.windows.isNotEmpty()) ws.windows.first().appName else "empty"
                dynamicItems.add(
                    ReflexCommandItem(
                        id = "dyn_move_to_ws_${ws.id}",
                        title = "Move $winDisplay to WS ${ws.id}",
                        command = "move to ${ws.id}",
                        category = CommandCategory.WINDOW,
                        description = "Move $winDisplay to Workspace ${ws.id} ($targetSummary) without shifting focus",
                        aliases = listOf(
                            "move to ${ws.id}",
                            "send to ${ws.id}",
                            "move window to ${ws.id}",
                            "move to workspace ${ws.id}"
                        )
                    )
                )
            }
        }

        // 4. Move to fresh empty workspace
        val firstUnusedWs = (1..10).firstOrNull { it !in allWsIds } ?: (allWsIds.maxOrNull()?.plus(1) ?: 3)
        dynamicItems.add(
            ReflexCommandItem(
                id = "dyn_move_to_new_ws",
                title = "Move $winDisplay to New WS $firstUnusedWs",
                command = "move to $firstUnusedWs",
                category = CommandCategory.WINDOW,
                description = "Move active window to a fresh empty workspace ($firstUnusedWs)",
                aliases = listOf(
                    "move to new workspace",
                    "move to $firstUnusedWs",
                    "send to new workspace",
                    "move to empty workspace"
                )
            )
        )

        return dynamicItems
    }
}
