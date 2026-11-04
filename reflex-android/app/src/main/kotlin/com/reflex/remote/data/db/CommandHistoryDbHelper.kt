package com.reflex.remote.data.db

import android.content.Context
import android.database.sqlite.SQLiteDatabase
import android.database.sqlite.SQLiteOpenHelper

class CommandHistoryDbHelper(context: Context) :
    SQLiteOpenHelper(context, DATABASE_NAME, null, DATABASE_VERSION) {

    override fun onCreate(db: SQLiteDatabase) {
        db.execSQL(
            """
            CREATE TABLE $TABLE_COMMAND_HISTORY (
                $COL_ID INTEGER PRIMARY KEY AUTOINCREMENT,
                $COL_COMMAND_TEXT TEXT NOT NULL,
                $COL_COMMAND_ID TEXT,
                $COL_APP_NAME TEXT,
                $COL_WINDOW_TITLE TEXT,
                $COL_WINDOW_CLASS TEXT,
                $COL_WORKSPACE TEXT,
                $COL_IS_MEDIA_PLAYING INTEGER NOT NULL DEFAULT 0,
                $COL_MEDIA_PLAYER TEXT,
                $COL_MEDIA_TITLE TEXT,
                $COL_PREVIOUS_COMMAND TEXT,
                $COL_TIMESTAMP INTEGER NOT NULL,
                $COL_SOURCE TEXT NOT NULL DEFAULT 'android'
            )
            """.trimIndent()
        )

        db.execSQL("CREATE INDEX idx_ch_timestamp ON $TABLE_COMMAND_HISTORY($COL_TIMESTAMP DESC)")
        db.execSQL("CREATE INDEX idx_ch_prev ON $TABLE_COMMAND_HISTORY($COL_PREVIOUS_COMMAND)")
        db.execSQL("CREATE INDEX idx_ch_app ON $TABLE_COMMAND_HISTORY($COL_APP_NAME)")
        db.execSQL("CREATE INDEX idx_ch_cmd ON $TABLE_COMMAND_HISTORY($COL_COMMAND_TEXT)")
    }

    override fun onUpgrade(db: SQLiteDatabase, oldVersion: Int, newVersion: Int) {
        db.execSQL("DROP TABLE IF EXISTS $TABLE_COMMAND_HISTORY")
        onCreate(db)
    }

    companion object {
        const val DATABASE_NAME = "reflex_command_history.db"
        const val DATABASE_VERSION = 1

        const val TABLE_COMMAND_HISTORY = "command_history"
        const val COL_ID = "id"
        const val COL_COMMAND_TEXT = "command_text"
        const val COL_COMMAND_ID = "command_id"
        const val COL_APP_NAME = "app_name"
        const val COL_WINDOW_TITLE = "window_title"
        const val COL_WINDOW_CLASS = "window_class"
        const val COL_WORKSPACE = "workspace"
        const val COL_IS_MEDIA_PLAYING = "is_media_playing"
        const val COL_MEDIA_PLAYER = "media_player"
        const val COL_MEDIA_TITLE = "media_title"
        const val COL_PREVIOUS_COMMAND = "previous_command"
        const val COL_TIMESTAMP = "timestamp"
        const val COL_SOURCE = "source"
    }
}
