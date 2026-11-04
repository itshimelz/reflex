package com.reflex.remote.data

import com.reflex.remote.data.model.ActiveWindowContext
import com.reflex.remote.data.model.DesktopContext
import com.reflex.remote.data.model.MediaContext
import com.reflex.remote.data.model.VolumeContext
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import org.json.JSONObject
import java.io.BufferedReader
import java.io.InputStreamReader
import java.io.OutputStreamWriter
import java.net.HttpURLConnection
import java.net.InetSocketAddress
import java.net.Socket
import java.net.URL

data class ReflexResponse(
    val status: String,
    val reason: String? = null,
    val action: String? = null,
    val clampedTo: Int? = null,
    val durationUs: Long = 0L,
    val roundTripMs: Long = 0L,
    val isSuccess: Boolean = status.equals("Success", ignoreCase = true)
)

data class PingResult(
    val latencyMs: Long,
    val context: DesktopContext? = null
)

class ReflexClient {

    private val tcpMutex = Mutex()
    private var persistentSocket: Socket? = null
    private var socketReader: BufferedReader? = null
    private var socketWriter: OutputStreamWriter? = null
    private var connectedHost: String? = null
    private var connectedPort: Int? = null

    private fun closeTcpSession() {
        try {
            socketWriter?.close()
        } catch (_: Exception) {}
        try {
            socketReader?.close()
        } catch (_: Exception) {}
        try {
            persistentSocket?.close()
        } catch (_: Exception) {}
        persistentSocket = null
        socketReader = null
        socketWriter = null
        connectedHost = null
        connectedPort = null
    }

    private fun ensureTcpConnected(host: String, port: Int): Boolean {
        if (persistentSocket?.isConnected == true &&
            persistentSocket?.isClosed == false &&
            connectedHost == host &&
            connectedPort == port
        ) {
            return true
        }

        closeTcpSession()

        return try {
            val socket = Socket().apply {
                tcpNoDelay = true
                keepAlive = true
                soTimeout = 2500
                connect(InetSocketAddress(host, port), 2000)
            }
            persistentSocket = socket
            socketReader = BufferedReader(InputStreamReader(socket.getInputStream(), Charsets.UTF_8))
            socketWriter = OutputStreamWriter(socket.getOutputStream(), Charsets.UTF_8)
            connectedHost = host
            connectedPort = port
            true
        } catch (_: Exception) {
            closeTcpSession()
            false
        }
    }

