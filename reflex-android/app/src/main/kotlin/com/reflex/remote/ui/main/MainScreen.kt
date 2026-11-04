package com.reflex.remote.ui.main

import android.graphics.BitmapFactory
import android.util.LruCache
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.NavigationBarItemDefaults
import androidx.compose.material3.NavigationRail
import androidx.compose.material3.NavigationRailItem
import androidx.compose.material3.NavigationRailItemDefaults
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.RadioButton
import androidx.compose.material3.RadioButtonDefaults
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.reflex.remote.R
import com.reflex.remote.data.ReflexResponse
import com.reflex.remote.data.model.CommandCategory
import com.reflex.remote.data.model.DesktopContext
import com.reflex.remote.data.model.ReflexCommandItem
import com.reflex.remote.domain.prediction.PredictedCommand
import com.reflex.remote.ui.theme.*
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.withContext
import java.net.HttpURLConnection
import java.net.URL

// ============================================================================
// Main Application Screen (Oto Music Reverse-Engineered Architecture)
// Features: Top Search Bar, Top 4 Action Hub, Mini Player Dock, and
// Full-Screen Media Control Bottom Sheet Scaffold.
// ============================================================================

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MainScreen(
    viewModel: MainScreenViewModel,
    modifier: Modifier = Modifier
) {
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()

    BoxWithConstraints(
        modifier = modifier
            .fillMaxSize()
            .statusBarsPadding()
            .navigationBarsPadding()
    ) {
        // Expanded widths restructure to a navigation rail; compact keeps the bar.
        val useRail = maxWidth >= 600.dp

        Scaffold(
            containerColor = MaterialTheme.colorScheme.background,
            // Insets are applied once on the outer container, so the scaffold must not re-apply them.
            contentWindowInsets = WindowInsets(0, 0, 0, 0),
            topBar = {
                OtoTopSearchBar(
                    query = uiState.topSearchQuery,
                    onQueryChange = { q ->
                        viewModel.setTopSearchQuery(q)
                        if (q.isNotBlank() && uiState.currentTab != ScreenTab.DIRECTORY) {
                            viewModel.setTab(ScreenTab.DIRECTORY)
                        }
                        viewModel.setDirectorySearchQuery(q)
                    },
                    connectionState = uiState.connectionState,
                    onPingClick = { viewModel.pingHost() },
                    onMenuClick = { viewModel.setTab(ScreenTab.SETTINGS) }
                )
            },
            bottomBar = {
                if (!useRail) {
                    Column(modifier = Modifier.fillMaxWidth()) {
                        // Docked Mini Player (Exact Oto Music Bottom Sheet Peek)
                        OtoMiniPlayer(
                            context = uiState.desktopContext,
                            pcHost = uiState.pcHost,
                            pcPort = uiState.pcPort,
                            isDispatching = uiState.isDispatching,
                            onExpandClick = { viewModel.setMediaSheetExpanded(true) },
                            onPlayPauseClick = { viewModel.togglePlayPause() },
                            onNextClick = { viewModel.skipNext() },
                            onPrevClick = { viewModel.skipPrevious() }
                        )

                        // Exact Oto Music Bottom Navigation Bar
                        OtoBottomNavigationBar(
                            currentTab = uiState.currentTab,
                            onTabSelect = { tab -> viewModel.setTab(tab) }
                        )
                    }
                }
            }
        ) { innerPadding ->
            Row(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(innerPadding)
            ) {
                if (useRail) {
                    OtoNavigationRail(
                        currentTab = uiState.currentTab,
                        onTabSelect = { tab -> viewModel.setTab(tab) }
                    )
                }

                Column(
                    modifier = Modifier
                        .weight(1f)
                        .fillMaxHeight()
                ) {
                    Box(
                        modifier = Modifier
                            .weight(1f)
                            .fillMaxWidth()
                    ) {
                        AnimatedContent(
                            targetState = uiState.currentTab,
                            transitionSpec = {
                                fadeIn(animationSpec = tween(220, easing = FastOutSlowInEasing)) togetherWith
                                        fadeOut(animationSpec = tween(180, easing = FastOutSlowInEasing))
                            },
                            label = "TabContentTransition"
                        ) { tab ->
                            when (tab) {
                                ScreenTab.REMOTE -> {
                                    RemoteTabContent(
                                        uiState = uiState,
                                        onMicToggle = { viewModel.toggleListening() },
                                        onCommandDispatch = { cmd -> viewModel.dispatchCommand(cmd) },
                                        onClearHistory = { viewModel.clearHistory() },
                                        onDismissBanner = { viewModel.dismissBanner() },
                                        onRefreshContext = { viewModel.pingHost() },
                                        onOpenMediaSheet = { viewModel.setMediaSheetExpanded(true) },
                                        onOpenDirectory = { viewModel.setTab(ScreenTab.DIRECTORY) }
                                    )
                                }

                                ScreenTab.DIRECTORY -> {
                                    CommandDirectoryTabContent(
                                        desktopContext = uiState.desktopContext,
                                        allItems = uiState.allCommands,
                                        rankedItems = if (uiState.rankedCatalogCommands.isNotEmpty()) uiState.rankedCatalogCommands else uiState.allCommands,
                                        predictedCommands = uiState.predictedCommands,
                                        dynamicCategories = uiState.dynamicCategories,
                                        contextSummary = uiState.predictionContextSummary,
                                        selectedCategory = uiState.directoryCategory,
                                        searchQuery = uiState.directorySearchQuery,
                                        onCategoryChange = { cat -> viewModel.setDirectoryCategory(cat) },
                                        onSearchQueryChange = { q -> viewModel.setDirectorySearchQuery(q) },
                                        onCommandClick = { item -> viewModel.dispatchCommand(item.command) }
                                    )
                                }

                                ScreenTab.SETTINGS -> {
                                    SettingsTabContent(
                                        uiState = uiState,
                                        onSaveSettings = { host, port, mode ->
                                            viewModel.saveSettings(host, port, mode)
                                        },
                                        onPing = { viewModel.pingHost() },
                                        onThemeChange = { mode -> viewModel.setThemeMode(mode) }
                                    )
                                }
                            }
                        }

                        // Full Media Control Bottom Sheet (ModalBottomSheet component)
                        if (uiState.isMediaSheetExpanded) {
                            OtoMediaBottomSheet(
                                context = uiState.desktopContext,
                                pcHost = uiState.pcHost,
                                pcPort = uiState.pcPort,
                                connectionState = uiState.connectionState,
                                onDismiss = { viewModel.setMediaSheetExpanded(false) },
                                onPlayPause = { viewModel.togglePlayPause() },
                                onNext = { viewModel.skipNext() },
                                onPrev = { viewModel.skipPrevious() },
                                onSeek = { sec -> viewModel.seekTo(sec) },
                                onToggleMute = { viewModel.toggleMute() },
                                onVolumeChange = { pct -> viewModel.setVolume(pct) }
                            )
                        }
                    }

                    if (useRail) {
                        OtoMiniPlayer(
                            context = uiState.desktopContext,
                            pcHost = uiState.pcHost,
                            pcPort = uiState.pcPort,
                            isDispatching = uiState.isDispatching,
                            onExpandClick = { viewModel.setMediaSheetExpanded(true) },
                            onPlayPauseClick = { viewModel.togglePlayPause() },
                            onNextClick = { viewModel.skipNext() },
                            onPrevClick = { viewModel.skipPrevious() }
                        )
                    }
                }
            }
        }
    }
}

// ============================================================================
// COMPONENT 1: Top Search Bar (Exact Oto Music Design, Color & Position)
// Pill-shaped container, Hamburger menu, Search placeholder, Cast/Network status
// ============================================================================

@Composable
private fun OtoTopSearchBar(
    query: String,
    onQueryChange: (String) -> Unit,
    connectionState: ConnectionState,
    onPingClick: () -> Unit,
    onMenuClick: () -> Unit
) {
    Surface(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 8.dp)
            .height(56.dp),
        shape = RoundedCornerShape(28.dp),
        color = MaterialTheme.colorScheme.surfaceContainerLow,
        border = androidx.compose.foundation.BorderStroke(1.dp, MaterialTheme.colorScheme.outline)
    ) {
        Row(
            modifier = Modifier
                .fillMaxSize()
                .padding(horizontal = 14.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(10.dp)
        ) {
            // Left: Hamburger Menu Icon Button
            IconButton(
                onClick = onMenuClick,
                modifier = Modifier.size(48.dp)
            ) {
                Icon(
                    painter = painterResource(id = R.drawable.ic_menu),
                    contentDescription = "Menu",
                    tint = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.size(22.dp)
                )
            }

            // Center: Search input text field
            Box(
                modifier = Modifier
                    .weight(1f)
                    .fillMaxHeight(),
                contentAlignment = Alignment.CenterStart
            ) {
                if (query.isEmpty()) {
                    Text(
                        text = "Search your commands",
                        fontSize = 15.sp,
                        fontWeight = FontWeight.Normal,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
                BasicTextField(
                    value = query,
                    onValueChange = onQueryChange,
                    singleLine = true,
                    textStyle = TextStyle(
                        color = MaterialTheme.colorScheme.onSurface,
                        fontSize = 15.sp,
                        fontWeight = FontWeight.Normal
                    ),
                    cursorBrush = SolidColor(MaterialTheme.colorScheme.primary),
                    modifier = Modifier.fillMaxWidth()
                )
            }

            // Right: connection status pill, tap to re-check
            val statusLabel = when (connectionState) {
                is ConnectionState.Online -> "Online, ${connectionState.latencyMs} milliseconds. Tap to re-check"
                is ConnectionState.Offline -> "Offline. Tap to retry"
                ConnectionState.Checking -> "Checking connection"
                ConnectionState.Idle -> "Not connected. Tap to retry"
            }

            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(6.dp),
                modifier = Modifier
                    .clip(RoundedCornerShape(16.dp))
                    .clickable(
                        onClickLabel = "Re-check connection",
                        role = Role.Button,
                        onClick = onPingClick
                    )
                    .heightIn(min = 48.dp)
                    .padding(horizontal = 8.dp)
            ) {
                if (connectionState is ConnectionState.Online) {
                    Text(
                        text = "${connectionState.latencyMs}ms",
                        fontSize = 11.sp,
                        fontFamily = FontFamily.Monospace,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.tertiary
                    )
                }

                Icon(
                    painter = painterResource(
                        id = if (connectionState is ConnectionState.Online) {
                            R.drawable.ic_wifi_connected
                        } else {
                            R.drawable.ic_wifi_disconnected
                        }
                    ),
                    contentDescription = statusLabel,
                    tint = if (connectionState is ConnectionState.Online) {
                        MaterialTheme.colorScheme.tertiary
                    } else {
                        MaterialTheme.colorScheme.onSurfaceVariant
                    },
                    modifier = Modifier.size(20.dp)
                )
            }
        }
    }
}

// ============================================================================
// COMPONENT 2: Quick action hub
// Four shortcuts, each labelled with the action it actually performs.
// ============================================================================

@Composable
private fun OtoTopActionButtons(
    onCommandsClick: () -> Unit,
    onVoiceClick: () -> Unit,
    onMediaClick: () -> Unit,
    onRefreshClick: () -> Unit
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 10.dp),
        horizontalArrangement = Arrangement.SpaceEvenly,
        verticalAlignment = Alignment.CenterVertically
    ) {
        OtoCircularButton(
            title = "Commands",
            backgroundColor = MaterialTheme.colorScheme.secondaryContainer,
            iconTint = MaterialTheme.colorScheme.onSecondaryContainer,
            iconRes = R.drawable.ic_terminal,
            onClick = onCommandsClick
        )

        OtoCircularButton(
            title = "Voice",
            backgroundColor = MaterialTheme.colorScheme.errorContainer,
            iconTint = MaterialTheme.colorScheme.onErrorContainer,
            iconRes = R.drawable.ic_mic,
            onClick = onVoiceClick
        )

        OtoCircularButton(
            title = "Media",
            backgroundColor = MaterialTheme.colorScheme.primaryContainer,
            iconTint = MaterialTheme.colorScheme.onPrimaryContainer,
            iconRes = R.drawable.ic_music_note,
            onClick = onMediaClick
        )

        OtoCircularButton(
            title = "Refresh",
            backgroundColor = MaterialTheme.colorScheme.tertiaryContainer,
            iconTint = MaterialTheme.colorScheme.onTertiaryContainer,
            iconRes = R.drawable.ic_refresh,
            onClick = onRefreshClick
        )
    }
}

