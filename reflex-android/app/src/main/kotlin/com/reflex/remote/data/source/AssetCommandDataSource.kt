package com.reflex.remote.data.source

import android.content.Context
import android.util.Log
import com.reflex.remote.data.model.CommandCategory
import com.reflex.remote.data.model.ReflexCommandItem
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.json.JSONArray
import java.io.BufferedReader
import java.io.InputStreamReader

interface CommandDataSource {
    suspend fun loadAllCommands(): List<ReflexCommandItem>
    fun getCachedCommands(): List<ReflexCommandItem>
    fun getCommandsByCategory(category: CommandCategory): List<ReflexCommandItem>
}

class AssetCommandDataSource(
    private val context: Context
) : CommandDataSource {

    private val cachedCommands = mutableListOf<ReflexCommandItem>()
    private var isLoaded = false

    override fun getCachedCommands(): List<ReflexCommandItem> {
        return synchronized(cachedCommands) {
            cachedCommands.toList()
        }
    }

    override fun getCommandsByCategory(category: CommandCategory): List<ReflexCommandItem> {
        val all = getCachedCommands()
        if (category == CommandCategory.ALL) return all
        return all.filter { it.category == category }
    }

    override suspend fun loadAllCommands(): List<ReflexCommandItem> = withContext(Dispatchers.IO) {
        if (isLoaded && cachedCommands.isNotEmpty()) {
            return@withContext getCachedCommands()
        }

        val allItems = mutableListOf<ReflexCommandItem>()
        val categories = CommandCategory.values().filter { it != CommandCategory.ALL }

        for (cat in categories) {
            if (cat.jsonFileName.isBlank()) continue
            try {
                val assetPath = "commands/${cat.jsonFileName}"
                val jsonString = context.assets.open(assetPath).use { input ->
                    BufferedReader(InputStreamReader(input, Charsets.UTF_8)).readText()
                }

                val array = JSONArray(jsonString)
                for (i in 0 until array.length()) {
                    val obj = array.getJSONObject(i)
                    val id = obj.optString("id", "")
                    val title = obj.optString("title", "")
                    val command = obj.optString("command", "")
                    val description = obj.optString("description", "")
                    val aliasesJson = obj.optJSONArray("aliases")
                    val aliases = mutableListOf<String>()
                    if (aliasesJson != null) {
                        for (j in 0 until aliasesJson.length()) {
                            aliases.add(aliasesJson.optString(j))
                        }
                    }

                    if (id.isNotBlank() && command.isNotBlank()) {
                        allItems.add(
                            ReflexCommandItem(
                                id = id,
                                title = title,
                                command = command,
                                category = cat,
                                description = description,
                                aliases = aliases
                            )
                        )
                    }
                }
            } catch (e: Exception) {
                Log.e("AssetCommandDataSource", "Failed to load commands for category ${cat.name}", e)
            }
        }

        synchronized(cachedCommands) {
            cachedCommands.clear()
            cachedCommands.addAll(allItems)
            isLoaded = true
        }

        return@withContext allItems
    }
}
