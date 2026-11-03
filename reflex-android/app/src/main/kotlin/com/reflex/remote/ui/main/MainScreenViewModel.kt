package com.reflex.remote.ui.main

import android.app.Application
import android.content.Context
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.reflex.remote.data.PingResult
import com.reflex.remote.data.ReflexClient
import com.reflex.remote.data.ReflexResponse
import com.reflex.remote.data.model.CommandCategory
import com.reflex.remote.data.model.DesktopContext
import com.reflex.remote.data.model.ReflexCommandItem
import com.reflex.remote.data.source.AssetCommandDataSource
import com.reflex.remote.domain.SmartCommandSuggester
import com.reflex.remote.speech.VoiceRecognitionManager
import com.reflex.remote.ui.theme.ThemeMode
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
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

enum class ScreenTab {
    REMOTE,
    DIRECTORY,
    SETTINGS
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
    val currentTab: ScreenTab = ScreenTab.REMOTE,
    val pcHost: String = "192.168.0.105",
    val pcPort: Int = 9421,
    val themeMode: ThemeMode = ThemeMode.SYSTEM,
    val connectionState: ConnectionState = ConnectionState.Idle,
    val isListening: Boolean = false,
    val audioLevel: Float = 0f,
    val currentTranscript: String = "",
    val lastResponse: ReflexResponse? = null,
    val history: List<CommandLogItem> = emptyList(),
    val showSettingsDialog: Boolean = false,
    val isDispatching: Boolean = false,
    val statusBanner: String? = null,
    val directoryCategory: CommandCategory = CommandCategory.ALL,
    val directorySearchQuery: String = "",
    // Smart Context
    val desktopContext: DesktopContext? = null,
    val contextBadge: String = "STANDBY",
    val contextDescription: String = "Connecting to desktop...",
    val suggestedCommands: List<ReflexCommandItem> = emptyList(),
    val allCommands: List<ReflexCommandItem> = emptyList(),
    val isMediaSheetExpanded: Boolean = false,
    val topSearchQuery: String = ""
)

class MainScreenViewModel(application: Application) : AndroidViewModel(application) {

    private val prefs = application.getSharedPreferences("reflex_remote_prefs", Context.MODE_PRIVATE)
    private val client = ReflexClient()
    private val commandDataSource = AssetCommandDataSource(application)

    private val savedThemeMode: ThemeMode = runCatching {
        ThemeMode.valueOf(prefs.getString("theme_mode", ThemeMode.SYSTEM.name) ?: ThemeMode.SYSTEM.name)
    }.getOrDefault(ThemeMode.SYSTEM)

    private val _uiState = MutableStateFlow(
        MainUiState(
            pcHost = prefs.getString("pc_host", "192.168.0.105") ?: "192.168.0.105",
            pcPort = prefs.getInt("pc_port", 9421),
            themeMode = savedThemeMode
        )
    )
    val uiState: StateFlow<MainUiState> = _uiState.asStateFlow()

    private var voiceManager: VoiceRecognitionManager? = null
    private var pingJob: Job? = null
    private var contextPollingJob: Job? = null

    init {
        loadCommands()
        initVoiceManager()
        pingHost()
        startContextPolling()
    }

    private fun loadCommands() {
        viewModelScope.launch {
            val items = commandDataSource.loadAllCommands()
            _uiState.update { current ->
                val suggestion = SmartCommandSuggester.suggestCommands(current.desktopContext, items)
                current.copy(
                    allCommands = items,
                    contextBadge = suggestion.contextBadge,
                    contextDescription = suggestion.contextDescription,
                    suggestedCommands = suggestion.commands
                )
            }
        }
    }

    private fun startContextPolling() {
        contextPollingJob?.cancel()
        contextPollingJob = viewModelScope.launch {
            while (true) {
                delay(2000)
                val state = _uiState.value
                val res = client.pingHost(state.pcHost, state.pcPort)
                res.fold(
                    onSuccess = { pingResult ->
                        _uiState.update { it.copy(connectionState = ConnectionState.Online(pingResult.latencyMs)) }
                        if (pingResult.context != null) {
                            updateContext(pingResult.context)
                        }
                    },
                    onFailure = { err ->
                        _uiState.update { it.copy(connectionState = ConnectionState.Offline(err.message ?: "Offline")) }
                    }
                )
            }
        }
    }

    private fun updateContext(context: DesktopContext) {
        _uiState.update { current ->
            val suggestion = SmartCommandSuggester.suggestCommands(context, current.allCommands)
            current.copy(
                desktopContext = context,
                contextBadge = suggestion.contextBadge,
                contextDescription = suggestion.contextDescription,
                suggestedCommands = suggestion.commands
            )
        }
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

    fun setTab(tab: ScreenTab) {
        _uiState.update { it.copy(currentTab = tab) }
    }

    fun setDirectoryCategory(cat: CommandCategory) {
        _uiState.update { it.copy(directoryCategory = cat) }
    }

    fun setDirectorySearchQuery(query: String) {
        _uiState.update { it.copy(directorySearchQuery = query) }
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

                    // Refresh context shortly after executing command to observe changes
                    delay(200)
                    client.fetchContext(state.pcHost, state.pcPort).onSuccess { ctx ->
                        updateContext(ctx)
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
                onSuccess = { pingResult ->
                    _uiState.update { it.copy(connectionState = ConnectionState.Online(pingResult.latencyMs)) }
                    if (pingResult.context != null) {
                        updateContext(pingResult.context)
                    }
                },
                onFailure = { err ->
                    _uiState.update { it.copy(connectionState = ConnectionState.Offline(err.message ?: "Offline")) }
                }
            )
        }
    }

    fun openSettingsDialog() {
        _uiState.update { it.copy(showSettingsDialog = true) }
    }

    fun closeSettingsDialog() {
        _uiState.update { it.copy(showSettingsDialog = false) }
    }

    fun setThemeMode(mode: ThemeMode) {
        prefs.edit().putString("theme_mode", mode.name).apply()
        _uiState.update { it.copy(themeMode = mode) }
    }

    fun saveSettings(host: String, port: Int, themeMode: ThemeMode) {
        val cleanHost = host.trim().ifEmpty { "192.168.0.105" }
        prefs.edit()
            .putString("pc_host", cleanHost)
            .putInt("pc_port", port)
            .putString("theme_mode", themeMode.name)
            .apply()

        _uiState.update {
            it.copy(
                pcHost = cleanHost,
                pcPort = port,
                themeMode = themeMode,
                showSettingsDialog = false
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

    fun togglePlayPause() {
        val isPlaying = _uiState.value.desktopContext?.media?.isPlaying == true
        val cmd = if (isPlaying) "pause" else "play"
        dispatchCommand(cmd)
    }

    fun skipNext() {
        dispatchCommand("next")
    }

    fun skipPrevious() {
        dispatchCommand("previous")
    }

    fun toggleMute() {
        val isMuted = _uiState.value.desktopContext?.volume?.isMuted == true
        val cmd = if (isMuted) "unmute" else "mute"
        dispatchCommand(cmd)
    }

    fun setVolume(percent: Int) {
        dispatchCommand("volume $percent")
    }

    fun setMediaSheetExpanded(expanded: Boolean) {
        _uiState.update { it.copy(isMediaSheetExpanded = expanded) }
    }

    fun setTopSearchQuery(query: String) {
        _uiState.update { it.copy(topSearchQuery = query) }
    }

    override fun onCleared() {
        super.onCleared()
        contextPollingJob?.cancel()
        pingJob?.cancel()
        voiceManager?.destroy()
    }
}
