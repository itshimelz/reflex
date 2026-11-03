package com.reflex.remote.ui.main

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.reflex.remote.R

// Modern Technical Theme Palette
private val ColorBg = Color(0xFF090A0E)
private val ColorCard = Color(0xFF12151D)
private val ColorCardBorder = Color(0xFF1E2433)
private val ColorCyan = Color(0xFF00E5FF)
private val ColorBlue = Color(0xFF2563EB)
private val ColorGreen = Color(0xFF10B981)
private val ColorAmber = Color(0xFFF59E0B)
private val ColorRed = Color(0xFFEF4444)
private val ColorTextMuted = Color(0xFF8E9BB0)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MainScreen(
    viewModel: MainScreenViewModel,
    modifier: Modifier = Modifier
) {
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()

    Scaffold(
        modifier = modifier.fillMaxSize(),
        containerColor = ColorBg,
        topBar = {
            ReflexTopBar(
                connectionState = uiState.connectionState,
                onPingClick = { viewModel.pingHost() },
                onSettingsClick = { viewModel.openSettings() }
            )
        }
    ) { innerPadding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
                .padding(horizontal = 20.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Spacer(modifier = Modifier.height(12.dp))

            // Status Banner if any
            AnimatedVisibility(visible = uiState.statusBanner != null) {
                uiState.statusBanner?.let { banner ->
                    StatusBanner(
                        text = banner,
                        onDismiss = { viewModel.dismissBanner() }
                    )
                }
            }

            // Central Voice Hero (Push-to-Talk)
            VoiceHeroSection(
                isListening = uiState.isListening,
                isDispatching = uiState.isDispatching,
                audioLevel = uiState.audioLevel,
                onMicClick = { viewModel.toggleListening() }
            )

            Spacer(modifier = Modifier.height(16.dp))

            // Current Transcript / Response Card
            LiveTranscriptCard(
                transcript = uiState.currentTranscript,
                isListening = uiState.isListening,
                isDispatching = uiState.isDispatching,
                lastResponse = uiState.lastResponse
            )

            Spacer(modifier = Modifier.height(16.dp))

            // Quick Shortcut Chips
            QuickShortcutsRow(
                onShortcutClick = { command -> viewModel.dispatchCommand(command) }
            )

            Spacer(modifier = Modifier.height(16.dp))

            // Activity Log
            ActivityLogSection(
                history = uiState.history,
                onClearClick = { viewModel.clearHistory() },
                modifier = Modifier.weight(1f)
            )
        }
    }

    if (uiState.showSettings) {
        SettingsDialog(
            currentHost = uiState.pcHost,
            currentPort = uiState.pcPort,
            connectionState = uiState.connectionState,
            onPing = { viewModel.pingHost() },
            onSave = { host, port -> viewModel.saveSettings(host, port) },
            onDismiss = { viewModel.closeSettings() }
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun ReflexTopBar(
    connectionState: ConnectionState,
    onPingClick: () -> Unit,
    onSettingsClick: () -> Unit
) {
    TopAppBar(
        colors = TopAppBarDefaults.topAppBarColors(
            containerColor = ColorBg,
            titleContentColor = Color.White
        ),
        title = {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(10.dp)
            ) {
                Icon(
                    painter = painterResource(id = R.drawable.ic_reflex_logo),
                    contentDescription = "Reflex Logo",
                    tint = Color.Unspecified,
                    modifier = Modifier.size(32.dp)
                )
                Text(
                    text = "REFLEX",
                    fontFamily = FontFamily.Monospace,
                    fontWeight = FontWeight.Bold,
                    fontSize = 18.sp,
                    letterSpacing = 2.sp,
                    color = Color.White
                )
            }
        },
        actions = {
            // Connection Pill Badge
            ConnectionPill(
                connectionState = connectionState,
                onClick = onPingClick
            )

            IconButton(onClick = onSettingsClick) {
                Icon(
                    painter = painterResource(id = R.drawable.ic_settings),
                    contentDescription = "Settings",
                    tint = ColorTextMuted,
                    modifier = Modifier.size(22.dp)
                )
            }
        }
    )
}

@Composable
private fun ConnectionPill(
    connectionState: ConnectionState,
    onClick: () -> Unit
) {
    val (dotColor, label) = when (connectionState) {
        is ConnectionState.Online -> ColorGreen to "ONLINE ${connectionState.latencyMs}ms"
        is ConnectionState.Offline -> ColorRed to "OFFLINE"
        is ConnectionState.Checking -> ColorCyan to "PINGING..."
        is ConnectionState.Idle -> ColorTextMuted to "HOST"
    }

    Row(
        modifier = Modifier
            .clip(RoundedCornerShape(20.dp))
            .background(ColorCard)
            .border(1.dp, ColorCardBorder, RoundedCornerShape(20.dp))
            .clickable(onClick = onClick)
            .padding(horizontal = 10.dp, vertical = 6.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(6.dp)
    ) {
        Box(
            modifier = Modifier
                .size(8.dp)
                .clip(CircleShape)
                .background(dotColor)
        )
        Text(
            text = label,
            fontSize = 11.sp,
            fontFamily = FontFamily.Monospace,
            fontWeight = FontWeight.SemiBold,
            color = Color.White
        )
    }
}

@Composable
private fun VoiceHeroSection(
    isListening: Boolean,
    isDispatching: Boolean,
    audioLevel: Float,
    onMicClick: () -> Unit
) {
    val infiniteTransition = rememberInfiniteTransition(label = "pulse")
    val pulseScale by infiniteTransition.animateFloat(
        initialValue = 1.0f,
        targetValue = 1.25f,
        animationSpec = infiniteRepeatable(
            animation = tween(900, easing = FastOutSlowInEasing),
            repeatMode = RepeatMode.Reverse
        ),
        label = "pulseScale"
    )

    val dynamicScale by animateFloatAsState(
        targetValue = if (isListening) (1.0f + (audioLevel / 20f)).coerceIn(1.0f, 1.4f) else 1.0f,
        label = "dynamicScale"
    )

    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        modifier = Modifier.padding(vertical = 12.dp)
    ) {
        Box(
            contentAlignment = Alignment.Center,
            modifier = Modifier.size(170.dp)
        ) {
            // Animated breathing ripple rings
            if (isListening) {
                Box(
                    modifier = Modifier
                        .size(160.dp)
                        .scale(pulseScale)
                        .clip(CircleShape)
                        .background(ColorCyan.copy(alpha = 0.15f))
                )
                Box(
                    modifier = Modifier
                        .size(136.dp)
                        .scale(dynamicScale)
                        .clip(CircleShape)
                        .background(ColorCyan.copy(alpha = 0.25f))
                )
            }

            // Main Tactile Mic Button
            val buttonColor = if (isListening) ColorRed else ColorCyan
            val buttonGrad = if (isListening) {
                Brush.linearGradient(listOf(Color(0xFFEF4444), Color(0xFFB91C1C)))
            } else {
                Brush.linearGradient(listOf(Color(0xFF00E5FF), Color(0xFF2563EB)))
            }

            Box(
                modifier = Modifier
                    .size(94.dp)
                    .clip(CircleShape)
                    .background(buttonGrad)
                    .clickable(onClick = onMicClick),
                contentAlignment = Alignment.Center
            ) {
                if (isDispatching) {
                    CircularProgressIndicator(
                        modifier = Modifier.size(42.dp),
                        color = Color.White,
                        strokeWidth = 3.dp
                    )
                } else {
                    Icon(
                        painter = painterResource(id = R.drawable.ic_mic),
                        contentDescription = "Voice Input",
                        tint = Color.White,
                        modifier = Modifier.size(44.dp)
                    )
                }
            }
        }

        Spacer(modifier = Modifier.height(4.dp))

        Text(
            text = when {
                isDispatching -> "DISPATCHING TO PC..."
                isListening -> "LISTENING TO VOICE..."
                else -> "HOLD OR TAP TO SPEAK"
            },
            fontFamily = FontFamily.Monospace,
            fontSize = 12.sp,
            fontWeight = FontWeight.Bold,
            letterSpacing = 1.5.sp,
            color = if (isListening) ColorCyan else ColorTextMuted
        )
    }
}

@Composable
private fun LiveTranscriptCard(
    transcript: String,
    isListening: Boolean,
    isDispatching: Boolean,
    lastResponse: com.reflex.remote.data.ReflexResponse?
) {
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(16.dp))
            .border(1.dp, ColorCardBorder, RoundedCornerShape(16.dp)),
        colors = CardDefaults.cardColors(containerColor = ColorCard)
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp)
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    text = "TRANSCRIPT & EXECUTION",
                    fontSize = 10.sp,
                    fontFamily = FontFamily.Monospace,
                    fontWeight = FontWeight.Bold,
                    color = ColorTextMuted,
                    letterSpacing = 1.2.sp
                )

                if (lastResponse != null) {
                    val badgeColor = if (lastResponse.isSuccess) ColorGreen else ColorAmber
                    Text(
                        text = lastResponse.status.uppercase(),
                        fontFamily = FontFamily.Monospace,
                        fontSize = 10.sp,
                        fontWeight = FontWeight.Bold,
                        color = badgeColor
                    )
                }
            }

            Spacer(modifier = Modifier.height(8.dp))

            val displayTranscript = if (transcript.isNotBlank()) {
                transcript
            } else if (isListening) {
                "..."
            } else {
                "Speak a command, e.g. \"sound 50\", \"weppy\", \"zip all\""
            }

            Text(
                text = displayTranscript,
                fontSize = 15.sp,
                fontWeight = FontWeight.Medium,
                color = if (transcript.isNotBlank()) Color.White else ColorTextMuted,
                lineHeight = 20.sp
            )

            // Execution telemetry details
            if (lastResponse != null) {
                Spacer(modifier = Modifier.height(10.dp))
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        text = lastResponse.action ?: lastResponse.reason ?: "Executed",
                        fontSize = 12.sp,
                        fontFamily = FontFamily.Monospace,
                        color = ColorCyan,
                        fontWeight = FontWeight.SemiBold
                    )
                    Text(
                        text = "${lastResponse.durationUs}us PC / ${lastResponse.roundTripMs}ms Net",
                        fontSize = 11.sp,
                        fontFamily = FontFamily.Monospace,
                        color = ColorTextMuted
                    )
                }
            }
        }
    }
}

