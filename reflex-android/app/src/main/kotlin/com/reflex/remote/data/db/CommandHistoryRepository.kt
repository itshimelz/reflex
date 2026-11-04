package com.reflex.remote.data.db

import android.content.ContentValues
import android.content.Context
import android.database.Cursor
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

interface ICommandHistoryRepository {
    suspend fun recordEvent(event: CommandEvent): Long
    suspend fun recordBatch(events: List<CommandEvent>)
    suspend fun getLastCommand(): CommandEvent?
    suspend fun getRecentEvents(limit: Int = 50): List<CommandEvent>
    suspend fun getTransitionCounts(prevCommand: String): Map<String, Int>
    suspend fun getAppCommandFrequencies(appName: String): Map<String, Int>
    suspend fun getMediaCommandFrequencies(isPlaying: Boolean): Map<String, Int>
    suspend fun getWorkspaceCommandFrequencies(workspace: String): Map<String, Int>
    suspend fun clearHistory()
}

open class CommandHistoryRepository(context: Context) : ICommandHistoryRepository {

    private val dbHelper = CommandHistoryDbHelper(context)

    override suspend fun recordEvent(event: CommandEvent): Long = withContext(Dispatchers.IO) {
        val db = dbHelper.writableDatabase
        val values = ContentValues().apply {
            put(CommandHistoryDbHelper.COL_COMMAND_TEXT, event.commandText.trim().lowercase())
            put(CommandHistoryDbHelper.COL_COMMAND_ID, event.commandId)
            put(CommandHistoryDbHelper.COL_APP_NAME, event.appName.trim().lowercase())
            put(CommandHistoryDbHelper.COL_WINDOW_TITLE, event.windowTitle.trim())
            put(CommandHistoryDbHelper.COL_WINDOW_CLASS, event.windowClass.trim().lowercase())
            put(CommandHistoryDbHelper.COL_WORKSPACE, event.workspace)
            put(CommandHistoryDbHelper.COL_IS_MEDIA_PLAYING, if (event.isMediaPlaying) 1 else 0)
            put(CommandHistoryDbHelper.COL_MEDIA_PLAYER, event.mediaPlayer.trim().lowercase())
            put(CommandHistoryDbHelper.COL_MEDIA_TITLE, event.mediaTitle.trim())
            put(CommandHistoryDbHelper.COL_PREVIOUS_COMMAND, event.previousCommand.trim().lowercase())
            put(CommandHistoryDbHelper.COL_TIMESTAMP, event.timestamp)
            put(CommandHistoryDbHelper.COL_SOURCE, event.source)
        }
        db.insert(CommandHistoryDbHelper.TABLE_COMMAND_HISTORY, null, values)
    }

    override suspend fun recordBatch(events: List<CommandEvent>) = withContext(Dispatchers.IO) {
        if (events.isEmpty()) return@withContext
        val db = dbHelper.writableDatabase
        db.beginTransaction()
        try {
            for (event in events) {
                val values = ContentValues().apply {
                    put(CommandHistoryDbHelper.COL_COMMAND_TEXT, event.commandText.trim().lowercase())
                    put(CommandHistoryDbHelper.COL_COMMAND_ID, event.commandId)
                    put(CommandHistoryDbHelper.COL_APP_NAME, event.appName.trim().lowercase())
                    put(CommandHistoryDbHelper.COL_WINDOW_TITLE, event.windowTitle.trim())
                    put(CommandHistoryDbHelper.COL_WINDOW_CLASS, event.windowClass.trim().lowercase())
                    put(CommandHistoryDbHelper.COL_WORKSPACE, event.workspace)
                    put(CommandHistoryDbHelper.COL_IS_MEDIA_PLAYING, if (event.isMediaPlaying) 1 else 0)
                    put(CommandHistoryDbHelper.COL_MEDIA_PLAYER, event.mediaPlayer.trim().lowercase())
                    put(CommandHistoryDbHelper.COL_MEDIA_TITLE, event.mediaTitle.trim())
                    put(CommandHistoryDbHelper.COL_PREVIOUS_COMMAND, event.previousCommand.trim().lowercase())
                    put(CommandHistoryDbHelper.COL_TIMESTAMP, event.timestamp)
                    put(CommandHistoryDbHelper.COL_SOURCE, event.source)
                }
                db.insert(CommandHistoryDbHelper.TABLE_COMMAND_HISTORY, null, values)
            }
            db.setTransactionSuccessful()
        } finally {
            db.endTransaction()
        }
    }

