package com.reflex.remote.ui.main

import android.app.Application
import android.content.Context
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.reflex.remote.data.ReflexClient
import com.reflex.remote.data.ReflexResponse
import com.reflex.remote.speech.VoiceRecognitionManager
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

sealed interface ConnectionState {
    object Idle : ConnectionState
    object Checking : ConnectionState
    data class Online(val latencyMs: Long) : ConnectionState
    data class Offline(val reason: String) : ConnectionState
}

data class CommandLogItem(
    val id: String,
    val command: String,
    val timestamp: String,
    val response: ReflexResponse?,
    val isError: Boolean = false,
    val errorMessage: String? = null
)

data class MainUiState(
    val pcHost: String = "192.168.0.105",
    val pcPort: Int = 9421,
    val connectionState: ConnectionState = ConnectionState.Idle,
    val isListening: Boolean = false,
    val audioLevel: Float = 0f,
    val currentTranscript: String = "",
    val lastResponse: ReflexResponse? = null,
    val history: List<CommandLogItem> = emptyList(),
    val showSettings: Boolean = false,
    val isDispatching: Boolean = false,
    val statusBanner: String? = null
)

class MainScreenViewModel(application: Application) : AndroidViewModel(application) {

    private val prefs = application.getSharedPreferences("reflex_remote_prefs", Context.MODE_PRIVATE)
    private val client = ReflexClient()

    private val _uiState = MutableStateFlow(
        MainUiState(
            pcHost = prefs.getString("pc_host", "192.168.0.105") ?: "192.168.0.105",
            pcPort = prefs.getInt("pc_port", 9421)
        )
    )
    val uiState: StateFlow<MainUiState> = _uiState.asStateFlow()

    private var voiceManager: VoiceRecognitionManager? = null
    private var pingJob: Job? = null

    init {
        initVoiceManager()
        pingHost()
    }

    private fun initVoiceManager() {
        voiceManager = VoiceRecognitionManager(
            context = getApplication(),
            onResultRecognized = { text ->
                _uiState.update { it.copy(currentTranscript = text, isListening = false) }
                dispatchCommand(text)
            },
            onRecognitionError = { err ->
                _uiState.update { it.copy(statusBanner = err, isListening = false) }
            }
        )

        viewModelScope.launch {
            voiceManager?.isListening?.collect { listening ->
                _uiState.update { it.copy(isListening = listening) }
            }
        }
        viewModelScope.launch {
            voiceManager?.rmsDb?.collect { level ->
                _uiState.update { it.copy(audioLevel = level) }
            }
        }
        viewModelScope.launch {
            voiceManager?.liveTranscript?.collect { text ->
                if (text.isNotBlank()) {
                    _uiState.update { it.copy(currentTranscript = text) }
                }
            }
        }
    }

    fun startListening() {
        _uiState.update { it.copy(currentTranscript = "", statusBanner = null) }
        voiceManager?.startListening()
    }

    fun stopListening() {
        voiceManager?.stopListening()
    }

    fun toggleListening() {
        if (_uiState.value.isListening) {
            stopListening()
        } else {
            startListening()
        }
    }

    fun dispatchCommand(rawText: String) {
        val trimmed = rawText.trim()
        if (trimmed.isEmpty()) return

        val state = _uiState.value
        _uiState.update { it.copy(isDispatching = true, currentTranscript = trimmed) }

        viewModelScope.launch {
            val result = client.sendCommand(state.pcHost, state.pcPort, trimmed)
            val timeStr = SimpleDateFormat("HH:mm:ss", Locale.getDefault()).format(Date())

            result.fold(
                onSuccess = { resp ->
                    val item = CommandLogItem(
                        id = System.currentTimeMillis().toString(),
                        command = trimmed,
                        timestamp = timeStr,
                        response = resp,
                        isError = !resp.isSuccess
                    )
                    _uiState.update { current ->
                        current.copy(
                            isDispatching = false,
                            lastResponse = resp,
                            history = listOf(item) + current.history.take(49),
                            connectionState = ConnectionState.Online(resp.roundTripMs),
                            statusBanner = null
                        )
                    }
                },
                onFailure = { err ->
                    val item = CommandLogItem(
                        id = System.currentTimeMillis().toString(),
                        command = trimmed,
                        timestamp = timeStr,
                        response = null,
                        isError = true,
                        errorMessage = err.message ?: "Failed to reach host"
                    )
                    _uiState.update { current ->
                        current.copy(
                            isDispatching = false,
                            history = listOf(item) + current.history.take(49),
                            connectionState = ConnectionState.Offline(err.message ?: "Connection failed"),
                            statusBanner = "Host unreachable: ${err.message}"
                        )
                    }
                }
            )
        }
    }

    fun pingHost() {
        pingJob?.cancel()
        _uiState.update { it.copy(connectionState = ConnectionState.Checking) }

        pingJob = viewModelScope.launch {
            val state = _uiState.value
            val res = client.pingHost(state.pcHost, state.pcPort)
            res.fold(
                onSuccess = { latency ->
                    _uiState.update { it.copy(connectionState = ConnectionState.Online(latency)) }
                },
                onFailure = { err ->
                    _uiState.update { it.copy(connectionState = ConnectionState.Offline(err.message ?: "Offline")) }
                }
            )
        }
    }

    fun openSettings() {
        _uiState.update { it.copy(showSettings = true) }
    }

    fun closeSettings() {
        _uiState.update { it.copy(showSettings = false) }
    }

    fun saveSettings(host: String, port: Int) {
        val cleanHost = host.trim().ifEmpty { "192.168.0.105" }
        prefs.edit()
            .putString("pc_host", cleanHost)
            .putInt("pc_port", port)
            .apply()

        _uiState.update {
            it.copy(
                pcHost = cleanHost,
                pcPort = port,
                showSettings = false
            )
        }
        pingHost()
    }

    fun clearHistory() {
        _uiState.update { it.copy(history = emptyList(), lastResponse = null, currentTranscript = "") }
    }

    fun dismissBanner() {
        _uiState.update { it.copy(statusBanner = null) }
    }

    override fun onCleared() {
        super.onCleared()
        voiceManager?.destroy()
    }
}
