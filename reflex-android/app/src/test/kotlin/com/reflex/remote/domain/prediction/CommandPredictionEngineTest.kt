package com.reflex.remote.domain.prediction

import com.reflex.remote.data.db.CommandEvent
import com.reflex.remote.data.db.ICommandHistoryRepository
import com.reflex.remote.data.model.ActiveWindowContext
import com.reflex.remote.data.model.CommandCategory
import com.reflex.remote.data.model.DesktopContext
import com.reflex.remote.data.model.MediaContext
import com.reflex.remote.data.model.ReflexCommandItem
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class CommandPredictionEngineTest {

    private class FakeHistoryRepository : ICommandHistoryRepository {
        val events = mutableListOf<CommandEvent>()
        val transitions = mutableMapOf<String, MutableMap<String, Int>>()
        val appFreqs = mutableMapOf<String, MutableMap<String, Int>>()

        override suspend fun recordEvent(event: CommandEvent): Long {
            events.add(event)
            return events.size.toLong()
        }

        override suspend fun recordBatch(events: List<CommandEvent>) {
            this.events.addAll(events)
        }

        override suspend fun getLastCommand(): CommandEvent? = events.lastOrNull()

        override suspend fun getRecentEvents(limit: Int): List<CommandEvent> =
            events.reversed().take(limit)

        override suspend fun getTransitionCounts(prevCommand: String): Map<String, Int> =
            transitions[prevCommand.lowercase()] ?: emptyMap()

        override suspend fun getAppCommandFrequencies(appName: String): Map<String, Int> =
            appFreqs[appName.lowercase()] ?: emptyMap()

        override suspend fun getMediaCommandFrequencies(isPlaying: Boolean): Map<String, Int> =
            emptyMap()

        override suspend fun getWorkspaceCommandFrequencies(workspace: String): Map<String, Int> =
            emptyMap()

        override suspend fun clearHistory() {
            events.clear()
            transitions.clear()
            appFreqs.clear()
        }
    }

    private val sampleCommands = listOf(
        ReflexCommandItem(
            id = "media_play",
            title = "Play / Resume",
            command = "play",
            description = "Resume media",
            category = CommandCategory.MEDIA
        ),
        ReflexCommandItem(
            id = "media_pause",
            title = "Pause Playback",
            command = "pause",
            description = "Pause media",
            category = CommandCategory.MEDIA
        ),
        ReflexCommandItem(
            id = "audio_mute",
            title = "Mute Audio",
            command = "mute",
            description = "Mute audio",
            category = CommandCategory.AUDIO
        ),
        ReflexCommandItem(
            id = "audio_unmute",
            title = "Unmute Audio",
            command = "unmute",
            description = "Unmute audio",
            category = CommandCategory.AUDIO
        ),
        ReflexCommandItem(
            id = "file_compress",
            title = "Compress Selection",
            command = "compress",
            description = "Compress files",
            category = CommandCategory.FILES
        ),
        ReflexCommandItem(
            id = "window_close",
            title = "Close Window",
            command = "close",
            description = "Close focused window",
            category = CommandCategory.WINDOW
        )
    )

    @Test
    fun testEmptyCommandsReturnsEmpty() = runBlocking {
        val repo = FakeHistoryRepository()
        val result = CommandPredictionEngine.predict(
            currentContext = null,
            lastExecutedCommand = null,
            allCommands = emptyList(),
            repository = repo
        )

        assertTrue(result.predictedCommands.isEmpty())
        assertTrue(result.rankedCatalog.isEmpty())
        assertEquals("Standby • No commands loaded", result.contextSummary)
    }

    @Test
    fun testMediaPlayingPrioritizesMediaCommands() = runBlocking {
        val repo = FakeHistoryRepository()
        val context = DesktopContext(
            media = MediaContext(
                player = "spotify",
                title = "Song Title",
                artist = "Artist",
                status = "Playing"
            )
        )

        val result = CommandPredictionEngine.predict(
            currentContext = context,
            lastExecutedCommand = null,
            allCommands = sampleCommands,
            repository = repo
        )

        assertFalse(result.predictedCommands.isEmpty())
        val topPrediction = result.predictedCommands.first()
        // With media playing, pause or mute should be top ranked
        assertTrue(
            "Expected media command to be ranked first, got ${topPrediction.commandItem.command}",
            topPrediction.commandItem.category == CommandCategory.MEDIA ||
                    topPrediction.commandItem.category == CommandCategory.AUDIO
        )
        // Media categories should be placed upfront right after ALL
        assertEquals(CommandCategory.ALL, result.prioritizedCategories[0])
        assertEquals(CommandCategory.MEDIA, result.prioritizedCategories[1])
        assertEquals(CommandCategory.AUDIO, result.prioritizedCategories[2])
    }

    @Test
    fun testSequenceTransitionPrediction() = runBlocking {
        val repo = FakeHistoryRepository()
        // Simulate user executing mute followed by unmute 5 times
        repo.transitions["mute"] = mutableMapOf("unmute" to 5)

        val result = CommandPredictionEngine.predict(
            currentContext = null,
            lastExecutedCommand = "mute",
            allCommands = sampleCommands,
            repository = repo
        )

        val topPrediction = result.predictedCommands.first()
        assertEquals("unmute", topPrediction.commandItem.command)
        assertTrue(topPrediction.reason.contains("Follows \"mute\""))
    }

    @Test
    fun testAppAffinityPrioritization() = runBlocking {
        val repo = FakeHistoryRepository()
        val context = DesktopContext(
            activeWindow = ActiveWindowContext(
                appName = "dolphin",
                className = "org.kde.dolphin",
                title = "Downloads — Dolphin"
            )
        )

        val result = CommandPredictionEngine.predict(
            currentContext = context,
            lastExecutedCommand = null,
            allCommands = sampleCommands,
            repository = repo
        )

        // File category should be prioritized when focused in dolphin right after ALL
        assertEquals(CommandCategory.ALL, result.prioritizedCategories[0])
        assertEquals(CommandCategory.FILES, result.prioritizedCategories[1])
        val fileCmd = result.predictedCommands.firstOrNull { it.commandItem.category == CommandCategory.FILES }
        assertTrue("Expected file command in predictions", fileCmd != null)
    }
}