    suspend fun pingHost(host: String, port: Int): Result<PingResult> = withContext(Dispatchers.IO) {
        // 1. Try persistent low-latency raw TCP stream first (sub-15ms on local Wi-Fi)
        tcpMutex.withLock {
            if (ensureTcpConnected(host, port)) {
                try {
                    val writer = socketWriter ?: throw IllegalStateException("Socket writer null")
                    val reader = socketReader ?: throw IllegalStateException("Socket reader null")

                    val pingStart = System.currentTimeMillis()
                    writer.write("PING\n")
                    writer.flush()
                    val pongLine = reader.readLine()
                    if (pongLine != null && pongLine.trim().equals("PONG", ignoreCase = true)) {
                        val latency = System.currentTimeMillis() - pingStart

                        // Retrieve context on the same open socket
                        var desktopContext: DesktopContext? = null
                        try {
                            writer.write("CONTEXT\n")
                            writer.flush()
                            val contextLine = reader.readLine()
                            if (contextLine != null && contextLine.startsWith("{")) {
                                desktopContext = parseDesktopContext(JSONObject(contextLine))
                            }
                        } catch (_: Exception) {}

                        return@withContext Result.success(PingResult(latencyMs = latency, context = desktopContext))
                    }
                } catch (_: Exception) {
                    closeTcpSession()
                }
            }
        }

        // 2. HTTP Fallback with Keep-Alive
        val httpStart = System.currentTimeMillis()
        try {
            val url = URL("http://$host:$port/ping")
            val conn = (url.openConnection() as HttpURLConnection).apply {
                connectTimeout = 1500
                readTimeout = 1500
                requestMethod = "GET"
                setRequestProperty("Connection", "Keep-Alive")
            }
            val responseCode = conn.responseCode
            val body = if (responseCode in 200..299) {
                BufferedReader(InputStreamReader(conn.inputStream, Charsets.UTF_8)).use { it.readText() }
            } else {
                conn.disconnect()
                return@withContext Result.failure(Exception("HTTP error $responseCode"))
            }
            conn.disconnect()

            val latency = System.currentTimeMillis() - httpStart
            var desktopContext: DesktopContext? = null
            try {
                val json = JSONObject(body)
                val ctxObj = json.optJSONObject("context")
                if (ctxObj != null) {
                    desktopContext = parseDesktopContext(ctxObj)
                }
            } catch (_: Exception) {}

            Result.success(PingResult(latencyMs = latency, context = desktopContext))
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    suspend fun fetchContext(host: String, port: Int): Result<DesktopContext> = withContext(Dispatchers.IO) {
        // Try TCP stream
        tcpMutex.withLock {
            if (ensureTcpConnected(host, port)) {
                try {
                    val writer = socketWriter ?: throw IllegalStateException("Socket writer null")
                    val reader = socketReader ?: throw IllegalStateException("Socket reader null")

                    writer.write("CONTEXT\n")
                    writer.flush()
                    val line = reader.readLine()
                    if (line != null && line.startsWith("{")) {
                        val ctx = parseDesktopContext(JSONObject(line))
                        return@withContext Result.success(ctx)
                    }
                } catch (_: Exception) {
                    closeTcpSession()
                }
            }
        }

        // Fallback HTTP
        try {
            val url = URL("http://$host:$port/api/context")
            val conn = (url.openConnection() as HttpURLConnection).apply {
                connectTimeout = 1500
                readTimeout = 1500
                requestMethod = "GET"
            }
            if (conn.responseCode in 200..299) {
                val body = BufferedReader(InputStreamReader(conn.inputStream, Charsets.UTF_8)).use { it.readText() }
                conn.disconnect()
                Result.success(parseDesktopContext(JSONObject(body)))
            } else {
                conn.disconnect()
                Result.failure(Exception("HTTP ${conn.responseCode}"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    suspend fun sendCommand(
        host: String,
        port: Int,
        commandText: String
    ): Result<ReflexResponse> = withContext(Dispatchers.IO) {
        val startTime = System.currentTimeMillis()

        // 1. Try persistent low-latency raw TCP socket (<2ms)
        tcpMutex.withLock {
            if (ensureTcpConnected(host, port)) {
                try {
                    val writer = socketWriter ?: throw IllegalStateException("Socket writer null")
                    val reader = socketReader ?: throw IllegalStateException("Socket reader null")

                    val cmdStart = System.currentTimeMillis()
                    writer.write("COMMAND $commandText\n")
                    writer.flush()
                    val line = reader.readLine()
                    if (line != null && line.isNotBlank()) {
                        val roundTrip = System.currentTimeMillis() - cmdStart
                        val json = JSONObject(line)
                        val status = json.optString("status", "Unknown")
                        val reason = if (json.has("reason")) json.optString("reason") else null
                        val action = if (json.has("action")) json.optString("action") else null
                        val clampedTo = if (json.has("clamped_to") && !json.isNull("clamped_to")) json.optInt("clamped_to") else null
                        val durationUs = json.optLong("duration_us", 0L)

                        return@withContext Result.success(
                            ReflexResponse(
                                status = status,
                                reason = reason,
                                action = action,
                                clampedTo = clampedTo,
                                durationUs = durationUs,
                                roundTripMs = roundTrip
                            )
                        )
                    }
                } catch (_: Exception) {
                    closeTcpSession()
                }
            }
        }

        // 2. HTTP Fallback POST /api/command
        try {
            val url = URL("http://$host:$port/api/command")
            val conn = (url.openConnection() as HttpURLConnection).apply {
                connectTimeout = 2500
                readTimeout = 4000
                requestMethod = "POST"
                doOutput = true
                setRequestProperty("Content-Type", "application/json; charset=UTF-8")
                setRequestProperty("Accept", "application/json")
            }

            val jsonBody = JSONObject().apply {
                put("command", commandText)
            }.toString()

            OutputStreamWriter(conn.outputStream, Charsets.UTF_8).use { writer ->
                writer.write(jsonBody)
                writer.flush()
            }

            val statusCode = conn.responseCode
            val inputStream = if (statusCode in 200..299) {
                conn.inputStream
            } else {
                conn.errorStream ?: conn.inputStream
            }

            val responseBody = BufferedReader(InputStreamReader(inputStream, Charsets.UTF_8)).use { reader ->
                reader.readText()
            }
            conn.disconnect()

            val roundTrip = System.currentTimeMillis() - startTime
            val json = JSONObject(responseBody)

            val status = json.optString("status", "Unknown")
            val reason = if (json.has("reason")) json.optString("reason") else null
            val action = if (json.has("action")) json.optString("action") else null
            val clampedTo = if (json.has("clamped_to") && !json.isNull("clamped_to")) json.optInt("clamped_to") else null
            val durationUs = json.optLong("duration_us", 0L)

            Result.success(
                ReflexResponse(
                    status = status,
                    reason = reason,
                    action = action,
                    clampedTo = clampedTo,
                    durationUs = durationUs,
                    roundTripMs = roundTrip
                )
            )
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    suspend fun fetchRemoteHistory(
        host: String,
        port: Int
    ): Result<List<com.reflex.remote.data.db.CommandEvent>> = withContext(Dispatchers.IO) {
        try {
            val url = URL("http://$host:$port/api/history")
            val conn = (url.openConnection() as HttpURLConnection).apply {
                connectTimeout = 2500
                readTimeout = 3000
                requestMethod = "GET"
                setRequestProperty("Accept", "application/json")
            }
            if (conn.responseCode in 200..299) {
                val body = BufferedReader(InputStreamReader(conn.inputStream, Charsets.UTF_8)).use { it.readText() }
                conn.disconnect()
                val array = org.json.JSONArray(body)
                val list = mutableListOf<com.reflex.remote.data.db.CommandEvent>()
                for (i in 0 until array.length()) {
                    val obj = array.getJSONObject(i)
                    list.add(
                        com.reflex.remote.data.db.CommandEvent(
                            commandText = obj.optString("command", ""),
                            appName = obj.optString("app_name", ""),
                            windowTitle = obj.optString("window_title", ""),
                            workspace = obj.optString("workspace", ""),
                            isMediaPlaying = obj.optBoolean("media_playing", false),
                            mediaPlayer = obj.optString("media_player", ""),
                            timestamp = obj.optLong("timestamp_ms", System.currentTimeMillis()),
                            source = "desktop_sync"
                        )
                    )
                }
                Result.success(list)
            } else {
                conn.disconnect()
                Result.failure(Exception("HTTP ${conn.responseCode}"))
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    private fun parseDesktopContext(json: JSONObject): DesktopContext {
        val winObj = json.optJSONObject("active_window")
        val activeWindow = if (winObj != null) {
            ActiveWindowContext(
                className = winObj.optString("class", ""),
                title = winObj.optString("title", ""),
                appName = winObj.optString("app_name", ""),
                workspace = if (winObj.has("workspace") && !winObj.isNull("workspace")) winObj.optInt("workspace") else null
            )
        } else {
            ActiveWindowContext()
        }

        val mediaObj = json.optJSONObject("media")
        val media = if (mediaObj != null) {
            val pos = if (mediaObj.has("position_seconds") && !mediaObj.isNull("position_seconds")) {
                mediaObj.optDouble("position_seconds").toFloat()
            } else null

            val dur = if (mediaObj.has("duration_seconds") && !mediaObj.isNull("duration_seconds")) {
                mediaObj.optDouble("duration_seconds").toFloat()
            } else null

            val art = if (mediaObj.has("art_url") && !mediaObj.isNull("art_url")) {
                mediaObj.optString("art_url").ifBlank { null }
            } else null

            MediaContext(
                player = mediaObj.optString("player", ""),
                title = mediaObj.optString("title", ""),
                artist = mediaObj.optString("artist", ""),
                status = mediaObj.optString("status", "Stopped"),
                positionSeconds = pos,
                durationSeconds = dur,
                artUrl = art
            )
        } else {
            MediaContext()
        }

        val volObj = json.optJSONObject("volume")
        val volume = if (volObj != null) {
            VolumeContext(
                percent = volObj.optInt("percent", 50),
                isMuted = volObj.optBoolean("muted", false)
            )
        } else {
            VolumeContext()
        }

        return DesktopContext(
            activeWindow = activeWindow,
            media = media,
            volume = volume
        )
    }
}