@Composable
private fun OtoCircularButton(
    title: String,
    backgroundColor: Color,
    iconTint: Color,
    iconRes: Int,
    onClick: () -> Unit
) {
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(6.dp),
        modifier = Modifier
            .clip(RoundedCornerShape(16.dp))
            .clickable(onClickLabel = title, role = Role.Button, onClick = onClick)
            .padding(horizontal = 6.dp, vertical = 4.dp)
    ) {
        Box(
            modifier = Modifier
                .size(60.dp)
                .clip(CircleShape)
                .background(backgroundColor),
            contentAlignment = Alignment.Center
        ) {
            Icon(
                painter = painterResource(id = iconRes),
                contentDescription = null,
                tint = iconTint,
                modifier = Modifier.size(26.dp)
            )
        }
        Text(
            text = title,
            fontSize = 12.sp,
            fontWeight = FontWeight.Medium,
            color = MaterialTheme.colorScheme.onSurfaceVariant
        )
    }
}

// ============================================================================
// Media Artwork Cache & Formatting Helpers
// ============================================================================

private object BitmapMemoryCache {
    private val maxMemory = (Runtime.getRuntime().maxMemory() / 1024).toInt()
    private val cacheSize = (maxMemory / 8).coerceAtLeast(1024)
    val cache = object : LruCache<String, ImageBitmap>(cacheSize) {
        override fun sizeOf(key: String, value: ImageBitmap): Int {
            return (value.width * value.height * 4) / 1024
        }
    }
}

@Composable
private fun rememberRemoteBitmap(url: String?): ImageBitmap? {
    if (url.isNullOrBlank()) return null
    var bitmap by remember(url) { mutableStateOf(BitmapMemoryCache.cache.get(url)) }

    LaunchedEffect(url) {
        if (bitmap != null) return@LaunchedEffect
        withContext(Dispatchers.IO) {
            try {
                val conn = URL(url).openConnection() as HttpURLConnection
                conn.connectTimeout = 4000
                conn.readTimeout = 5000
                conn.instanceFollowRedirects = true
                conn.setRequestProperty("User-Agent", "Mozilla/5.0 (Linux; Android 10; Mobile) ReflexCompanion/1.0")
                conn.setRequestProperty("Accept", "image/*")
                conn.doInput = true
                conn.connect()
                if (conn.responseCode in 200..299) {
                    val bytes = conn.inputStream.use { it.readBytes() }
                    if (bytes.isNotEmpty()) {
                        val decoded = BitmapFactory.decodeByteArray(bytes, 0, bytes.size)
                        if (decoded != null) {
                            val imgBmp = decoded.asImageBitmap()
                            BitmapMemoryCache.cache.put(url, imgBmp)
                            bitmap = imgBmp
                        }
                    }
                }
            } catch (_: Exception) {
                // Return gracefully without crashing, falling back to vector art
            }
        }
    }
    return bitmap
}

private fun resolveArtUrl(artUrl: String?, pcHost: String, pcPort: Int, trackKey: String): String? {
    if (artUrl.isNullOrBlank()) return null
    val trimmed = artUrl.trim()
    return if (trimmed.startsWith("http://") || trimmed.startsWith("https://")) {
        trimmed
    } else {
        val path = if (trimmed.startsWith("/")) trimmed else "/$trimmed"
        val sep = if (path.contains("?")) "&" else "?"
        "http://$pcHost:$pcPort$path${sep}k=${trackKey.hashCode()}"
    }
}

private fun formatTimestamp(seconds: Float): String {
    if (seconds.isNaN() || seconds < 0f) return "00:00"
    val totalSecs = seconds.toInt()
    val mins = totalSecs / 60
    val secs = totalSecs % 60
    return "%02d:%02d".format(mins, secs)
}

// ============================================================================
// COMPONENT 3: Mini Player Bar (Exact Oto Music Bottom Sheet Peek)
// Sits docked directly above the Bottom Navigation Bar
// ============================================================================

@Composable
private fun OtoMiniPlayer(
    context: DesktopContext?,
    pcHost: String,
    pcPort: Int,
    isDispatching: Boolean,
    onExpandClick: () -> Unit,
    onPlayPauseClick: () -> Unit,
    onNextClick: () -> Unit,
    onPrevClick: () -> Unit
) {
    val isPlaying = context?.media?.isPlaying == true
    val songTitle = context?.media?.title?.ifBlank { null }
        ?: if (context?.activeWindow?.appName?.isNotBlank() == true) {
            context.activeWindow.appName
        } else {
            "Nothing playing"
        }

    val artistName = context?.media?.artist?.ifBlank { null }
        ?: if (context?.activeWindow?.title?.isNotBlank() == true) {
            context.activeWindow.title
        } else {
            "Reflex remote"
        }

    val duration = context?.media?.durationSeconds ?: 0f
    val basePosition = context?.media?.positionSeconds ?: 0f
    var localPosition by remember(basePosition) { mutableFloatStateOf(basePosition) }

    LaunchedEffect(isPlaying, basePosition) {
        if (isPlaying && duration > 0f) {
            while (isActive) {
                delay(500)
                localPosition = (localPosition + 0.5f).coerceAtMost(duration)
            }
        }
    }

    val progressFraction = if (duration > 0f) {
        (localPosition / duration).coerceIn(0f, 1f)
    } else {
        ((context?.volume?.percent ?: 50) / 100f).coerceIn(0.05f, 1f)
    }

    val trackKey = "${context?.media?.title}_${context?.media?.artist}"
    val resolvedArtUrl = resolveArtUrl(context?.media?.artUrl, pcHost, pcPort, trackKey)
    val artBitmap = rememberRemoteBitmap(resolvedArtUrl)

    Surface(
        color = MaterialTheme.colorScheme.surfaceContainer,
        modifier = Modifier
            .fillMaxWidth()
            .height(72.dp)
            .border(
                width = 0.8.dp,
                color = MaterialTheme.colorScheme.outlineVariant,
                shape = RoundedCornerShape(topStart = 16.dp, topEnd = 16.dp)
            )
            .clip(RoundedCornerShape(topStart = 16.dp, topEnd = 16.dp))
            .clickable(
                onClickLabel = "Open player",
                role = Role.Button,
                onClick = onExpandClick
            )
    ) {
        Column(modifier = Modifier.fillMaxSize()) {
            // Real-time progress bar at the very top of Mini Player
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(2.5.dp)
                    .background(MaterialTheme.colorScheme.surfaceContainerHighest)
            ) {
                Box(
                    modifier = Modifier
                        .fillMaxWidth(fraction = progressFraction)
                        .fillMaxHeight()
                        .background(MaterialTheme.colorScheme.primary)
                )
            }

            Row(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(horizontal = 14.dp, vertical = 6.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                // Media Thumbnail with Album Art Styling and concentric border radius & outline depth
                Box(
                    modifier = Modifier
                        .size(46.dp)
                        .clip(RoundedCornerShape(10.dp))
                        .background(MaterialTheme.colorScheme.surfaceContainerLow)
                        .border(
                            1.dp,
                            MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.4f),
                            RoundedCornerShape(10.dp)
                        ),
                    contentAlignment = Alignment.Center
                ) {
                    AnimatedContent(
                        targetState = artBitmap to isPlaying,
                        transitionSpec = {
                            (fadeIn(animationSpec = tween(350, easing = FastOutSlowInEasing)) +
                                scaleIn(
                                    initialScale = 0.88f,
                                    animationSpec = spring(
                                        dampingRatio = Spring.DampingRatioMediumBouncy,
                                        stiffness = Spring.StiffnessMediumLow
                                    )
                                )) togetherWith (fadeOut(animationSpec = tween(180)) +
                                scaleOut(targetScale = 1.05f))
                        },
                        label = "MiniPlayerArtAnimation"
                    ) { (bitmap, playing) ->
                        if (bitmap != null) {
                            Image(
                                bitmap = bitmap,
                                contentDescription = "Album Art",
                                contentScale = ContentScale.Crop,
                                modifier = Modifier.fillMaxSize()
                            )
                        } else {
                            Icon(
                                painter = painterResource(id = if (playing) R.drawable.ic_music_note else R.drawable.ic_reflex_logo),
                                contentDescription = null,
                                tint = if (playing) MaterialTheme.colorScheme.primary else Color.Unspecified,
                                modifier = Modifier.size(24.dp)
                            )
                        }
                    }
                }

                // Title & Subtitle column
                Column(
                    modifier = Modifier.weight(1f),
                    verticalArrangement = Arrangement.Center
                ) {
                    Text(
                        text = songTitle,
                        fontSize = 14.sp,
                        fontWeight = FontWeight.SemiBold,
                        color = MaterialTheme.colorScheme.onSurface,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                    Spacer(modifier = Modifier.height(2.dp))
                    Text(
                        text = artistName,
                        fontSize = 12.sp,
                        fontWeight = FontWeight.Normal,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                }

                // Controls: Previous, Play/Pause, Next
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(2.dp)
                ) {
                    IconButton(
                        onClick = onPrevClick,
                        modifier = Modifier.size(48.dp)
                    ) {
                        Icon(
                            painter = painterResource(id = R.drawable.ic_skip_previous),
                            contentDescription = "Previous Track",
                            tint = MaterialTheme.colorScheme.primary,
                            modifier = Modifier.size(20.dp)
                        )
                    }

                    IconButton(
                        onClick = onPlayPauseClick,
                        modifier = Modifier.size(48.dp)
                    ) {
                        if (isDispatching) {
                            CircularProgressIndicator(
                                modifier = Modifier.size(22.dp),
                                color = MaterialTheme.colorScheme.primary,
                                strokeWidth = 2.5.dp
                            )
                        } else {
                            Box(
                                modifier = Modifier.size(28.dp),
                                contentAlignment = Alignment.Center
                            ) {
                                AnimatedContent(
                                    targetState = isPlaying,
                                    transitionSpec = {
                                        (scaleIn(
                                            initialScale = 0.7f,
                                            animationSpec = spring(
                                                dampingRatio = Spring.DampingRatioMediumBouncy,
                                                stiffness = Spring.StiffnessMedium
                                            )
                                        ) + fadeIn(animationSpec = tween(150)))
                                            .togetherWith(scaleOut(targetScale = 0.7f) + fadeOut(animationSpec = tween(120)))
                                    },
                                    label = "MiniPlayerPlayPauseAnim"
                                ) { playing ->
                                    Icon(
                                        painter = painterResource(id = if (playing) R.drawable.ic_pause else R.drawable.ic_play_arrow),
                                        contentDescription = if (playing) "Pause" else "Play",
                                        tint = MaterialTheme.colorScheme.primary,
                                        modifier = Modifier
                                            .size(28.dp)
                                            .padding(start = if (playing) 0.dp else 2.dp)
                                    )
                                }
                            }
                        }
                    }

                    IconButton(
                        onClick = onNextClick,
                        modifier = Modifier.size(48.dp)
                    ) {
                        Icon(
                            painter = painterResource(id = R.drawable.ic_skip_next),
                            contentDescription = "Next Track",
                            tint = MaterialTheme.colorScheme.primary,
                            modifier = Modifier.size(20.dp)
                        )
                    }
                }
            }
        }
    }
}