@Composable
private fun QuickShortcutsRow(
    onShortcutClick: (String) -> Unit
) {
    val shortcuts = listOf(
        "sound 50",
        "sound 75",
        "mute",
        "screenshot",
        "weppy",
        "zip all",
        "terminal",
        "next track",
        "full path",
        "clean filename"
    )

    Column(modifier = Modifier.fillMaxWidth()) {
        Text(
            text = "QUICK COMMANDS",
            fontSize = 10.sp,
            fontFamily = FontFamily.Monospace,
            fontWeight = FontWeight.Bold,
            color = ColorTextMuted,
            letterSpacing = 1.2.sp,
            modifier = Modifier.padding(bottom = 8.dp)
        )

        Row(
            modifier = Modifier
                .fillMaxWidth()
                .horizontalScroll(rememberScrollState()),
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            shortcuts.forEach { cmd ->
                Surface(
                    color = ColorCard,
                    shape = RoundedCornerShape(10.dp),
                    modifier = Modifier
                        .border(1.dp, ColorCardBorder, RoundedCornerShape(10.dp))
                        .clickable { onShortcutClick(cmd) }
                ) {
                    Text(
                        text = cmd,
                        fontSize = 12.sp,
                        fontFamily = FontFamily.Monospace,
                        color = Color.White,
                        modifier = Modifier.padding(horizontal = 12.dp, vertical = 8.dp)
                    )
                }
            }
        }
    }
}

