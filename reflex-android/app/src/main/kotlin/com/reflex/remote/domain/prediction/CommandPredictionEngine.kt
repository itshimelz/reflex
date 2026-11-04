package com.reflex.remote.domain.prediction

import com.reflex.remote.data.db.ICommandHistoryRepository
import com.reflex.remote.data.model.CommandCategory
import com.reflex.remote.data.model.DesktopContext
import com.reflex.remote.data.model.ReflexCommandItem
import kotlin.math.exp

data class PredictedCommand(
    val commandItem: ReflexCommandItem,
    val score: Float,
    val confidence: Float,
    val reason: String
)

data class PredictionResult(
    val contextSummary: String,
    val predictedCommands: List<PredictedCommand>,
    val rankedCatalog: List<ReflexCommandItem>,
    val prioritizedCategories: List<CommandCategory>
)

object CommandPredictionEngine {

    suspend fun predict(
        currentContext: DesktopContext?,
        lastExecutedCommand: String?,
        allCommands: List<ReflexCommandItem>,
        repository: ICommandHistoryRepository
    ): PredictionResult {
        if (allCommands.isEmpty()) {
            return PredictionResult(
                contextSummary = "Standby • No commands loaded",
                predictedCommands = emptyList(),
                rankedCatalog = emptyList(),
                prioritizedCategories = CommandCategory.values().toList()
            )
        }

        val commandsByText = allCommands.associateBy { it.command.lowercase().trim() }
        val commandsById = allCommands.associateBy { it.id }

        // Determine effective last command (passed or fetched from DB)
        val effectiveLastCommand = lastExecutedCommand?.ifBlank { null }
            ?: repository.getLastCommand()?.commandText

        // Gather database metrics
        val transitions = if (!effectiveLastCommand.isNullOrBlank()) {
            repository.getTransitionCounts(effectiveLastCommand)
        } else {
            emptyMap()
        }

        val appName = currentContext?.activeWindow?.appName?.lowercase()?.trim() ?: ""
        val windowClass = currentContext?.activeWindow?.className?.lowercase()?.trim() ?: ""
        val workspace = currentContext?.activeWindow?.workspace?.toString() ?: ""
        val isMediaPlaying = currentContext?.media?.isPlaying == true

        val appFrequencies = if (appName.isNotBlank()) {
            repository.getAppCommandFrequencies(appName)
        } else {
            emptyMap()
        }

        val mediaFrequencies = repository.getMediaCommandFrequencies(isMediaPlaying)
        val workspaceFrequencies = if (workspace.isNotBlank()) {
            repository.getWorkspaceCommandFrequencies(workspace)
        } else {
            emptyMap()
        }

        val recentEvents = repository.getRecentEvents(25)
        val now = System.currentTimeMillis()

        // Score each catalog command
        val scoredItems = allCommands.map { item ->
            val cmdKey = item.command.lowercase().trim()
            val cmdId = item.id

            // 1. Sequence Transition Probability (Weight = 3.5)
            val dbTransitionCount = transitions[cmdKey] ?: transitions[cmdId] ?: 0
            val heuristicTransitionPrior = getTransitionPrior(effectiveLastCommand, item)
            val transitionScore = (dbTransitionCount * 4.0f) + (heuristicTransitionPrior * 2.5f)

            // 2. Application Context Affinity (Weight = 2.8)
            val dbAppCount = appFrequencies[cmdKey] ?: appFrequencies[cmdId] ?: 0
            val heuristicAppPrior = getAppPrior(appName, windowClass, item)
            val appScore = (dbAppCount * 2.5f) + (heuristicAppPrior * 2.0f)

            // 3. Media State Affinity (Weight = 2.5)
            val dbMediaCount = mediaFrequencies[cmdKey] ?: mediaFrequencies[cmdId] ?: 0
            val heuristicMediaPrior = getMediaPrior(isMediaPlaying, item)
            val mediaScore = (dbMediaCount * 2.0f) + (heuristicMediaPrior * 2.5f)

            // 4. Workspace Affinity (Weight = 1.2)
            val dbWsCount = workspaceFrequencies[cmdKey] ?: workspaceFrequencies[cmdId] ?: 0
            val wsScore = dbWsCount * 1.5f

            // 5. Recency / Momentum Score (Exponential time-decay over 20 minutes)
            var recencyScore = 0f
            for (event in recentEvents) {
                if (event.commandText == cmdKey || event.commandId == cmdId) {
                    val ageMins = ((now - event.timestamp) / 60000f).coerceAtLeast(0f)
                    recencyScore += exp(-ageMins / 20f) * 2.0f
                }
            }

            val totalScore = transitionScore + appScore + mediaScore + wsScore + recencyScore

            // Determine dominant factor for human reason tag
            val reason = when {
                transitionScore > 3.0f && transitionScore >= appScore && transitionScore >= mediaScore -> {
                    if (effectiveLastCommand != null) "Follows \"$effectiveLastCommand\"" else "Sequence Prediction"
                }
                mediaScore > 3.0f && mediaScore >= appScore -> {
                    if (isMediaPlaying) "Active Playback Control" else "Media Action"
                }
                appScore > 3.0f -> {
                    val displayApp = if (appName.isNotBlank()) appName.replaceFirstChar { it.uppercase() } else "Focused App"
                    "Frequent in $displayApp"
                }
                wsScore > 2.0f -> "Workspace $workspace Action"
                recencyScore > 1.5f -> "Recent Workflow"
                heuristicMediaPrior > 1.5f -> "Media Shortcut"
                heuristicAppPrior > 1.5f -> "App Shortcut"
                else -> "Quick Action"
            }

            val confidence = (totalScore / (totalScore + 10f)).coerceIn(0.1f, 0.99f)
            PredictedCommand(
                commandItem = item,
                score = totalScore,
                confidence = confidence,
                reason = reason
            )
        }

        // Sort predicted items descending by score
        val sortedPredictions = scoredItems.sortedByDescending { it.score }

        // Top 8 high-scoring distinct predicted commands
        val topPredicted = sortedPredictions
            .filter { it.score > 1.5f }
            .take(8)
            .ifEmpty { sortedPredictions.take(6) }

        // Dynamically ranked catalog (entire list ordered by contextual relevance)
        val rankedCatalog = sortedPredictions.map { it.commandItem }

        // Prioritized categories
        val prioritizedCategories = getPrioritizedCategories(appName, isMediaPlaying)

        // Context summary label
        val contextSummary = buildContextSummary(appName, workspace, isMediaPlaying, effectiveLastCommand)

        return PredictionResult(
            contextSummary = contextSummary,
            predictedCommands = topPredicted,
            rankedCatalog = rankedCatalog,
            prioritizedCategories = prioritizedCategories
        )
    }