// ============================================================================
// COMPONENT 4: Full Media Control Bottom Sheet (Exact Oto Full Player)
// Reverse-engineered from Oto Music full player fragment
// Large artwork, track details, slider, big play button, specs & bottom toolbar
// ============================================================================

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun OtoMediaBottomSheet(
    context: DesktopContext?,
    pcHost: String,
    pcPort: Int,
    connectionState: ConnectionState,
    onDismiss: () -> Unit,
    onPlayPause: () -> Unit,
    onNext: () -> Unit,
    onPrev: () -> Unit,
    onSeek: (Int) -> Unit,
    onToggleMute: () -> Unit,
    onVolumeChange: (Int) -> Unit
) {
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    val isPlaying = context?.media?.isPlaying == true
    val songTitle = context?.media?.title?.ifBlank { null }
        ?: if (context?.activeWindow?.title?.isNotBlank() == true) {
            context.activeWindow.title
        } else {
            "Desktop Audio Stream"
        }

    val artistName = context?.media?.artist?.ifBlank { null }
        ?: if (context?.activeWindow?.appName?.isNotBlank() == true) {
            context.activeWindow.appName
        } else {
            "System Audio"
        }

    val playerEngine = context?.media?.player?.ifBlank { null } ?: "WirePlumber"

    val duration = context?.media?.durationSeconds ?: 0f
    val basePosition = context?.media?.positionSeconds ?: 0f

    var localPosition by remember(basePosition) { mutableFloatStateOf(basePosition) }
    var isDraggingProgress by remember { mutableStateOf(false) }
    var dragProgressPosition by remember { mutableFloatStateOf(0f) }

    LaunchedEffect(isPlaying, basePosition) {
        if (isPlaying && duration > 0f) {
            while (isActive) {
                delay(500)
                if (!isDraggingProgress) {
                    localPosition = (localPosition + 0.5f).coerceAtMost(duration)
                }
            }
        }
    }

    val trackKey = "${context?.media?.title}_${context?.media?.artist}"
    val resolvedArtUrl = resolveArtUrl(context?.media?.artUrl, pcHost, pcPort, trackKey)
    val artBitmap = rememberRemoteBitmap(resolvedArtUrl)

    var currentVolumeSlider by remember(context?.volume?.percent) {
        mutableFloatStateOf((context?.volume?.percent ?: 50).toFloat())
    }

    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = sheetState,
        containerColor = MaterialTheme.colorScheme.background,
        dragHandle = {
            Box(
                modifier = Modifier
                    .padding(top = 12.dp, bottom = 8.dp)
                    .size(width = 38.dp, height = 4.dp)
                    .clip(CircleShape)
                    .background(MaterialTheme.colorScheme.secondaryContainer)
            )
        }
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 24.dp)
                .padding(bottom = 28.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            // Header: Collapse arrow, Title, More options
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                IconButton(onClick = onDismiss) {
                    Icon(
                        painter = painterResource(id = R.drawable.ic_close),
                        contentDescription = "Close player",
                        tint = MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier.size(20.dp)
                    )
                }

                Text(
                    text = "NOW PLAYING",
                    fontSize = 11.sp,
                    fontWeight = FontWeight.Bold,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    letterSpacing = 1.6.sp
                )

                // Keeps the title optically centered against the leading button.
                Spacer(modifier = Modifier.size(48.dp))
            }

            Spacer(modifier = Modifier.height(14.dp))

            // Artwork scales with the sheet, capped so tablets don't get a wall of card.
            Card(
                modifier = Modifier
                    .fillMaxWidth(0.72f)
                    .widthIn(max = 260.dp)
                    .aspectRatio(1f)
                    .clip(RoundedCornerShape(26.dp))
                    .border(
                        1.dp,
                        MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.35f),
                        RoundedCornerShape(26.dp)
                    ),
                colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceContainerHigh),
                shape = RoundedCornerShape(26.dp),
                elevation = CardDefaults.cardElevation(defaultElevation = 0.dp)
            ) {
                Box(
                    modifier = Modifier.fillMaxSize(),
                    contentAlignment = Alignment.Center
                ) {
                    AnimatedContent(
                        targetState = artBitmap to isPlaying,
                        transitionSpec = {
                            (fadeIn(animationSpec = tween(400, easing = FastOutSlowInEasing)) +
                                scaleIn(
                                    initialScale = 0.90f,
                                    animationSpec = spring(
                                        dampingRatio = Spring.DampingRatioMediumBouncy,
                                        stiffness = Spring.StiffnessMediumLow
                                    )
                                )) togetherWith (fadeOut(animationSpec = tween(200)) +
                                scaleOut(targetScale = 1.05f))
                        },
                        label = "FullPlayerArtAnimation"
                    ) { (bitmap, playing) ->
                        if (bitmap != null) {
                            Image(
                                bitmap = bitmap,
                                contentDescription = "Album Artwork",
                                contentScale = ContentScale.Crop,
                                modifier = Modifier.fillMaxSize()
                            )
                        } else {
                            // Concentric soundwave circles canvas fallback
                            Box(
                                modifier = Modifier.fillMaxSize(),
                                contentAlignment = Alignment.Center
                            ) {
                                val ringColor = MaterialTheme.colorScheme.outlineVariant
                                Canvas(modifier = Modifier.fillMaxSize()) {
                                    val center = Offset(size.width / 2f, size.height / 2f)
                                    val maxRadius = size.minDimension / 2f
                                    drawCircle(
                                        color = ringColor.copy(alpha = 0.6f),
                                        radius = maxRadius * 0.9f,
                                        center = center,
                                        style = Stroke(width = 2f)
                                    )
                                    drawCircle(
                                        color = ringColor.copy(alpha = 0.5f),
                                        radius = maxRadius * 0.7f,
                                        center = center,
                                        style = Stroke(width = 2f)
                                    )
                                    drawCircle(
                                        color = ringColor.copy(alpha = 0.4f),
                                        radius = maxRadius * 0.45f,
                                        center = center,
                                        style = Stroke(width = 2f)
                                    )
                                }

                                Icon(
                                    painter = painterResource(id = R.drawable.ic_music_note),
                                    contentDescription = "Audio Art",
                                    tint = if (playing) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.secondaryContainer,
                                    modifier = Modifier.size(80.dp)
                                )
                            }
                        }
                    }
                }
            }

            Spacer(modifier = Modifier.height(20.dp))

            // Track Title & Artist Info
            Text(
                text = songTitle,
                style = MaterialTheme.typography.headlineSmall,
                color = MaterialTheme.colorScheme.onSurface,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )

            Spacer(modifier = Modifier.height(4.dp))

            Text(
                text = "$artistName • $playerEngine",
                fontSize = 14.sp,
                fontWeight = FontWeight.Normal,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )

            Spacer(modifier = Modifier.height(16.dp))

            // ================================================================
            // REAL-TIME PLAYBACK PROGRESS SCRUBBER
            // Features tabular monospace timestamps and smooth scrub handling
            // ================================================================
            if (duration > 0f) {
                val currentSec = if (isDraggingProgress) dragProgressPosition else localPosition
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 4.dp)
                ) {
                    Slider(
                        value = currentSec.coerceIn(0f, duration),
                        onValueChange = {
                            isDraggingProgress = true
                            dragProgressPosition = it
                        },
                        onValueChangeFinished = {
                            isDraggingProgress = false
                            localPosition = dragProgressPosition
                            onSeek(dragProgressPosition.toInt())
                        },
                        valueRange = 0f..duration,
                        colors = SliderDefaults.colors(
                            thumbColor = MaterialTheme.colorScheme.primary,
                            activeTrackColor = MaterialTheme.colorScheme.primary,
                            inactiveTrackColor = MaterialTheme.colorScheme.surfaceContainerHighest
                        ),
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(28.dp)
                    )

                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(horizontal = 4.dp),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Text(
                            text = formatTimestamp(currentSec),
                            fontSize = 12.sp,
                            fontFamily = FontFamily.Monospace,
                            fontWeight = FontWeight.Medium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        Text(
                            text = formatTimestamp(duration),
                            fontSize = 12.sp,
                            fontFamily = FontFamily.Monospace,
                            fontWeight = FontWeight.Medium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
            } else {
                Spacer(modifier = Modifier.height(4.dp))
            }

            Spacer(modifier = Modifier.height(10.dp))

            // Transport controls (Skip Prev, Big Optical Play/Pause, Skip Next)
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.Center,
                verticalAlignment = Alignment.CenterVertically
            ) {
                IconButton(
                    onClick = onPrev,
                    modifier = Modifier.size(56.dp)
                ) {
                    Icon(
                        painter = painterResource(id = R.drawable.ic_skip_previous),
                        contentDescription = "Previous track",
                        tint = MaterialTheme.colorScheme.primary,
                        modifier = Modifier.size(34.dp)
                    )
                }

                Spacer(modifier = Modifier.width(24.dp))

                Box(
                    modifier = Modifier
                        .size(72.dp)
                        .clip(CircleShape)
                        .background(MaterialTheme.colorScheme.primary)
                        .clickable(onClick = onPlayPause),
                    contentAlignment = Alignment.Center
                ) {
                    AnimatedContent(
                        targetState = isPlaying,
                        transitionSpec = {
                            (scaleIn(
                                initialScale = 0.7f,
                                animationSpec = spring(
                                    dampingRatio = Spring.DampingRatioMediumBouncy,
                                    stiffness = Spring.StiffnessMedium
                                )
                            ) + fadeIn(animationSpec = tween(150)))
                                .togetherWith(scaleOut(targetScale = 0.7f) + fadeOut(animationSpec = tween(120)))
                        },
                        label = "FullPlayerPlayPauseAnim"
                    ) { playing ->
                        Icon(
                            painter = painterResource(
                                id = if (playing) R.drawable.ic_pause else R.drawable.ic_play_arrow
                            ),
                            contentDescription = if (playing) "Pause" else "Play",
                            tint = MaterialTheme.colorScheme.onPrimary,
                            modifier = Modifier
                                .size(38.dp)
                                .padding(start = if (playing) 0.dp else 2.dp)
                        )
                    }
                }

                Spacer(modifier = Modifier.width(24.dp))

                IconButton(
                    onClick = onNext,
                    modifier = Modifier.size(56.dp)
                ) {
                    Icon(
                        painter = painterResource(id = R.drawable.ic_skip_next),
                        contentDescription = "Next track",
                        tint = MaterialTheme.colorScheme.primary,
                        modifier = Modifier.size(34.dp)
                    )
                }
            }

            Spacer(modifier = Modifier.height(14.dp))

            // Volume row
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 4.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(10.dp)
            ) {
                IconButton(
                    onClick = onToggleMute,
                    modifier = Modifier.size(40.dp)
                ) {
                    Icon(
                        painter = painterResource(
                            id = if (context?.volume?.isMuted == true) R.drawable.ic_volume_off else R.drawable.ic_volume
                        ),
                        contentDescription = "Mute",
                        tint = if (context?.volume?.isMuted == true) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier.size(18.dp)
                    )
                }

                Slider(
                    value = currentVolumeSlider,
                    onValueChange = { currentVolumeSlider = it },
                    onValueChangeFinished = { onVolumeChange(currentVolumeSlider.toInt()) },
                    valueRange = 0f..100f,
                    colors = SliderDefaults.colors(
                        thumbColor = MaterialTheme.colorScheme.secondary,
                        activeTrackColor = MaterialTheme.colorScheme.secondary,
                        inactiveTrackColor = MaterialTheme.colorScheme.surfaceContainerHighest
                    ),
                    modifier = Modifier.weight(1f)
                )

                IconButton(
                    onClick = { onVolumeChange(100) },
                    modifier = Modifier.size(40.dp)
                ) {
                    Icon(
                        painter = painterResource(id = R.drawable.ic_volume_up),
                        contentDescription = "Max Volume",
                        tint = MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier.size(18.dp)
                    )
                }
            }

            Spacer(modifier = Modifier.height(12.dp))

            // Engine and link readout
            val linkLabel = when (connectionState) {
                is ConnectionState.Online -> "${connectionState.latencyMs} ms"
                is ConnectionState.Offline -> "Offline"
                ConnectionState.Checking -> "Checking link"
                ConnectionState.Idle -> "Not connected"
            }
            Text(
                text = "${playerEngine.uppercase()} • $linkLabel",
                fontSize = 11.sp,
                fontFamily = FontFamily.Monospace,
                fontWeight = FontWeight.Medium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                letterSpacing = 1.2.sp
            )
        }
    }
}

