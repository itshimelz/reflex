package com.reflex.remote.data

import kotlinx.coroutines.Dispatchers
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

class ReflexClient {

    suspend fun pingHost(host: String, port: Int): Result<Long> = withContext(Dispatchers.IO) {
        val start = System.currentTimeMillis()
        try {
            val url = URL("http://$host:$port/ping")
            val conn = (url.openConnection() as HttpURLConnection).apply {
                connectTimeout = 2000
                readTimeout = 2000
                requestMethod = "GET"
            }
            val responseCode = conn.responseCode
            conn.disconnect()
            if (responseCode in 200..299) {
                val elapsed = System.currentTimeMillis() - start
                Result.success(elapsed)
            } else {
                Result.failure(Exception("HTTP error $responseCode"))
            }
        } catch (e: Exception) {
            // Fallback: direct TCP connect ping
            try {
                val socket = Socket()
                socket.connect(InetSocketAddress(host, port), 2000)
                socket.close()
                val elapsed = System.currentTimeMillis() - start
                Result.success(elapsed)
            } catch (sockEx: Exception) {
                Result.failure(e)
            }
        }
    }

    suspend fun sendCommand(
        host: String,
        port: Int,
        commandText: String
    ): Result<ReflexResponse> = withContext(Dispatchers.IO) {
        val startTime = System.currentTimeMillis()
        try {
            val url = URL("http://$host:$port/api/command")
            val conn = (url.openConnection() as HttpURLConnection).apply {
                connectTimeout = 3000
                readTimeout = 5000
                requestMethod = "POST"
                doOutput = true
                setRequestProperty("Content-Type", "application/json; charset=UTF-8")
                setRequestProperty("Accept", "application/json")
            }

            val jsonBody = JSONObject().apply {
                put("command", commandText)
            }.toString()

            OutputStreamWriter(conn.outputStream, "UTF-8").use { writer ->
                writer.write(jsonBody)
                writer.flush()
            }

            val statusCode = conn.responseCode
            val inputStream = if (statusCode in 200..299) {
                conn.inputStream
            } else {
                conn.errorStream ?: conn.inputStream
            }

            val responseBody = BufferedReader(InputStreamReader(inputStream, "UTF-8")).use { reader ->
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

            val response = ReflexResponse(
                status = status,
                reason = reason,
                action = action,
                clampedTo = clampedTo,
                durationUs = durationUs,
                roundTripMs = roundTrip
            )
            Result.success(response)
        } catch (e: Exception) {
            // Fallback: try raw TCP socket dispatch
            try {
                val socket = Socket()
                socket.connect(InetSocketAddress(host, port), 3000)
                socket.soTimeout = 4000

                val writer = OutputStreamWriter(socket.getOutputStream(), "UTF-8")
                writer.write("$commandText\n")
                writer.flush()

                val reader = BufferedReader(InputStreamReader(socket.getInputStream(), "UTF-8"))
                val line = reader.readLine()
                socket.close()

                val roundTrip = System.currentTimeMillis() - startTime
                if (line != null && line.isNotBlank()) {
                    val json = JSONObject(line)
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
                } else {
                    Result.failure(Exception("Empty response from host"))
                }
            } catch (sockEx: Exception) {
                Result.failure(e)
            }
        }
    }
}