    private fun getTransitionPrior(lastCmd: String?, candidate: ReflexCommandItem): Float {
        if (lastCmd.isNullOrBlank()) return 0f
        val normLast = lastCmd.lowercase().trim()
        val candCmd = candidate.command.lowercase().trim()
        val candId = candidate.id

        return when {
            // After Mute -> Unmute, Volume, Media
            normLast == "mute" || normLast == "toggle_mute" || normLast.contains("mute") -> when {
                candCmd == "unmute" || candId == "toggle_mute" -> 5.0f
                candCmd.startsWith("volume") -> 3.5f
                candId == "media_toggle" -> 3.0f
                else -> 0f
            }

            // After Unmute -> Mute, Volume
            normLast == "unmute" -> when {
                candCmd == "mute" || candId == "toggle_mute" -> 4.5f
                candCmd.startsWith("volume") -> 3.5f
                else -> 0f
            }

            // After Media Next -> Media Next, Media Prev, Media Toggle
            normLast == "media next" || candId == "media_next" -> when {
                candCmd == "media next" -> 4.0f
                candCmd == "media prev" -> 3.5f
                candId == "media_toggle" -> 3.0f
                candCmd.startsWith("volume") -> 2.5f
                else -> 0f
            }

            // After Media Toggle -> Media Next, Volume
            normLast == "media toggle" || normLast == "play" || normLast == "pause" -> when {
                candCmd == "media next" -> 3.5f
                candCmd.startsWith("volume") -> 3.0f
                candCmd == "toggle mute" -> 2.5f
                else -> 0f
            }

            // After Zip Selected -> Copy Path, Unzip, Open Terminal
            normLast.contains("zip") -> when {
                candCmd.contains("unzip") -> 4.5f
                candCmd.contains("path") -> 4.0f
                candCmd.contains("terminal") -> 3.5f
                else -> 0f
            }

            // After Screenshot -> Clipboard, Close
            normLast.contains("screenshot") -> when {
                candCmd.contains("clipboard") -> 4.5f
                candCmd.contains("close") -> 3.0f
                else -> 0f
            }

            // After Convert WebP/MP3 -> Clean filename, Duplicate
            normLast.contains("convert") -> when {
                candCmd.contains("clean") -> 4.0f
                candCmd.contains("duplicate") -> 3.5f
                candCmd.contains("path") -> 3.0f
                else -> 0f
            }

            // After Volume -> Volume adjustments
            normLast.contains("volume") -> when {
                candCmd.contains("volume") -> 3.5f
                candCmd.contains("mute") -> 3.0f
                else -> 0f
            }

            else -> 0f
        }
    }