// ============================================================================
// COMPONENT 5: Bottom Navigation Bar (compact widths)
// ============================================================================

@Composable
private fun OtoBottomNavigationBar(
    currentTab: ScreenTab,
    onTabSelect: (ScreenTab) -> Unit
) {
    val itemColors = NavigationBarItemDefaults.colors(
        indicatorColor = MaterialTheme.colorScheme.secondaryContainer,
        selectedIconColor = MaterialTheme.colorScheme.onSecondaryContainer,
        selectedTextColor = MaterialTheme.colorScheme.onSurface,
        unselectedIconColor = MaterialTheme.colorScheme.onSurfaceVariant,
        unselectedTextColor = MaterialTheme.colorScheme.onSurfaceVariant
    )

    NavigationBar(
        containerColor = MaterialTheme.colorScheme.surfaceContainer,
        tonalElevation = 0.dp,
        modifier = Modifier
            .fillMaxWidth()
            .border(
                width = 0.8.dp,
                color = MaterialTheme.colorScheme.outlineVariant,
                shape = RoundedCornerShape(0.dp)
            )
    ) {
        NavigationBarItem(
            selected = currentTab == ScreenTab.REMOTE,
            onClick = { onTabSelect(ScreenTab.REMOTE) },
            icon = {
                Icon(
                    painter = painterResource(id = R.drawable.ic_home),
                    contentDescription = null,
                    modifier = Modifier.size(22.dp)
                )
            },
            label = {
                Text(
                    text = "Home",
                    style = MaterialTheme.typography.labelMedium,
                    fontWeight = if (currentTab == ScreenTab.REMOTE) FontWeight.Bold else FontWeight.Medium
                )
            },
            colors = itemColors
        )

        NavigationBarItem(
            selected = currentTab == ScreenTab.DIRECTORY,
            onClick = { onTabSelect(ScreenTab.DIRECTORY) },
            icon = {
                Icon(
                    painter = painterResource(id = R.drawable.ic_terminal),
                    contentDescription = null,
                    modifier = Modifier.size(22.dp)
                )
            },
            label = {
                Text(
                    text = "Commands",
                    style = MaterialTheme.typography.labelMedium,
                    fontWeight = if (currentTab == ScreenTab.DIRECTORY) FontWeight.Bold else FontWeight.Medium
                )
            },
            colors = itemColors
        )

        NavigationBarItem(
            selected = currentTab == ScreenTab.SETTINGS,
            onClick = { onTabSelect(ScreenTab.SETTINGS) },
            icon = {
                Icon(
                    painter = painterResource(id = R.drawable.ic_settings),
                    contentDescription = null,
                    modifier = Modifier.size(22.dp)
                )
            },
            label = {
                Text(
                    text = "Settings",
                    style = MaterialTheme.typography.labelMedium,
                    fontWeight = if (currentTab == ScreenTab.SETTINGS) FontWeight.Bold else FontWeight.Medium
                )
            },
            colors = itemColors
        )
    }
}

// ============================================================================
// COMPONENT 5b: Navigation Rail (expanded widths)
// Same destinations as the bottom bar, restructured for tablet / foldable.
// ============================================================================

@Composable
private fun OtoNavigationRail(
    currentTab: ScreenTab,
    onTabSelect: (ScreenTab) -> Unit
) {
    val itemColors = NavigationRailItemDefaults.colors(
        indicatorColor = MaterialTheme.colorScheme.primaryContainer,
        selectedIconColor = MaterialTheme.colorScheme.onPrimaryContainer,
        selectedTextColor = MaterialTheme.colorScheme.onSurface,
        unselectedIconColor = MaterialTheme.colorScheme.onSurfaceVariant,
        unselectedTextColor = MaterialTheme.colorScheme.onSurfaceVariant
    )

    NavigationRail(
        containerColor = MaterialTheme.colorScheme.surfaceContainer,
        modifier = Modifier
            .fillMaxHeight()
            .border(0.8.dp, MaterialTheme.colorScheme.outlineVariant)
    ) {
        Spacer(modifier = Modifier.height(8.dp))

        NavigationRailItem(
            selected = currentTab == ScreenTab.REMOTE,
            onClick = { onTabSelect(ScreenTab.REMOTE) },
            icon = {
                Icon(
                    painter = painterResource(id = R.drawable.ic_home),
                    contentDescription = null,
                    modifier = Modifier.size(24.dp)
                )
            },
            label = { Text(text = "Home", fontSize = 11.sp) },
            colors = itemColors
        )

        NavigationRailItem(
            selected = currentTab == ScreenTab.DIRECTORY,
            onClick = { onTabSelect(ScreenTab.DIRECTORY) },
            icon = {
                Icon(
                    painter = painterResource(id = R.drawable.ic_music_note),
                    contentDescription = null,
                    modifier = Modifier.size(24.dp)
                )
            },
            label = { Text(text = "Commands", fontSize = 11.sp) },
            colors = itemColors
        )

        NavigationRailItem(
            selected = currentTab == ScreenTab.SETTINGS,
            onClick = { onTabSelect(ScreenTab.SETTINGS) },
            icon = {
                Icon(
                    painter = painterResource(id = R.drawable.ic_settings),
                    contentDescription = null,
                    modifier = Modifier.size(24.dp)
                )
            },
            label = { Text(text = "Settings", fontSize = 11.sp) },
            colors = itemColors
        )
    }
}

// ============================================================================
// TAB 1 CONTENT: Remote Voice & Desktop Telemetry
// With Oto Music Top 4 Buttons, Carousel Cards ("For You Now", "Late Night"),
// "Hidden gems" quick commands, and Voice input hub
// ============================================================================

@Composable
private fun RemoteTabContent(
    uiState: MainUiState,
    onMicToggle: () -> Unit,
    onCommandDispatch: (String) -> Unit,
    onClearHistory: () -> Unit,
    onDismissBanner: () -> Unit,
    onRefreshContext: () -> Unit,
    onOpenMediaSheet: () -> Unit,
    onOpenDirectory: () -> Unit
) {
    LazyColumn(
        modifier = Modifier
            .fillMaxSize()
            .padding(horizontal = 16.dp),
        contentPadding = PaddingValues(top = 4.dp, bottom = 24.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        // Status banner if error
        if (uiState.statusBanner != null) {
            item {
                StatusBanner(
                    text = uiState.statusBanner,
                    onDismiss = onDismissBanner
                )
            }
        }

        // Top 4 Action Buttons
        item {
            OtoTopActionButtons(
                onCommandsClick = onOpenDirectory,
                onVoiceClick = onMicToggle,
                onMediaClick = onOpenMediaSheet,
                onRefreshClick = onRefreshContext
            )
        }

        // Context readouts: active window and desktop volume (compact side-by-side)
        item {
            CompactHeroStatusRow(
                context = uiState.desktopContext,
                onWindowClick = onRefreshContext,
                onAudioClick = onOpenMediaSheet
            )
        }

        // Real-Time Desktop Workspaces & Apps Overview
        if (uiState.desktopContext?.workspaces?.isNotEmpty() == true) {
            item {
                WorkspaceOverviewSection(
                    context = uiState.desktopContext,
                    onWorkspaceClick = onCommandDispatch,
                    onFocusAppClick = onCommandDispatch
                )
            }
        }

        // Voice Hero Push-to-Talk Hub
        item {
            VoiceHeroSection(
                isListening = uiState.isListening,
                isDispatching = uiState.isDispatching,
                audioLevel = uiState.audioLevel,
                onMicClick = onMicToggle
            )
        }

        // "Hidden gems" Section: Context-Adaptive Quick Commands
        item {
            OtoHiddenGemsSection(
                badge = uiState.contextBadge,
                description = uiState.contextDescription,
                commands = uiState.suggestedCommands,
                onCommandClick = onCommandDispatch
            )
        }

        // Live Voice Transcript & Telemetry
        item {
            LiveTranscriptCard(
                transcript = uiState.currentTranscript,
                isListening = uiState.isListening,
                isDispatching = uiState.isDispatching,
                lastResponse = uiState.lastResponse
            )
        }

        // Activity Log / Dispatch History Section
        item {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    text = "Dispatch history",
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.onSurface
                )

                if (uiState.history.isNotEmpty()) {
                    TextButton(
                        onClick = onClearHistory,
                        modifier = Modifier.heightIn(min = 48.dp)
                    ) {
                        Text(
                            text = "Clear",
                            style = MaterialTheme.typography.labelLarge,
                            color = MaterialTheme.colorScheme.primary
                        )
                    }
                }
            }
        }

        if (uiState.history.isEmpty()) {
            item {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(vertical = 12.dp),
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.spacedBy(4.dp)
                ) {
                    Text(
                        text = "Nothing dispatched yet",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Text(
                        text = "Tap the mic or pick a command to send your first one.",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        textAlign = TextAlign.Center
                    )
                }
            }
        } else {
            items(uiState.history, key = { it.id }) { item ->
                HistoryItemCard(item = item)
            }
        }
    }
}

// ============================================================================
// COMPACT HERO STATUS ROW (Focused Window & Desktop Volume Side-By-Side)
// Replaces the oversized 230dp carousel with a compact 86dp dual card row
// ============================================================================