    override suspend fun getRecentEvents(limit: Int): List<CommandEvent> = withContext(Dispatchers.IO) {
        val db = dbHelper.readableDatabase
        val list = mutableListOf<CommandEvent>()
        val cursor: Cursor = db.query(
            CommandHistoryDbHelper.TABLE_COMMAND_HISTORY,
            null,
            null,
            null,
            null,
            null,
            "${CommandHistoryDbHelper.COL_TIMESTAMP} DESC",
            limit.toString()
        )
        cursor.use { c ->
            val idIdx = c.getColumnIndexOrThrow(CommandHistoryDbHelper.COL_ID)
            val cmdIdx = c.getColumnIndexOrThrow(CommandHistoryDbHelper.COL_COMMAND_TEXT)
            val cmdIdIdx = c.getColumnIndexOrThrow(CommandHistoryDbHelper.COL_COMMAND_ID)
            val appIdx = c.getColumnIndexOrThrow(CommandHistoryDbHelper.COL_APP_NAME)
            val titleIdx = c.getColumnIndexOrThrow(CommandHistoryDbHelper.COL_WINDOW_TITLE)
            val classIdx = c.getColumnIndexOrThrow(CommandHistoryDbHelper.COL_WINDOW_CLASS)
            val wsIdx = c.getColumnIndexOrThrow(CommandHistoryDbHelper.COL_WORKSPACE)
            val mediaPlayIdx = c.getColumnIndexOrThrow(CommandHistoryDbHelper.COL_IS_MEDIA_PLAYING)
            val playerIdx = c.getColumnIndexOrThrow(CommandHistoryDbHelper.COL_MEDIA_PLAYER)
            val mediaTitleIdx = c.getColumnIndexOrThrow(CommandHistoryDbHelper.COL_MEDIA_TITLE)
            val prevIdx = c.getColumnIndexOrThrow(CommandHistoryDbHelper.COL_PREVIOUS_COMMAND)
            val timeIdx = c.getColumnIndexOrThrow(CommandHistoryDbHelper.COL_TIMESTAMP)
            val srcIdx = c.getColumnIndexOrThrow(CommandHistoryDbHelper.COL_SOURCE)

            while (c.moveToNext()) {
                list.add(
                    CommandEvent(
                        id = c.getLong(idIdx),
                        commandText = c.getString(cmdIdx),
                        commandId = c.getString(cmdIdIdx),
                        appName = c.getString(appIdx),
                        windowTitle = c.getString(titleIdx),
                        windowClass = c.getString(classIdx),
                        workspace = c.getString(wsIdx),
                        isMediaPlaying = c.getInt(mediaPlayIdx) == 1,
                        mediaPlayer = c.getString(playerIdx),
                        mediaTitle = c.getString(mediaTitleIdx),
                        previousCommand = c.getString(prevIdx),
                        timestamp = c.getLong(timeIdx),
                        source = c.getString(srcIdx)
                    )
                )
            }
        }
        list
    }

    override suspend fun getLastCommand(): CommandEvent? = withContext(Dispatchers.IO) {
        getRecentEvents(1).firstOrNull()
    }

    override suspend fun getTransitionCounts(prevCommand: String): Map<String, Int> = withContext(Dispatchers.IO) {
        val db = dbHelper.readableDatabase
        val map = mutableMapOf<String, Int>()
        val cleanFrom = prevCommand.trim().lowercase()
        if (cleanFrom.isBlank()) return@withContext emptyMap()

        val cursor = db.rawQuery(
            """
            SELECT ${CommandHistoryDbHelper.COL_COMMAND_TEXT}, COUNT(*) as cnt
            FROM ${CommandHistoryDbHelper.TABLE_COMMAND_HISTORY}
            WHERE ${CommandHistoryDbHelper.COL_PREVIOUS_COMMAND} = ?
            GROUP BY ${CommandHistoryDbHelper.COL_COMMAND_TEXT}
            ORDER BY cnt DESC
            """.trimIndent(),
            arrayOf(cleanFrom)
        )
        cursor.use { c ->
            while (c.moveToNext()) {
                map[c.getString(0)] = c.getInt(1)
            }
        }
        map
    }