    private fun getAppPrior(appName: String, windowClass: String, candidate: ReflexCommandItem): Float {
        val app = appName.lowercase()
        val cls = windowClass.lowercase()
        val cat = candidate.category
        val id = candidate.id

        return when {
            app == "dolphin" || cls.contains("dolphin") || app in listOf("nautilus", "thunar", "nemo") -> {
                when (cat) {
                    CommandCategory.FILES -> 4.5f
                    CommandCategory.CONVERT -> 4.0f
                    CommandCategory.METADATA -> 3.5f
                    else -> 0f
                }
            }

            app in listOf("kitty", "alacritty", "wezterm", "terminal", "foot", "konsole") || cls.contains("kitty") -> {
                when {
                    id in listOf("type_clipboard", "strip_clipboard", "clear_clipboard") -> 5.0f
                    id in listOf("screenshot_window", "confirm_action", "close_window") -> 3.5f
                    cat == CommandCategory.WINDOW -> 3.0f
                    else -> 0f
                }
            }

            app in listOf("code", "nvim", "editor", "antigravity", "idea", "studio") -> {
                when {
                    id in listOf("type_clipboard", "screenshot_window", "toggle_floating", "toggle_split") -> 4.5f
                    cat == CommandCategory.WINDOW -> 3.5f
                    else -> 0f
                }
            }

            app in listOf("brave", "chrome", "firefox", "browser") || cls.contains("brave") -> {
                when {
                    id in listOf("media_toggle", "scroll_down", "scroll_up", "mute_brave") -> 4.5f
                    cat == CommandCategory.MEDIA -> 3.5f
                    cat == CommandCategory.AUDIO -> 3.0f
                    else -> 0f
                }
            }

            app.contains("spotify") -> {
                when (cat) {
                    CommandCategory.MEDIA -> 5.0f
                    CommandCategory.AUDIO -> 4.5f
                    else -> 0f
                }
            }

            else -> 0f
        }
    }

    private fun getMediaPrior(isMediaPlaying: Boolean, candidate: ReflexCommandItem): Float {
        val cat = candidate.category
        val id = candidate.id

        return if (isMediaPlaying) {
            when {
                id in listOf("media_toggle", "media_next", "media_prev", "media_seek_fwd") -> 5.0f
                cat == CommandCategory.MEDIA -> 4.5f
                id in listOf("volume_up", "volume_down", "toggle_mute") -> 4.0f
                cat == CommandCategory.AUDIO -> 3.5f
                else -> 0f
            }
        } else {
            when (id) {
                "media_toggle" -> 2.5f
                "volume_up", "volume_down" -> 1.5f
                else -> 0f
            }
        }
    }

    private fun getPrioritizedCategories(appName: String, isMediaPlaying: Boolean): List<CommandCategory> {
        val app = appName.lowercase()
        return when {
            isMediaPlaying || app in listOf("spotify", "youtube") -> listOf(
                CommandCategory.ALL,
                CommandCategory.MEDIA,
                CommandCategory.AUDIO,
                CommandCategory.WINDOW,
                CommandCategory.FILES,
                CommandCategory.CONVERT,
                CommandCategory.METADATA,
                CommandCategory.SYSTEM
            )
            app in listOf("dolphin", "nautilus", "thunar") -> listOf(
                CommandCategory.ALL,
                CommandCategory.FILES,
                CommandCategory.CONVERT,
                CommandCategory.METADATA,
                CommandCategory.WINDOW,
                CommandCategory.AUDIO,
                CommandCategory.MEDIA,
                CommandCategory.SYSTEM
            )
            app in listOf("kitty", "alacritty", "terminal", "code", "nvim", "antigravity") -> listOf(
                CommandCategory.ALL,
                CommandCategory.WINDOW,
                CommandCategory.SYSTEM,
                CommandCategory.FILES,
                CommandCategory.AUDIO,
                CommandCategory.MEDIA,
                CommandCategory.CONVERT,
                CommandCategory.METADATA
            )
            else -> listOf(
                CommandCategory.ALL,
                CommandCategory.AUDIO,
                CommandCategory.MEDIA,
                CommandCategory.WINDOW,
                CommandCategory.FILES,
                CommandCategory.CONVERT,
                CommandCategory.METADATA,
                CommandCategory.SYSTEM
            )
        }
    }

    private fun buildContextSummary(
        appName: String,
        workspace: String,
        isMediaPlaying: Boolean,
        lastCommand: String?
    ): String {
        val appDisplay = if (appName.isNotBlank()) appName.replaceFirstChar { it.uppercase() } else "Desktop"
        val wsDisplay = if (workspace.isNotBlank()) "WS $workspace" else ""
        val mediaDisplay = if (isMediaPlaying) "Playing Media" else ""

        val contextParts = listOf(appDisplay, wsDisplay, mediaDisplay).filter { it.isNotBlank() }
        val prefix = contextParts.joinToString(" • ")

        return if (!lastCommand.isNullOrBlank()) {
            "$prefix • Next after \"$lastCommand\""
        } else {
            prefix
        }
    }
}