@Composable
private fun CompactHeroStatusRow(
    context: DesktopContext?,
    onWindowClick: () -> Unit,
    onAudioClick: () -> Unit
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.spacedBy(10.dp)
    ) {
        // Left Card: Focused Window & Workspace
        Surface(
            modifier = Modifier
                .weight(1f)
                .height(86.dp)
                .clip(RoundedCornerShape(18.dp))
                .clickable(
                    onClickLabel = "Refresh desktop focus",
                    role = Role.Button,
                    onClick = onWindowClick
                ),
            color = MaterialTheme.colorScheme.surfaceContainerHigh,
            shape = RoundedCornerShape(18.dp),
            border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.5f))
        ) {
            Column(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(horizontal = 12.dp, vertical = 10.dp),
                verticalArrangement = Arrangement.SpaceBetween
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Row(
                        horizontalArrangement = Arrangement.spacedBy(6.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Box(
                            modifier = Modifier
                                .size(24.dp)
                                .clip(CircleShape)
                                .background(MaterialTheme.colorScheme.primaryContainer),
                            contentAlignment = Alignment.Center
                        ) {
                            Icon(
                                painter = painterResource(id = R.drawable.ic_window),
                                contentDescription = null,
                                tint = MaterialTheme.colorScheme.onPrimaryContainer,
                                modifier = Modifier.size(13.dp)
                            )
                        }
                        Text(
                            text = "FOCUSED",
                            fontSize = 10.sp,
                            fontWeight = FontWeight.Bold,
                            letterSpacing = 1.1.sp,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }

                    val ws = context?.activeWindow?.workspace
                    if (ws != null) {
                        Surface(
                            color = MaterialTheme.colorScheme.secondaryContainer,
                            shape = RoundedCornerShape(6.dp)
                        ) {
                            Text(
                                text = "WS $ws",
                                fontSize = 9.sp,
                                fontWeight = FontWeight.Bold,
                                color = MaterialTheme.colorScheme.onSecondaryContainer,
                                modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp)
                            )
                        }
                    }
                }

                Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
                    Text(
                        text = context?.activeWindow?.appName?.ifBlank { "No focus" } ?: "Standby",
                        style = MaterialTheme.typography.titleSmall,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.onSurface,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                    Text(
                        text = context?.activeWindow?.title?.ifBlank { "Desktop workspace" } ?: "Ready",
                        style = MaterialTheme.typography.bodySmall,
                        fontSize = 11.sp,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                }
            }
        }

        // Right Card: Desktop Volume & Output
        val isMuted = context?.volume?.isMuted == true
        val volPercent = context?.volume?.percent ?: 50

        Surface(
            modifier = Modifier
                .weight(1f)
                .height(86.dp)
                .clip(RoundedCornerShape(18.dp))
                .clickable(
                    onClickLabel = "Open audio controls",
                    role = Role.Button,
                    onClick = onAudioClick
                ),
            color = if (isMuted) MaterialTheme.colorScheme.errorContainer.copy(alpha = 0.35f) else MaterialTheme.colorScheme.secondaryContainer,
            shape = RoundedCornerShape(18.dp),
            border = BorderStroke(
                1.dp,
                if (isMuted) MaterialTheme.colorScheme.error.copy(alpha = 0.4f) else MaterialTheme.colorScheme.primary.copy(alpha = 0.25f)
            )
        ) {
            Column(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(horizontal = 12.dp, vertical = 10.dp),
                verticalArrangement = Arrangement.SpaceBetween
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Row(
                        horizontalArrangement = Arrangement.spacedBy(6.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Box(
                            modifier = Modifier
                                .size(24.dp)
                                .clip(CircleShape)
                                .background(if (isMuted) MaterialTheme.colorScheme.errorContainer else MaterialTheme.colorScheme.primaryContainer),
                            contentAlignment = Alignment.Center
                        ) {
                            Icon(
                                painter = painterResource(
                                    id = if (isMuted) R.drawable.ic_volume_off else R.drawable.ic_volume_up
                                ),
                                contentDescription = null,
                                tint = if (isMuted) MaterialTheme.colorScheme.onErrorContainer else MaterialTheme.colorScheme.onPrimaryContainer,
                                modifier = Modifier.size(13.dp)
                            )
                        }
                        Text(
                            text = "VOLUME",
                            fontSize = 10.sp,
                            fontWeight = FontWeight.Bold,
                            letterSpacing = 1.1.sp,
                            color = MaterialTheme.colorScheme.onSecondaryContainer
                        )
                    }

                    Surface(
                        color = if (isMuted) MaterialTheme.colorScheme.error.copy(alpha = 0.2f) else MaterialTheme.colorScheme.primary.copy(alpha = 0.15f),
                        shape = RoundedCornerShape(6.dp)
                    ) {
                        Text(
                            text = if (isMuted) "MUTED" else "$volPercent%",
                            fontSize = 10.sp,
                            fontFamily = FontFamily.Monospace,
                            fontWeight = FontWeight.Bold,
                            color = if (isMuted) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.primary,
                            modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp)
                        )
                    }
                }

                // Volume Bar & Status line
                Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    Text(
                        text = if (isMuted) "Output Muted" else "Default Sink",
                        fontSize = 11.sp,
                        fontWeight = FontWeight.Medium,
                        color = MaterialTheme.colorScheme.onSecondaryContainer
                    )

                    // Sleek Mini Volume Bar
                    Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(4.dp)
                            .clip(RoundedCornerShape(2.dp))
                            .background(MaterialTheme.colorScheme.surfaceContainerHighest)
                    ) {
                        Box(
                            modifier = Modifier
                                .fillMaxWidth(if (isMuted) 0f else (volPercent / 100f).coerceIn(0f, 1f))
                                .height(4.dp)
                                .clip(RoundedCornerShape(2.dp))
                                .background(if (isMuted) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.primary)
                        )
                    }
                }
            }
        }
    }
}

// ============================================================================
// WORKSPACE & RUNNING APPS OVERVIEW (Hyprland Real-Time State)
// Displays open workspaces, active workspace badge, and running apps with direct
// one-tap focus & workspace switching.
// ============================================================================