@Composable
private fun ActivityLogSection(
    history: List<CommandLogItem>,
    onClearClick: () -> Unit,
    modifier: Modifier = Modifier
) {
    Column(modifier = modifier.fillMaxWidth()) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                text = "DISPATCH HISTORY",
                fontSize = 10.sp,
                fontFamily = FontFamily.Monospace,
                fontWeight = FontWeight.Bold,
                color = ColorTextMuted,
                letterSpacing = 1.2.sp
            )

            if (history.isNotEmpty()) {
                Text(
                    text = "CLEAR",
                    fontSize = 10.sp,
                    fontFamily = FontFamily.Monospace,
                    fontWeight = FontWeight.Bold,
                    color = ColorCyan,
                    modifier = Modifier
                        .clickable(onClick = onClearClick)
                        .padding(4.dp)
                )
            }
        }

        Spacer(modifier = Modifier.height(6.dp))

        if (history.isEmpty()) {
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(vertical = 24.dp),
                contentAlignment = Alignment.Center
            ) {
                Text(
                    text = "No commands dispatched yet",
                    fontFamily = FontFamily.Monospace,
                    fontSize = 12.sp,
                    color = ColorTextMuted
                )
            }
        } else {
            LazyColumn(
                modifier = Modifier.fillMaxSize(),
                verticalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                items(history, key = { it.id }) { item ->
                    HistoryItemCard(item = item)
                }
            }
        }
    }
}

@Composable
private fun HistoryItemCard(item: CommandLogItem) {
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(12.dp))
            .border(1.dp, ColorCardBorder, RoundedCornerShape(12.dp)),
        colors = CardDefaults.cardColors(containerColor = ColorCard)
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 14.dp, vertical = 10.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = item.command,
                    fontSize = 14.sp,
                    fontWeight = FontWeight.SemiBold,
                    color = Color.White
                )
                val detail = item.response?.action
                    ?: item.response?.reason
                    ?: item.errorMessage
                    ?: "Dispatched"
                Text(
                    text = detail,
                    fontSize = 11.sp,
                    fontFamily = FontFamily.Monospace,
                    color = if (item.isError) ColorAmber else ColorCyan,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
            }

            Column(horizontalAlignment = Alignment.End) {
                val statusText = if (item.isError) "FAIL" else "OK"
                val statusColor = if (item.isError) ColorRed else ColorGreen
                Text(
                    text = statusText,
                    fontSize = 11.sp,
                    fontFamily = FontFamily.Monospace,
                    fontWeight = FontWeight.Bold,
                    color = statusColor
                )
                Text(
                    text = item.timestamp,
                    fontSize = 10.sp,
                    fontFamily = FontFamily.Monospace,
                    color = ColorTextMuted
                )
            }
        }
    }
}