    override suspend fun getAppCommandFrequencies(appName: String): Map<String, Int> = withContext(Dispatchers.IO) {
        val db = dbHelper.readableDatabase
        val map = mutableMapOf<String, Int>()
        val cleanApp = appName.trim().lowercase()
        if (cleanApp.isBlank()) return@withContext emptyMap()

        val cursor = db.rawQuery(
            """
            SELECT ${CommandHistoryDbHelper.COL_COMMAND_TEXT}, COUNT(*) as cnt
            FROM ${CommandHistoryDbHelper.TABLE_COMMAND_HISTORY}
            WHERE ${CommandHistoryDbHelper.COL_APP_NAME} = ? OR ${CommandHistoryDbHelper.COL_WINDOW_CLASS} LIKE ?
            GROUP BY ${CommandHistoryDbHelper.COL_COMMAND_TEXT}
            ORDER BY cnt DESC
            """.trimIndent(),
            arrayOf(cleanApp, "%$cleanApp%")
        )
        cursor.use { c ->
            while (c.moveToNext()) {
                map[c.getString(0)] = c.getInt(1)
            }
        }
        map
    }

    override suspend fun getMediaCommandFrequencies(isPlaying: Boolean): Map<String, Int> = withContext(Dispatchers.IO) {
        val db = dbHelper.readableDatabase
        val map = mutableMapOf<String, Int>()
        val cursor = db.rawQuery(
            """
            SELECT ${CommandHistoryDbHelper.COL_COMMAND_TEXT}, COUNT(*) as cnt
            FROM ${CommandHistoryDbHelper.TABLE_COMMAND_HISTORY}
            WHERE ${CommandHistoryDbHelper.COL_IS_MEDIA_PLAYING} = ?
            GROUP BY ${CommandHistoryDbHelper.COL_COMMAND_TEXT}
            ORDER BY cnt DESC
            """.trimIndent(),
            arrayOf(if (isPlaying) "1" else "0")
        )
        cursor.use { c ->
            while (c.moveToNext()) {
                map[c.getString(0)] = c.getInt(1)
            }
        }
        map
    }

    override suspend fun getWorkspaceCommandFrequencies(workspace: String): Map<String, Int> = withContext(Dispatchers.IO) {
        val db = dbHelper.readableDatabase
        val map = mutableMapOf<String, Int>()
        if (workspace.isBlank()) return@withContext emptyMap()

        val cursor = db.rawQuery(
            """
            SELECT ${CommandHistoryDbHelper.COL_COMMAND_TEXT}, COUNT(*) as cnt
            FROM ${CommandHistoryDbHelper.TABLE_COMMAND_HISTORY}
            WHERE ${CommandHistoryDbHelper.COL_WORKSPACE} = ?
            GROUP BY ${CommandHistoryDbHelper.COL_COMMAND_TEXT}
            ORDER BY cnt DESC
            """.trimIndent(),
            arrayOf(workspace.trim())
        )
        cursor.use { c ->
            while (c.moveToNext()) {
                map[c.getString(0)] = c.getInt(1)
            }
        }
        map
    }

    suspend fun getTotalEventCount(): Int = withContext(Dispatchers.IO) {
        val db = dbHelper.readableDatabase
        val cursor = db.rawQuery("SELECT COUNT(*) FROM ${CommandHistoryDbHelper.TABLE_COMMAND_HISTORY}", null)
        cursor.use { c ->
            if (c.moveToNext()) c.getInt(0) else 0
        }
    }

    override suspend fun clearHistory() = withContext(Dispatchers.IO) {
        val db = dbHelper.writableDatabase
        db.delete(CommandHistoryDbHelper.TABLE_COMMAND_HISTORY, null, null)
        Unit
    }
}