@Composable
private fun WorkspaceOverviewSection(
    context: DesktopContext?,
    onWorkspaceClick: (String) -> Unit,
    onFocusAppClick: (String) -> Unit
) {
    val workspaces = context?.workspaces ?: emptyList()
    if (workspaces.isEmpty()) return

    val activeWsId = context?.activeWorkspace ?: 1

    Column(
        modifier = Modifier.fillMaxWidth(),
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        // Section Header
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(6.dp)
            ) {
                Box(
                    modifier = Modifier
                        .size(20.dp)
                        .clip(CircleShape)
                        .background(MaterialTheme.colorScheme.primary.copy(alpha = 0.15f)),
                    contentAlignment = Alignment.Center
                ) {
                    Icon(
                        painter = painterResource(id = R.drawable.ic_window),
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.primary,
                        modifier = Modifier.size(12.dp)
                    )
                }
                Text(
                    text = "PC WORKSPACES & APPS",
                    style = MaterialTheme.typography.labelSmall,
                    fontWeight = FontWeight.Bold,
                    letterSpacing = 1.1.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }

            Surface(
                color = MaterialTheme.colorScheme.primaryContainer,
                shape = RoundedCornerShape(8.dp)
            ) {
                Text(
                    text = "ACTIVE • WS $activeWsId",
                    style = MaterialTheme.typography.labelSmall,
                    fontWeight = FontWeight.Bold,
                    color = MaterialTheme.colorScheme.onPrimaryContainer,
                    modifier = Modifier.padding(horizontal = 8.dp, vertical = 2.dp)
                )
            }
        }

        // Horizontal Row of Workspace Cards
        LazyRow(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(10.dp),
            contentPadding = PaddingValues(horizontal = 2.dp, vertical = 2.dp)
        ) {
            items(workspaces, key = { it.id }) { ws ->
                val isActiveWs = ws.isActive || ws.id == activeWsId
                Surface(
                    modifier = Modifier
                        .widthIn(min = 150.dp, max = 220.dp)
                        .clip(RoundedCornerShape(16.dp))
                        .clickable(
                            onClickLabel = "Switch to Workspace ${ws.id}",
                            role = Role.Button
                        ) {
                            onWorkspaceClick("workspace ${ws.id}")
                        },
                    color = if (isActiveWs) {
                        MaterialTheme.colorScheme.surfaceContainerHighest
                    } else {
                        MaterialTheme.colorScheme.surfaceContainerHigh
                    },
                    shape = RoundedCornerShape(16.dp),
                    border = BorderStroke(
                        width = if (isActiveWs) 1.5.dp else 1.dp,
                        color = if (isActiveWs) {
                            MaterialTheme.colorScheme.primary
                        } else {
                            MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.4f)
                        }
                    )
                ) {
                    Column(
                        modifier = Modifier.padding(12.dp),
                        verticalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        // Card Top: Workspace ID badge + Window count / active indicator
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.SpaceBetween,
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Surface(
                                color = if (isActiveWs) {
                                    MaterialTheme.colorScheme.primary
                                } else {
                                    MaterialTheme.colorScheme.secondaryContainer
                                },
                                shape = RoundedCornerShape(6.dp)
                            ) {
                                Text(
                                    text = "WS ${ws.id}",
                                    fontSize = 11.sp,
                                    fontWeight = FontWeight.Bold,
                                    color = if (isActiveWs) {
                                        MaterialTheme.colorScheme.onPrimary
                                    } else {
                                        MaterialTheme.colorScheme.onSecondaryContainer
                                    },
                                    modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp)
                                )
                            }

                            if (isActiveWs) {
                                Row(
                                    verticalAlignment = Alignment.CenterVertically,
                                    horizontalArrangement = Arrangement.spacedBy(4.dp)
                                ) {
                                    Box(
                                        modifier = Modifier
                                            .size(7.dp)
                                            .clip(CircleShape)
                                            .background(MaterialTheme.colorScheme.primary)
                                    )
                                    Text(
                                        text = "Active",
                                        fontSize = 11.sp,
                                        fontWeight = FontWeight.SemiBold,
                                        color = MaterialTheme.colorScheme.primary
                                    )
                                }
                            } else {
                                Text(
                                    text = "${ws.windowCount} ${if (ws.windowCount == 1) "app" else "apps"}",
                                    fontSize = 11.sp,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant
                                )
                            }
                        }

                        // App list inside this workspace
                        if (ws.windows.isEmpty()) {
                            Text(
                                text = "Empty workspace",
                                fontSize = 11.sp,
                                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.7f),
                                fontStyle = androidx.compose.ui.text.font.FontStyle.Italic
                            )
                        } else {
                            Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
                                for (w in ws.windows.take(3)) {
                                    Surface(
                                        color = if (w.isActive) {
                                            MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.5f)
                                        } else {
                                            MaterialTheme.colorScheme.surfaceContainer
                                        },
                                        shape = RoundedCornerShape(8.dp),
                                        modifier = Modifier
                                            .fillMaxWidth()
                                            .clip(RoundedCornerShape(8.dp))
                                            .clickable(
                                                onClickLabel = "Focus ${w.appName}",
                                                role = Role.Button
                                            ) {
                                                onFocusAppClick("focus ${w.appName.lowercase().takeWhile { it != '(' }.trim()}")
                                            }
                                    ) {
                                        Row(
                                            modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp),
                                            verticalAlignment = Alignment.CenterVertically,
                                            horizontalArrangement = Arrangement.spacedBy(6.dp)
                                        ) {
                                            Box(
                                                modifier = Modifier
                                                    .size(6.dp)
                                                    .clip(CircleShape)
                                                    .background(
                                                        if (w.isActive) MaterialTheme.colorScheme.primary
                                                        else MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.5f)
                                                    )
                                            )
                                            Column(modifier = Modifier.weight(1f)) {
                                                Text(
                                                    text = w.appName,
                                                    fontSize = 11.sp,
                                                    fontWeight = if (w.isActive) FontWeight.Bold else FontWeight.Medium,
                                                    color = MaterialTheme.colorScheme.onSurface,
                                                    maxLines = 1,
                                                    overflow = TextOverflow.Ellipsis
                                                )
                                                if (w.title.isNotBlank()) {
                                                    Text(
                                                        text = w.title,
                                                        fontSize = 9.sp,
                                                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                                                        maxLines = 1,
                                                        overflow = TextOverflow.Ellipsis
                                                    )
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// "SUGGESTED FOR YOU" SECTION (Adaptive Responsive 2-Column Grid)
// Renders context-adaptive quick shortcuts in a clean non-squished layout
// ============================================================================

@Composable
private fun OtoHiddenGemsSection(
    badge: String,
    description: String,
    commands: List<ReflexCommandItem>,
    onCommandClick: (String) -> Unit
) {
    Column(
        modifier = Modifier.fillMaxWidth(),
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        // 1. Adaptive Header Row with Title and Constrained Badge
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                text = "Suggested for you",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold,
                color = MaterialTheme.colorScheme.onSurface
            )

            if (badge.isNotBlank()) {
                Surface(
                    color = MaterialTheme.colorScheme.secondaryContainer,
                    shape = RoundedCornerShape(8.dp),
                    modifier = Modifier.widthIn(max = 160.dp)
                ) {
                    Text(
                        text = badge,
                        style = MaterialTheme.typography.labelSmall,
                        fontWeight = FontWeight.Bold,
                        color = MaterialTheme.colorScheme.onSecondaryContainer,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.padding(horizontal = 8.dp, vertical = 3.dp)
                    )
                }
            }
        }

        // Subtitle context description
        if (description.isNotBlank()) {
            Text(
                text = description,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
        }

        if (commands.isEmpty()) {
            Surface(
                color = MaterialTheme.colorScheme.surfaceContainerHigh,
                shape = RoundedCornerShape(14.dp),
                modifier = Modifier.fillMaxWidth()
            ) {
                Text(
                    text = "Awaiting desktop telemetry for contextual shortcuts...",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.padding(14.dp)
                )
            }
        } else {
            // Adaptive 2-Column Grid (Chunked into pairs of 2)
            val chunked = remember(commands) { commands.take(6).chunked(2) }
            Column(
                modifier = Modifier.fillMaxWidth(),
                verticalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                for (rowItems in chunked) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        for (item in rowItems) {
                            val iconRes = remember(item.command, item.category) {
                                when {
                                    item.command.contains("volume") || item.category == CommandCategory.AUDIO -> R.drawable.ic_volume_up
                                    item.command == "mute" || item.command == "unmute" || item.command == "toggle_mute" -> R.drawable.ic_volume_off
                                    item.command == "play" || item.command == "pause" || item.command == "media_toggle" -> R.drawable.ic_play_arrow
                                    item.command.contains("next") -> R.drawable.ic_skip_next
                                    item.command.contains("prev") -> R.drawable.ic_skip_previous
                                    item.command.contains("screenshot") || item.category == CommandCategory.WINDOW -> R.drawable.ic_window
                                    else -> R.drawable.ic_terminal
                                }
                            }

                            Surface(
                                color = MaterialTheme.colorScheme.surfaceContainerHigh,
                                shape = RoundedCornerShape(14.dp),
                                border = BorderStroke(1.dp, MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.4f)),
                                modifier = Modifier
                                    .weight(1f)
                                    .height(56.dp)
                                    .clip(RoundedCornerShape(14.dp))
                                    .clickable(
                                        onClickLabel = "Run ${item.title}",
                                        role = Role.Button
                                    ) { onCommandClick(item.command) }
                            ) {
                                Row(
                                    modifier = Modifier
                                        .fillMaxSize()
                                        .padding(horizontal = 10.dp, vertical = 6.dp),
                                    verticalAlignment = Alignment.CenterVertically,
                                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                                ) {
                                    Box(
                                        modifier = Modifier
                                            .size(34.dp)
                                            .clip(RoundedCornerShape(8.dp))
                                            .background(MaterialTheme.colorScheme.secondaryContainer),
                                        contentAlignment = Alignment.Center
                                    ) {
                                        Icon(
                                            painter = painterResource(id = iconRes),
                                            contentDescription = null,
                                            tint = MaterialTheme.colorScheme.onSecondaryContainer,
                                            modifier = Modifier.size(17.dp)
                                        )
                                    }

                                    Column(
                                        modifier = Modifier.weight(1f),
                                        verticalArrangement = Arrangement.Center
                                    ) {
                                        Text(
                                            text = item.title,
                                            fontSize = 12.sp,
                                            fontWeight = FontWeight.SemiBold,
                                            color = MaterialTheme.colorScheme.onSurface,
                                            maxLines = 1,
                                            overflow = TextOverflow.Ellipsis
                                        )
                                        Text(
                                            text = item.command,
                                            fontSize = 10.sp,
                                            fontFamily = FontFamily.Monospace,
                                            color = MaterialTheme.colorScheme.primary,
                                            maxLines = 1,
                                            overflow = TextOverflow.Ellipsis
                                        )
                                    }
                                }
                            }
                        }
                        // If single odd item in row, fill the other half with empty space
                        if (rowItems.size == 1) {
                            Spacer(modifier = Modifier.weight(1f))
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// VOICE HERO SECTION (Push-To-Talk with sleek ripple pulses)
// ============================================================================

@Composable
private fun VoiceHeroSection(
    isListening: Boolean,
    isDispatching: Boolean,
    audioLevel: Float,
    onMicClick: () -> Unit
) {
    // Only animates while there is something to show, so idle stays off the frame clock.
    val pulseActive = isListening || isDispatching
    val pulseScale by animateFloatAsState(
        targetValue = if (pulseActive) 1.22f else 1f,
        animationSpec = if (pulseActive) {
            infiniteRepeatable(
                animation = tween(850, easing = FastOutSlowInEasing),
                repeatMode = RepeatMode.Reverse
            )
        } else {
            tween(150)
        },
        label = "pulseScale"
    )

    val dynamicAudioScale by animateFloatAsState(
        targetValue = if (isListening) (1.0f + (audioLevel / 18f)).coerceIn(1.0f, 1.45f) else 1.0f,
        animationSpec = spring(
            dampingRatio = Spring.DampingRatioMediumBouncy,
            stiffness = Spring.StiffnessMediumLow
        ),
        label = "dynamicAudioScale"
    )

    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        modifier = Modifier.padding(vertical = 4.dp)
    ) {
        Box(
            contentAlignment = Alignment.Center,
            modifier = Modifier.size(110.dp)
        ) {
            if (isListening) {
                Box(
                    modifier = Modifier
                        .size(104.dp)
                        .scale(pulseScale)
                        .clip(CircleShape)
                        .background(MaterialTheme.colorScheme.errorContainer.copy(alpha = 0.5f))
                )
                Box(
                    modifier = Modifier
                        .size(90.dp)
                        .scale(dynamicAudioScale)
                        .clip(CircleShape)
                        .background(MaterialTheme.colorScheme.errorContainer.copy(alpha = 0.8f))
                )
            } else if (isDispatching) {
                Box(
                    modifier = Modifier
                        .size(96.dp)
                        .scale(pulseScale)
                        .clip(CircleShape)
                        .background(MaterialTheme.colorScheme.secondaryContainer.copy(alpha = 0.6f))
                )
            }

            // Big Push-To-Talk Button
            Surface(
                modifier = Modifier
                    .size(74.dp)
                    .clip(CircleShape)
                    .clickable(
                        onClickLabel = if (isListening) "Stop listening" else "Start listening",
                        role = Role.Button,
                        onClick = onMicClick
                    ),
                shape = CircleShape,
                shadowElevation = 0.dp,
                color = Color.Transparent
            ) {
                val buttonGradient = when {
                    isListening -> Brush.linearGradient(listOf(MaterialTheme.colorScheme.errorContainer, MaterialTheme.colorScheme.error))
                    isDispatching -> Brush.linearGradient(listOf(MaterialTheme.colorScheme.secondaryContainer, MaterialTheme.colorScheme.secondary))
                    else -> Brush.linearGradient(listOf(MaterialTheme.colorScheme.surfaceContainerHigh, MaterialTheme.colorScheme.surfaceContainerHighest))
                }

                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .background(buttonGradient)
                        .border(1.dp, MaterialTheme.colorScheme.outline, CircleShape),
                    contentAlignment = Alignment.Center
                ) {
                    if (isDispatching) {
                        CircularProgressIndicator(
                            modifier = Modifier.size(32.dp),
                            color = MaterialTheme.colorScheme.primary,
                            strokeWidth = 3.dp
                        )
                    } else {
                        Icon(
                            painter = painterResource(id = R.drawable.ic_mic),
                            contentDescription = null,
                            tint = if (isListening) MaterialTheme.colorScheme.onErrorContainer else MaterialTheme.colorScheme.primary,
                            modifier = Modifier.size(32.dp)
                        )
                    }
                }
            }
        }

        Spacer(modifier = Modifier.height(2.dp))

        val statusText = when {
            isDispatching -> "Sending to PC"
            isListening -> "Listening — tap to stop"
            else -> "Tap to speak"
        }

        Text(
            text = statusText,
            style = MaterialTheme.typography.labelMedium,
            color = if (isListening) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.onSurfaceVariant
        )
    }
}

// ============================================================================
// LIVE TRANSCRIPT CARD & ACTIVITY HISTORY CARDS
// ============================================================================

@Composable
private fun LiveTranscriptCard(
    transcript: String,
    isListening: Boolean,
    isDispatching: Boolean,
    lastResponse: ReflexResponse?
) {
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(20.dp))
            .border(1.dp, MaterialTheme.colorScheme.outline, RoundedCornerShape(20.dp)),
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface),
        shape = RoundedCornerShape(20.dp)
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
                    text = "Transcript",
                    style = MaterialTheme.typography.titleSmall,
                    color = MaterialTheme.colorScheme.onSurface
                )

                if (lastResponse != null) {
                    val badgeColor = if (lastResponse.isSuccess) MaterialTheme.colorScheme.tertiary else MaterialTheme.colorScheme.error
                    Surface(
                        color = badgeColor.copy(alpha = 0.15f),
                        shape = RoundedCornerShape(999.dp)
                    ) {
                        Text(
                            text = lastResponse.status.uppercase(),
                            /* monospace telemetry */
                            fontSize = 12.sp,
                            fontWeight = FontWeight.Bold,
                            color = badgeColor,
                            modifier = Modifier.padding(horizontal = 8.dp, vertical = 3.dp)
                        )
                    }
                }
            }

            Spacer(modifier = Modifier.height(8.dp))

            val displayTranscript = when {
                isDispatching -> "Sending to PC…"
                transcript.isNotBlank() -> transcript
                isListening -> "Listening…"
                else -> "Say \"sound 50\", \"next\", \"unmute\", or \"pause\""
            }

            Text(
                text = displayTranscript,
                style = MaterialTheme.typography.bodyLarge,
                color = if (transcript.isNotBlank() || isDispatching) {
                    MaterialTheme.colorScheme.onSurface
                } else {
                    MaterialTheme.colorScheme.onSurfaceVariant
                }
            )

            if (lastResponse != null) {
                Spacer(modifier = Modifier.height(10.dp))
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(10.dp))
                        .background(MaterialTheme.colorScheme.surfaceContainerLow)
                        .padding(horizontal = 10.dp, vertical = 6.dp),
                    horizontalArrangement = Arrangement.spacedBy(12.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        text = lastResponse.action ?: lastResponse.reason ?: "Executed",
                        fontSize = 11.sp,
                        fontFamily = FontFamily.Monospace,
                        color = MaterialTheme.colorScheme.primary,
                        fontWeight = FontWeight.Bold,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.weight(1f, fill = false)
                    )
                    Text(
                        text = "${lastResponse.durationUs} us PC / ${lastResponse.roundTripMs} ms net",
                        fontSize = 11.sp,
                        fontFamily = FontFamily.Monospace,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 1
                    )
                }
            }
        }
    }
}