@Composable
private fun StatusBanner(
    text: String,
    onDismiss: () -> Unit
) {
    Surface(
        color = Color(0xFF2A1515),
        shape = RoundedCornerShape(10.dp),
        modifier = Modifier
            .fillMaxWidth()
            .padding(bottom = 10.dp)
            .border(1.dp, Color(0xFF7F1D1D), RoundedCornerShape(10.dp))
            .clickable(onClick = onDismiss)
    ) {
        Row(
            modifier = Modifier.padding(12.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(10.dp)
        ) {
            Icon(
                painter = painterResource(id = R.drawable.ic_alert),
                contentDescription = "Alert",
                tint = ColorAmber,
                modifier = Modifier.size(18.dp)
            )
            Text(
                text = text,
                color = Color.White,
                fontSize = 12.sp,
                fontFamily = FontFamily.Monospace,
                modifier = Modifier.weight(1f)
            )
        }
    }
}

@Composable
private fun SettingsDialog(
    currentHost: String,
    currentPort: Int,
    connectionState: ConnectionState,
    onPing: () -> Unit,
    onSave: (String, Int) -> Unit,
    onDismiss: () -> Unit
) {
    var hostText by remember { mutableStateOf(currentHost) }
    var portText by remember { mutableStateOf(currentPort.toString()) }

    AlertDialog(
        onDismissRequest = onDismiss,
        containerColor = ColorCard,
        title = {
            Text(
                text = "HOST SETTINGS",
                fontFamily = FontFamily.Monospace,
                fontWeight = FontWeight.Bold,
                fontSize = 16.sp,
                color = Color.White
            )
        },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                OutlinedTextField(
                    value = hostText,
                    onValueChange = { hostText = it },
                    label = { Text("PC IP Address", color = ColorTextMuted) },
                    singleLine = true,
                    colors = OutlinedTextFieldDefaults.colors(
                        focusedTextColor = Color.White,
                        unfocusedTextColor = Color.White,
                        focusedBorderColor = ColorCyan,
                        unfocusedBorderColor = ColorCardBorder
                    ),
                    modifier = Modifier.fillMaxWidth()
                )

                OutlinedTextField(
                    value = portText,
                    onValueChange = { portText = it },
                    label = { Text("Port", color = ColorTextMuted) },
                    singleLine = true,
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
                    colors = OutlinedTextFieldDefaults.colors(
                        focusedTextColor = Color.White,
                        unfocusedTextColor = Color.White,
                        focusedBorderColor = ColorCyan,
                        unfocusedBorderColor = ColorCardBorder
                    ),
                    modifier = Modifier.fillMaxWidth()
                )

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    val statusStr = when (connectionState) {
                        is ConnectionState.Online -> "ONLINE (${connectionState.latencyMs}ms)"
                        is ConnectionState.Offline -> "OFFLINE"
                        is ConnectionState.Checking -> "PINGING..."
                        is ConnectionState.Idle -> "IDLE"
                    }
                    val statusColor = when (connectionState) {
                        is ConnectionState.Online -> ColorGreen
                        is ConnectionState.Offline -> ColorRed
                        else -> ColorCyan
                    }

                    Text(
                        text = statusStr,
                        fontFamily = FontFamily.Monospace,
                        fontSize = 11.sp,
                        fontWeight = FontWeight.Bold,
                        color = statusColor
                    )

                    OutlinedButton(
                        onClick = onPing,
                        shape = RoundedCornerShape(8.dp)
                    ) {
                        Text("TEST", fontFamily = FontFamily.Monospace, fontSize = 11.sp, color = ColorCyan)
                    }
                }
            }
        },
        confirmButton = {
            Button(
                onClick = {
                    val port = portText.toIntOrNull() ?: 9421
                    onSave(hostText, port)
                },
                colors = ButtonDefaults.buttonColors(containerColor = ColorCyan),
                shape = RoundedCornerShape(8.dp)
            ) {
                Text("SAVE", fontFamily = FontFamily.Monospace, fontWeight = FontWeight.Bold, color = Color(0xFF020617))
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) {
                Text("CLOSE", fontFamily = FontFamily.Monospace, color = ColorTextMuted)
            }
        }
    )
}