@Composable
private fun HistoryItemCard(item: CommandLogItem) {
    Surface(
        color = MaterialTheme.colorScheme.surfaceContainerLow,
        shape = RoundedCornerShape(14.dp),
        modifier = Modifier
            .fillMaxWidth()
            .border(0.8.dp, MaterialTheme.colorScheme.outlineVariant, RoundedCornerShape(14.dp))
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 12.dp, vertical = 10.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(10.dp)
        ) {
            Box(
                modifier = Modifier
                    .size(8.dp)
                    .clip(CircleShape)
                    .background(if (item.isError) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.tertiary)
            )

            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = item.command,
                    fontSize = 13.sp,
                    fontFamily = FontFamily.Monospace,
                    fontWeight = FontWeight.Bold,
                    color = MaterialTheme.colorScheme.onSurface,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
                Text(
                    text = when {
                        item.isError -> item.errorMessage ?: "Failed"
                        else -> item.response?.action ?: "Sent"
                    },
                    fontSize = 11.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
            }

            Text(
                text = item.timestamp,
                fontSize = 11.sp,
                fontFamily = FontFamily.Monospace,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
    }
}

@Composable
private fun StatusBanner(
    text: String,
    onDismiss: () -> Unit
) {
    Surface(
        color = MaterialTheme.colorScheme.errorContainer,
        shape = RoundedCornerShape(14.dp),
        modifier = Modifier
            .fillMaxWidth()
            .border(1.dp, MaterialTheme.colorScheme.error, RoundedCornerShape(14.dp))
            .clickable(onClick = onDismiss)
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 14.dp, vertical = 10.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(10.dp)
        ) {
            Icon(
                painter = painterResource(id = R.drawable.ic_alert),
                contentDescription = "Alert",
                tint = MaterialTheme.colorScheme.onErrorContainer,
                modifier = Modifier.size(18.dp)
            )
            Text(
                text = text,
                fontSize = 12.sp,
                /* monospace telemetry */
                color = MaterialTheme.colorScheme.onErrorContainer,
                modifier = Modifier.weight(1f)
            )
            Icon(
                painter = painterResource(id = R.drawable.ic_close),
                contentDescription = "Dismiss",
                tint = MaterialTheme.colorScheme.onErrorContainer,
                modifier = Modifier.size(16.dp)
            )
        }
    }
}

// ============================================================================
// TAB 2: COMMAND DIRECTORY (Styled in Oto Music Palette)
// ============================================================================

@Composable
private fun CommandDirectoryTabContent(
    desktopContext: DesktopContext?,
    allItems: List<ReflexCommandItem>,
    rankedItems: List<ReflexCommandItem>,
    predictedCommands: List<PredictedCommand>,
    dynamicCategories: List<CommandCategory>,
    contextSummary: String,
    selectedCategory: CommandCategory,
    searchQuery: String,
    onCategoryChange: (CommandCategory) -> Unit,
    onSearchQueryChange: (String) -> Unit,
    onCommandClick: (ReflexCommandItem) -> Unit
) {
    val effectiveList = if (rankedItems.isNotEmpty()) rankedItems else allItems
    val filteredItems = remember(effectiveList, selectedCategory, searchQuery) {
        effectiveList.filter { item ->
            val matchesCategory = (selectedCategory == CommandCategory.ALL || item.category == selectedCategory)
            val matchesSearch = if (searchQuery.isBlank()) {
                true
            } else {
                val q = searchQuery.trim().lowercase()
                item.title.lowercase().contains(q) ||
                        item.command.lowercase().contains(q) ||
                        item.description.lowercase().contains(q) ||
                        item.category.label.lowercase().contains(q) ||
                        item.aliases.any { it.lowercase().contains(q) }
            }
            matchesCategory && matchesSearch
        }
    }

    val listState = rememberLazyListState()

    LaunchedEffect(selectedCategory, searchQuery) {
        listState.scrollToItem(0)
    }

    LazyColumn(
        state = listState,
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(horizontal = 16.dp, vertical = 8.dp),
        verticalArrangement = Arrangement.spacedBy(10.dp)
    ) {
        // 1. Dynamic Context & Prediction Header
        if (contextSummary.isNotBlank()) {
            item {
                Surface(
                    color = MaterialTheme.colorScheme.surfaceContainer,
                    shape = RoundedCornerShape(16.dp),
                    modifier = Modifier
                        .fillMaxWidth()
                        .border(
                            1.dp,
                            MaterialTheme.colorScheme.outlineVariant,
                            RoundedCornerShape(16.dp)
                        )
                ) {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(horizontal = 14.dp, vertical = 10.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(10.dp)
                    ) {
                        Box(
                            modifier = Modifier
                                .size(34.dp)
                                .clip(CircleShape)
                                .background(MaterialTheme.colorScheme.primaryContainer),
                            contentAlignment = Alignment.Center
                        ) {
                            Icon(
                                painter = painterResource(id = R.drawable.ic_trending_up),
                                contentDescription = null,
                                tint = MaterialTheme.colorScheme.onPrimaryContainer,
                                modifier = Modifier.size(18.dp)
                            )
                        }
                        Column(modifier = Modifier.weight(1f)) {
                            Text(
                                text = "ADAPTIVE PREDICTION ENGINE",
                                fontSize = 10.sp,
                                fontWeight = FontWeight.Bold,
                                letterSpacing = 1.2.sp,
                                color = MaterialTheme.colorScheme.primary
                            )
                            Text(
                                text = contextSummary,
                                fontSize = 12.sp,
                                fontWeight = FontWeight.Medium,
                                color = MaterialTheme.colorScheme.onSurface,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis
                            )
                        }
                        Surface(
                            color = MaterialTheme.colorScheme.tertiaryContainer,
                            shape = RoundedCornerShape(8.dp)
                        ) {
                            Text(
                                text = "ACTIVE",
                                fontSize = 10.sp,
                                fontWeight = FontWeight.Bold,
                                color = MaterialTheme.colorScheme.onTertiaryContainer,
                                modifier = Modifier.padding(horizontal = 6.dp, vertical = 3.dp)
                            )
                        }
                    }
                }
            }
        }

        // Live Workspace Quick-Filter / Switcher Bar
        if (desktopContext?.workspaces?.isNotEmpty() == true && searchQuery.isBlank()) {
            item {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .horizontalScroll(rememberScrollState()),
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        text = "WORKSPACES:",
                        fontSize = 10.sp,
                        fontWeight = FontWeight.Bold,
                        letterSpacing = 1.1.sp,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    for (ws in desktopContext.workspaces) {
                        val isAct = ws.isActive || ws.id == desktopContext.activeWorkspace
                        Surface(
                            color = if (isAct) MaterialTheme.colorScheme.primaryContainer else MaterialTheme.colorScheme.surfaceContainerHigh,
                            shape = RoundedCornerShape(8.dp),
                            border = BorderStroke(
                                1.dp,
                                if (isAct) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.4f)
                            ),
                            modifier = Modifier
                                .clip(RoundedCornerShape(8.dp))
                                .clickable {
                                    onCommandClick(
                                        ReflexCommandItem(
                                            id = "dir_ws_${ws.id}",
                                            title = "Workspace ${ws.id}",
                                            command = "workspace ${ws.id}",
                                            category = CommandCategory.WINDOW,
                                            description = "Switch to Workspace ${ws.id}"
                                        )
                                    )
                                }
                        ) {
                            val appSummary = ws.windows.firstOrNull()?.appName ?: "Empty"
                            Text(
                                text = "WS ${ws.id} ($appSummary)",
                                fontSize = 11.sp,
                                fontWeight = if (isAct) FontWeight.Bold else FontWeight.Medium,
                                color = if (isAct) MaterialTheme.colorScheme.onPrimaryContainer else MaterialTheme.colorScheme.onSurface,
                                modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp)
                            )
                        }
                    }
                }
            }
        }

        // 2. Predicted Next Actions Carousel (Horizontal LazyRow)
        if (predictedCommands.isNotEmpty() && searchQuery.isBlank()) {
            item {
                Column(
                    modifier = Modifier.fillMaxWidth(),
                    verticalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(horizontal = 2.dp),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Text(
                            text = "PREDICTED NEXT ACTIONS",
                            fontSize = 11.sp,
                            fontWeight = FontWeight.Bold,
                            letterSpacing = 1.2.sp,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        Text(
                            text = "${predictedCommands.size} PREDICTIONS",
                            fontSize = 10.sp,
                            fontFamily = FontFamily.Monospace,
                            fontWeight = FontWeight.Bold,
                            color = MaterialTheme.colorScheme.primary
                        )
                    }

                    LazyRow(
                        horizontalArrangement = Arrangement.spacedBy(10.dp),
                        contentPadding = PaddingValues(vertical = 4.dp)
                    ) {
                        items(predictedCommands, key = { it.commandItem.id }) { pred ->
                            Surface(
                                color = MaterialTheme.colorScheme.surfaceContainerHigh,
                                shape = RoundedCornerShape(16.dp),
                                modifier = Modifier
                                    .width(220.dp)
                                    .border(
                                        width = 1.dp,
                                        color = MaterialTheme.colorScheme.primary.copy(alpha = 0.35f),
                                        shape = RoundedCornerShape(16.dp)
                                    )
                                    .clip(RoundedCornerShape(16.dp))
                                    .clickable { onCommandClick(pred.commandItem) }
                            ) {
                                Column(
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .padding(12.dp),
                                    verticalArrangement = Arrangement.spacedBy(6.dp)
                                ) {
                                    Row(
                                        modifier = Modifier.fillMaxWidth(),
                                        horizontalArrangement = Arrangement.SpaceBetween,
                                        verticalAlignment = Alignment.CenterVertically
                                    ) {
                                        Box(
                                            modifier = Modifier
                                                .size(28.dp)
                                                .clip(CircleShape)
                                                .background(MaterialTheme.colorScheme.secondaryContainer),
                                            contentAlignment = Alignment.Center
                                        ) {
                                            Icon(
                                                painter = painterResource(id = R.drawable.ic_terminal),
                                                contentDescription = null,
                                                tint = MaterialTheme.colorScheme.onSecondaryContainer,
                                                modifier = Modifier.size(16.dp)
                                            )
                                        }

                                        Surface(
                                            color = MaterialTheme.colorScheme.primary.copy(alpha = 0.12f),
                                            shape = RoundedCornerShape(6.dp)
                                        ) {
                                            Text(
                                                text = pred.reason,
                                                fontSize = 9.sp,
                                                fontWeight = FontWeight.Bold,
                                                color = MaterialTheme.colorScheme.primary,
                                                maxLines = 1,
                                                overflow = TextOverflow.Ellipsis,
                                                modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp)
                                            )
                                        }
                                    }

                                    Text(
                                        text = pred.commandItem.title,
                                        fontSize = 13.sp,
                                        fontWeight = FontWeight.SemiBold,
                                        color = MaterialTheme.colorScheme.onSurface,
                                        maxLines = 1,
                                        overflow = TextOverflow.Ellipsis
                                    )

                                    Text(
                                        text = pred.commandItem.command,
                                        fontSize = 11.sp,
                                        fontFamily = FontFamily.Monospace,
                                        fontWeight = FontWeight.Bold,
                                        color = MaterialTheme.colorScheme.primary,
                                        maxLines = 1,
                                        overflow = TextOverflow.Ellipsis
                                    )
                                }
                            }
                        }
                    }
                }
            }
        }

        // 3. Dynamic Category Filter Chips Row
        item {
            val categories = if (dynamicCategories.isNotEmpty()) dynamicCategories else CommandCategory.values().toList()
            LazyRow(
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                contentPadding = PaddingValues(vertical = 6.dp)
            ) {
                items(categories) { cat ->
                    val isSelected = cat == selectedCategory
                    Surface(
                        color = if (isSelected) {
                            MaterialTheme.colorScheme.secondaryContainer
                        } else {
                            MaterialTheme.colorScheme.surfaceContainerLow
                        },
                        shape = RoundedCornerShape(16.dp),
                        modifier = Modifier
                            .heightIn(min = 48.dp)
                            .border(
                                0.8.dp,
                                if (isSelected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.outlineVariant,
                                RoundedCornerShape(16.dp)
                            )
                            .clip(RoundedCornerShape(16.dp))
                            .selectable(selected = isSelected, onClick = { onCategoryChange(cat) })
                    ) {
                        Box(
                            modifier = Modifier
                                .fillMaxHeight()
                                .padding(horizontal = 16.dp),
                            contentAlignment = Alignment.Center
                        ) {
                            Text(
                                text = cat.label,
                                fontSize = 12.sp,
                                fontWeight = if (isSelected) FontWeight.Bold else FontWeight.Medium,
                                color = if (isSelected) {
                                    MaterialTheme.colorScheme.onSecondaryContainer
                                } else {
                                    MaterialTheme.colorScheme.onSurfaceVariant
                                }
                            )
                        }
                    }
                }
            }
        }

        when {
            allItems.isEmpty() -> item {
                DirectoryMessage(
                    title = "Loading commands",
                    body = "Reading the command catalog from the app bundle.",
                    showSpinner = true
                )
            }

            filteredItems.isEmpty() -> item {
                DirectoryMessage(
                    title = "No commands match",
                    body = if (searchQuery.isBlank()) {
                        "Nothing in ${selectedCategory.label}. Try another category."
                    } else {
                        "Nothing matches \"$searchQuery\". Check the spelling or clear the search."
                    },
                    actionLabel = if (searchQuery.isNotBlank()) "Clear search" else "Show all",
                    onAction = {
                        if (searchQuery.isNotBlank()) {
                            onSearchQueryChange("")
                        } else {
                            onCategoryChange(CommandCategory.ALL)
                        }
                    }
                )
            }

            else -> items(filteredItems, key = { it.id }) { item ->
                val predMatch = predictedCommands.firstOrNull { it.commandItem.id == item.id }

                Surface(
                    color = if (predMatch != null) MaterialTheme.colorScheme.surfaceContainer else MaterialTheme.colorScheme.surfaceContainerLow,
                    shape = RoundedCornerShape(14.dp),
                    modifier = Modifier
                        .fillMaxWidth()
                        .border(
                            if (predMatch != null) 1.dp else 0.8.dp,
                            if (predMatch != null) MaterialTheme.colorScheme.primary.copy(alpha = 0.4f) else MaterialTheme.colorScheme.outlineVariant,
                            RoundedCornerShape(14.dp)
                        )
                        .clip(RoundedCornerShape(14.dp))
                        .clickable { onCommandClick(item) }
                ) {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(14.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(12.dp)
                    ) {
                        Box(
                            modifier = Modifier
                                .size(42.dp)
                                .clip(RoundedCornerShape(10.dp))
                                .background(MaterialTheme.colorScheme.surfaceContainerHigh),
                            contentAlignment = Alignment.Center
                        ) {
                            Icon(
                                painter = painterResource(id = R.drawable.ic_terminal),
                                contentDescription = null,
                                tint = if (predMatch != null) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant,
                                modifier = Modifier.size(20.dp)
                            )
                        }

                        Column(modifier = Modifier.weight(1f)) {
                            Row(
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(6.dp)
                            ) {
                                Text(
                                    text = item.title,
                                    fontSize = 14.sp,
                                    fontWeight = FontWeight.SemiBold,
                                    color = MaterialTheme.colorScheme.onSurface,
                                    modifier = Modifier.weight(1f, fill = false)
                                )
                                if (predMatch != null) {
                                    Surface(
                                        color = MaterialTheme.colorScheme.primary.copy(alpha = 0.14f),
                                        shape = RoundedCornerShape(6.dp)
                                    ) {
                                        Text(
                                            text = predMatch.reason,
                                            fontSize = 9.sp,
                                            fontWeight = FontWeight.Bold,
                                            color = MaterialTheme.colorScheme.primary,
                                            maxLines = 1,
                                            overflow = TextOverflow.Ellipsis,
                                            modifier = Modifier.padding(horizontal = 5.dp, vertical = 2.dp)
                                        )
                                    }
                                }
                            }
                            Text(
                                text = item.description,
                                fontSize = 12.sp,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis
                            )
                            Text(
                                text = item.command,
                                fontSize = 11.sp,
                                fontFamily = FontFamily.Monospace,
                                fontWeight = FontWeight.Bold,
                                color = MaterialTheme.colorScheme.primary,
                                modifier = Modifier.padding(top = 2.dp)
                            )
                        }

                        Icon(
                            painter = painterResource(id = R.drawable.ic_play_arrow),
                            contentDescription = null,
                            tint = MaterialTheme.colorScheme.onSurfaceVariant,
                            modifier = Modifier.size(20.dp)
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun DirectoryMessage(
    title: String,
    body: String,
    showSpinner: Boolean = false,
    actionLabel: String? = null,
    onAction: (() -> Unit)? = null
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(top = 48.dp, bottom = 24.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(10.dp)
    ) {
        if (showSpinner) {
            CircularProgressIndicator(
                modifier = Modifier.size(28.dp),
                strokeWidth = 2.5.dp,
                color = MaterialTheme.colorScheme.primary
            )
        }

        Text(
            text = title,
            style = MaterialTheme.typography.titleMedium,
            color = MaterialTheme.colorScheme.onSurface
        )

        Text(
            text = body,
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
            modifier = Modifier.padding(horizontal = 24.dp)
        )

        if (actionLabel != null && onAction != null) {
            OutlinedButton(
                onClick = onAction,
                shape = RoundedCornerShape(12.dp),
                modifier = Modifier.heightIn(min = 48.dp)
            ) {
                Text(actionLabel)
            }
        }
    }
}

// ============================================================================
// TAB 3: SETTINGS (Styled in Oto Music Palette)
// ============================================================================

@Composable
private fun SettingsTabContent(
    uiState: MainUiState,
    onSaveSettings: (String, Int, ThemeMode) -> Unit,
    onPing: () -> Unit,
    onThemeChange: (ThemeMode) -> Unit
) {
    var hostInput by remember(uiState.pcHost) { mutableStateOf(uiState.pcHost) }
    var portInput by remember(uiState.pcPort) { mutableStateOf(uiState.pcPort.toString()) }
    var showErrors by remember { mutableStateOf(false) }

    val trimmedHost = hostInput.trim()
    val hostError = when {
        trimmedHost.isEmpty() -> "Enter the PC's address"
        trimmedHost.any { it.isWhitespace() } -> "Address can't contain spaces"
        else -> null
    }

    val portValue = portInput.trim().toIntOrNull()
    val portError = when {
        portInput.isBlank() -> "Enter a port"
        portValue == null -> "Port must be a number"
        portValue !in 1..65535 -> "Port must be between 1 and 65535"
        else -> null
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .imePadding()
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        Text(
            text = "Settings",
            style = MaterialTheme.typography.headlineSmall,
            color = MaterialTheme.colorScheme.onSurface
        )

        // --- Connection ---
        Card(
            modifier = Modifier
                .fillMaxWidth()
                .border(1.dp, MaterialTheme.colorScheme.outlineVariant, RoundedCornerShape(18.dp)),
            colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceContainerLow),
            shape = RoundedCornerShape(18.dp)
        ) {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(16.dp),
                verticalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        text = "PC connection",
                        style = MaterialTheme.typography.titleMedium,
                        color = MaterialTheme.colorScheme.onSurface
                    )
                    ConnectionBadge(connectionState = uiState.connectionState)
                }

                OutlinedTextField(
                    value = hostInput,
                    onValueChange = {
                        hostInput = it
                        showErrors = false
                    },
                    label = { Text("PC address") },
                    placeholder = { Text("192.168.0.105") },
                    singleLine = true,
                    isError = showErrors && hostError != null,
                    supportingText = if (showErrors && hostError != null) {
                        { Text(hostError) }
                    } else {
                        null
                    },
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Uri),
                    colors = OutlinedTextFieldDefaults.colors(
                        focusedTextColor = MaterialTheme.colorScheme.onSurface,
                        unfocusedTextColor = MaterialTheme.colorScheme.onSurface,
                        focusedBorderColor = MaterialTheme.colorScheme.primary,
                        unfocusedBorderColor = MaterialTheme.colorScheme.outline
                    ),
                    modifier = Modifier.fillMaxWidth()
                )

                OutlinedTextField(
                    value = portInput,
                    onValueChange = {
                        portInput = it
                        showErrors = false
                    },
                    label = { Text("Port") },
                    placeholder = { Text("9421") },
                    singleLine = true,
                    isError = showErrors && portError != null,
                    supportingText = if (showErrors && portError != null) {
                        { Text(portError) }
                    } else {
                        null
                    },
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
                    colors = OutlinedTextFieldDefaults.colors(
                        focusedTextColor = MaterialTheme.colorScheme.onSurface,
                        unfocusedTextColor = MaterialTheme.colorScheme.onSurface,
                        focusedBorderColor = MaterialTheme.colorScheme.primary,
                        unfocusedBorderColor = MaterialTheme.colorScheme.outline
                    ),
                    modifier = Modifier.fillMaxWidth()
                )

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    Button(
                        onClick = {
                            if (hostError != null || portError != null) {
                                showErrors = true
                            } else {
                                onSaveSettings(trimmedHost, portValue ?: 9421, uiState.themeMode)
                            }
                        },
                        colors = ButtonDefaults.buttonColors(
                            containerColor = MaterialTheme.colorScheme.primary,
                            contentColor = MaterialTheme.colorScheme.onPrimary
                        ),
                        shape = RoundedCornerShape(12.dp),
                        modifier = Modifier
                            .weight(1f)
                            .heightIn(min = 48.dp)
                    ) {
                        Text("Save & connect", fontWeight = FontWeight.Bold)
                    }

                    OutlinedButton(
                        onClick = onPing,
                        shape = RoundedCornerShape(12.dp),
                        modifier = Modifier.heightIn(min = 48.dp)
                    ) {
                        Text("Test")
                    }
                }
            }
        }

        // --- Appearance ---
        Card(
            modifier = Modifier
                .fillMaxWidth()
                .border(1.dp, MaterialTheme.colorScheme.outlineVariant, RoundedCornerShape(18.dp)),
            colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceContainerLow),
            shape = RoundedCornerShape(18.dp)
        ) {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(16.dp),
                verticalArrangement = Arrangement.spacedBy(4.dp)
            ) {
                Text(
                    text = "Appearance",
                    style = MaterialTheme.typography.titleMedium,
                    color = MaterialTheme.colorScheme.onSurface
                )
                Spacer(modifier = Modifier.height(4.dp))

                ThemeMode.values().forEach { mode ->
                    val label = when (mode) {
                        ThemeMode.SYSTEM -> "Follow system"
                        ThemeMode.LIGHT -> "Light"
                        ThemeMode.DARK -> "Dark"
                    }
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .clip(RoundedCornerShape(12.dp))
                            .selectable(
                                selected = uiState.themeMode == mode,
                                role = Role.RadioButton,
                                onClick = { onThemeChange(mode) }
                            )
                            .heightIn(min = 48.dp)
                            .padding(horizontal = 4.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        RadioButton(
                            selected = uiState.themeMode == mode,
                            onClick = null,
                            colors = RadioButtonDefaults.colors(
                                selectedColor = MaterialTheme.colorScheme.primary
                            )
                        )
                        Text(
                            text = label,
                            style = MaterialTheme.typography.bodyLarge,
                            color = MaterialTheme.colorScheme.onSurface
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun ConnectionBadge(connectionState: ConnectionState) {
    val (label, tint) = when (connectionState) {
        is ConnectionState.Online -> "Online" to OtoOnlineGreen
        is ConnectionState.Offline -> "Offline" to MaterialTheme.colorScheme.error
        ConnectionState.Checking -> "Checking" to MaterialTheme.colorScheme.onSurfaceVariant
        ConnectionState.Idle -> "Not connected" to MaterialTheme.colorScheme.onSurfaceVariant
    }

    Surface(
        color = tint.copy(alpha = 0.14f),
        shape = RoundedCornerShape(999.dp)
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 10.dp, vertical = 5.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(6.dp)
        ) {
            Box(
                modifier = Modifier
                    .size(8.dp)
                    .clip(CircleShape)
                    .background(tint)
            )
            Text(
                text = label,
                fontSize = 12.sp,
                fontWeight = FontWeight.SemiBold,
                color = tint
            )
        }
    }
}
