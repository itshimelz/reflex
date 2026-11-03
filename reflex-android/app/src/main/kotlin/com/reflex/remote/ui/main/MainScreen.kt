package com.reflex.remote.ui.main

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.NavigationBarItemDefaults
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
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.reflex.remote.R
import com.reflex.remote.data.ReflexResponse
import com.reflex.remote.data.model.CommandCategory
import com.reflex.remote.data.model.DesktopContext
import com.reflex.remote.data.model.ReflexCommandItem
import com.reflex.remote.ui.theme.*

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

    Scaffold(
        modifier = modifier
            .fillMaxSize()
            .statusBarsPadding()
            .navigationBarsPadding(),
        containerColor = MaterialTheme.colorScheme.background,
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
                onMenuClick = { viewModel.openSettingsDialog() }
            )
        },
        bottomBar = {
            Column(modifier = Modifier.fillMaxWidth()) {
                // Docked Mini Player (Exact Oto Music Bottom Sheet Peek)
                OtoMiniPlayer(
                    context = uiState.desktopContext,
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
    ) { innerPadding ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
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
                            allItems = uiState.allCommands,
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
                            onPing = { viewModel.pingHost() }
                        )
                    }
                }
            }

            // Full Media Control Bottom Sheet (ModalBottomSheet component)
            if (uiState.isMediaSheetExpanded) {
                OtoMediaBottomSheet(
                    context = uiState.desktopContext,
                    connectionState = uiState.connectionState,
                    onDismiss = { viewModel.setMediaSheetExpanded(false) },
                    onPlayPause = { viewModel.togglePlayPause() },
                    onNext = { viewModel.skipNext() },
                    onPrev = { viewModel.skipPrevious() },
                    onToggleMute = { viewModel.toggleMute() },
                    onVolumeChange = { pct -> viewModel.setVolume(pct) },
                    onCommandDispatch = { cmd -> viewModel.dispatchCommand(cmd) }
                )
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
            .height(52.dp),
        shape = RoundedCornerShape(26.dp),
        color = OtoSearchBarBg,
        border = androidx.compose.foundation.BorderStroke(1.dp, OtoDarkOutline)
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
                modifier = Modifier.size(36.dp)
            ) {
                Icon(
                    painter = painterResource(id = R.drawable.ic_menu),
                    contentDescription = "Menu",
                    tint = OtoSearchIcon,
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
                        color = OtoSearchPlaceholder
                    )
                }
                BasicTextField(
                    value = query,
                    onValueChange = onQueryChange,
                    singleLine = true,
                    textStyle = TextStyle(
                        color = OtoTextPrimary,
                        fontSize = 15.sp,
                        fontWeight = FontWeight.Normal
                    ),
                    cursorBrush = SolidColor(OtoPlayerPlayBtnBg),
                    modifier = Modifier.fillMaxWidth()
                )
            }

            // Right: Cast / Connection Status Pill
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(6.dp),
                modifier = Modifier
                    .clip(RoundedCornerShape(16.dp))
                    .clickable(onClick = onPingClick)
                    .padding(horizontal = 6.dp, vertical = 4.dp)
            ) {
                val isOnline = connectionState is ConnectionState.Online
                val latencyText = if (isOnline) "${(connectionState as ConnectionState.Online).latencyMs}ms" else ""

                if (latencyText.isNotBlank()) {
                    Text(
                        text = latencyText,
                        fontSize = 11.sp,
                        fontFamily = FontFamily.Monospace,
                        fontWeight = FontWeight.Bold,
                        color = OtoOnlineGreen
                    )
                }

                Icon(
                    painter = painterResource(id = R.drawable.ic_cast),
                    contentDescription = "Connection Status",
                    tint = if (isOnline) OtoOnlineGreen else OtoSearchIcon,
                    modifier = Modifier.size(20.dp)
                )
            }
        }
    }
}

// ============================================================================
// COMPONENT 2: Top 4 Action Buttons (Exact Oto Music Circular Action Hub)
// History (Blue), Favorites (Magenta), Most Played (Amber), Shuffle (Emerald)
// ============================================================================

@Composable
private fun OtoTopActionButtons(
    onHistoryClick: () -> Unit,
    onFavoritesClick: () -> Unit,
    onMostPlayedClick: () -> Unit,
    onShuffleClick: () -> Unit
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 10.dp),
        horizontalArrangement = Arrangement.SpaceEvenly,
        verticalAlignment = Alignment.CenterVertically
    ) {
        // Button 1: History
        OtoCircularButton(
            title = "History",
            backgroundColor = OtoHistoryBg,
            iconTint = OtoHistoryIcon,
            iconRes = R.drawable.ic_history,
            onClick = onHistoryClick
        )

        // Button 2: Favorites / Quick
        OtoCircularButton(
            title = "Favorites",
            backgroundColor = OtoFavoritesBg,
            iconTint = OtoFavoritesIcon,
            iconRes = R.drawable.ic_favorite,
            onClick = onFavoritesClick
        )

        // Button 3: Most played / Media Control
        OtoCircularButton(
            title = "Most played",
            backgroundColor = OtoMostPlayedBg,
            iconTint = OtoMostPlayedIcon,
            iconRes = R.drawable.ic_trending_up,
            onClick = onMostPlayedClick
        )

        // Button 4: Shuffle / Commands
        OtoCircularButton(
            title = "Shuffle",
            backgroundColor = OtoShuffleBg,
            iconTint = OtoShuffleIcon,
            iconRes = R.drawable.ic_shuffle,
            onClick = onShuffleClick
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
        modifier = Modifier.clickable(onClick = onClick)
    ) {
        Box(
            modifier = Modifier
                .size(56.dp)
                .clip(CircleShape)
                .background(backgroundColor),
            contentAlignment = Alignment.Center
        ) {
            Icon(
                painter = painterResource(id = iconRes),
                contentDescription = title,
                tint = iconTint,
                modifier = Modifier.size(26.dp)
            )
        }
        Text(
            text = title,
            fontSize = 12.sp,
            fontWeight = FontWeight.Medium,
            color = OtoTextSecondary
        )
    }
}

// ============================================================================
// COMPONENT 3: Mini Player Bar (Exact Oto Music Bottom Sheet Peek)
// Sits docked directly above the Bottom Navigation Bar
// ============================================================================

@Composable
private fun OtoMiniPlayer(
    context: DesktopContext?,
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
            "Desktop System Ready"
        }

    val artistName = context?.media?.artist?.ifBlank { null }
        ?: if (context?.activeWindow?.title?.isNotBlank() == true) {
            context.activeWindow.title
        } else {
            "Reflex LAN Daemon"
        }

    val volumeFraction = (context?.volume?.percent ?: 50) / 100f

    Surface(
        color = OtoDarkSurface,
        modifier = Modifier
            .fillMaxWidth()
            .height(66.dp)
            .border(
                width = 0.8.dp,
                color = OtoDarkOutlineVariant,
                shape = RoundedCornerShape(topStart = 16.dp, topEnd = 16.dp)
            )
            .clip(RoundedCornerShape(topStart = 16.dp, topEnd = 16.dp))
            .clickable(onClick = onExpandClick)
    ) {
        Column(modifier = Modifier.fillMaxSize()) {
            // Thin progress/volume indicator line at the top
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(2.5.dp)
                    .background(OtoPlayerSliderInactive)
            ) {
                Box(
                    modifier = Modifier
                        .fillMaxWidth(fraction = volumeFraction.coerceIn(0.05f, 1f))
                        .fillMaxHeight()
                        .background(OtoPlayerSliderActive)
                )
            }

            Row(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(horizontal = 14.dp, vertical = 6.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                // Media Thumbnail with Album Art Styling
                Box(
                    modifier = Modifier
                        .size(46.dp)
                        .clip(RoundedCornerShape(10.dp))
                        .background(OtoDarkCard),
                    contentAlignment = Alignment.Center
                ) {
                    Icon(
                        painter = painterResource(id = if (isPlaying) R.drawable.ic_music_note else R.drawable.ic_reflex_logo),
                        contentDescription = "Track Art",
                        tint = if (isPlaying) OtoPlayerActiveControl else Color.Unspecified,
                        modifier = Modifier.size(24.dp)
                    )
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
                        color = OtoTextPrimary,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                    Spacer(modifier = Modifier.height(2.dp))
                    Text(
                        text = artistName,
                        fontSize = 12.sp,
                        fontWeight = FontWeight.Normal,
                        color = OtoTextSecondary,
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
                        modifier = Modifier.size(36.dp)
                    ) {
                        Icon(
                            painter = painterResource(id = R.drawable.ic_skip_previous),
                            contentDescription = "Previous Track",
                            tint = OtoPlayerActiveControl,
                            modifier = Modifier.size(20.dp)
                        )
                    }

                    IconButton(
                        onClick = onPlayPauseClick,
                        modifier = Modifier.size(40.dp)
                    ) {
                        if (isDispatching) {
                            CircularProgressIndicator(
                                modifier = Modifier.size(22.dp),
                                color = OtoPlayerActiveControl,
                                strokeWidth = 2.5.dp
                            )
                        } else {
                            Icon(
                                painter = painterResource(id = if (isPlaying) R.drawable.ic_pause else R.drawable.ic_play_arrow),
                                contentDescription = if (isPlaying) "Pause" else "Play",
                                tint = OtoPlayerActiveControl,
                                modifier = Modifier.size(28.dp)
                            )
                        }
                    }

                    IconButton(
                        onClick = onNextClick,
                        modifier = Modifier.size(36.dp)
                    ) {
                        Icon(
                            painter = painterResource(id = R.drawable.ic_skip_next),
                            contentDescription = "Next Track",
                            tint = OtoPlayerActiveControl,
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
    connectionState: ConnectionState,
    onDismiss: () -> Unit,
    onPlayPause: () -> Unit,
    onNext: () -> Unit,
    onPrev: () -> Unit,
    onToggleMute: () -> Unit,
    onVolumeChange: (Int) -> Unit,
    onCommandDispatch: (String) -> Unit
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

    var currentVolumeSlider by remember(context?.volume?.percent) {
        mutableFloatStateOf((context?.volume?.percent ?: 50).toFloat())
    }

    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = sheetState,
        containerColor = OtoDarkBackground,
        dragHandle = {
            Box(
                modifier = Modifier
                    .padding(top = 12.dp, bottom = 8.dp)
                    .size(width = 38.dp, height = 4.dp)
                    .clip(CircleShape)
                    .background(OtoActiveNavPill)
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
                        contentDescription = "Collapse Player",
                        tint = OtoSearchIcon,
                        modifier = Modifier.size(20.dp)
                    )
                }

                Text(
                    text = "NOW PLAYING",
                    fontSize = 11.sp,
                    fontFamily = FontFamily.Monospace,
                    fontWeight = FontWeight.Bold,
                    color = OtoTextSecondary,
                    letterSpacing = 1.6.sp
                )

                IconButton(onClick = { onCommandDispatch("volume 100") }) {
                    Icon(
                        painter = painterResource(id = R.drawable.ic_more_horiz),
                        contentDescription = "Options",
                        tint = OtoSearchIcon,
                        modifier = Modifier.size(22.dp)
                    )
                }
            }

            Spacer(modifier = Modifier.height(14.dp))

            // Large Album Artwork Display (Exact 260dp x 260dp squircle card with disc visualizer)
            Card(
                modifier = Modifier
                    .size(260.dp)
                    .clip(RoundedCornerShape(26.dp)),
                colors = CardDefaults.cardColors(containerColor = OtoPlayerArtworkBg),
                shape = RoundedCornerShape(26.dp),
                elevation = CardDefaults.cardElevation(defaultElevation = 8.dp)
            ) {
                Box(
                    modifier = Modifier.fillMaxSize(),
                    contentAlignment = Alignment.Center
                ) {
                    // Concentric soundwave circles canvas
                    Canvas(modifier = Modifier.fillMaxSize()) {
                        val center = Offset(size.width / 2f, size.height / 2f)
                        val maxRadius = size.minDimension / 2f
                        drawCircle(
                            color = OtoDarkSurfaceVariant.copy(alpha = 0.5f),
                            radius = maxRadius * 0.9f,
                            center = center,
                            style = Stroke(width = 2f)
                        )
                        drawCircle(
                            color = OtoDarkSurfaceVariant.copy(alpha = 0.4f),
                            radius = maxRadius * 0.7f,
                            center = center,
                            style = Stroke(width = 2f)
                        )
                        drawCircle(
                            color = OtoDarkSurfaceVariant.copy(alpha = 0.3f),
                            radius = maxRadius * 0.45f,
                            center = center,
                            style = Stroke(width = 2f)
                        )
                    }

                    Icon(
                        painter = painterResource(id = R.drawable.ic_music_note),
                        contentDescription = "Audio Art",
                        tint = if (isPlaying) OtoPlayerPlayBtnBg else OtoActiveNavPill,
                        modifier = Modifier.size(80.dp)
                    )
                }
            }

            Spacer(modifier = Modifier.height(24.dp))

            // Track Title & Artist Info
            Text(
                text = songTitle,
                fontSize = 22.sp,
                fontWeight = FontWeight.Bold,
                color = OtoTextPrimary,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )

            Spacer(modifier = Modifier.height(4.dp))

            Text(
                text = "$artistName • $playerEngine",
                fontSize = 14.sp,
                fontWeight = FontWeight.Normal,
                color = OtoTextSecondary,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )

            Spacer(modifier = Modifier.height(18.dp))

            // Time / Progress Slider (Simulated or audio duration)
            Column(modifier = Modifier.fillMaxWidth()) {
                Slider(
                    value = 0.42f,
                    onValueChange = {},
                    enabled = false,
                    colors = SliderDefaults.colors(
                        thumbColor = OtoPlayerSliderActive,
                        activeTrackColor = OtoPlayerSliderActive,
                        inactiveTrackColor = OtoPlayerSliderInactive
                    ),
                    modifier = Modifier.fillMaxWidth()
                )
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 4.dp),
                    horizontalArrangement = Arrangement.SpaceBetween
                ) {
                    Text(text = "01:46", fontSize = 12.sp, color = OtoTextSecondary)
                    Text(text = "03:44", fontSize = 12.sp, color = OtoTextSecondary)
                }
            }

            Spacer(modifier = Modifier.height(10.dp))

            // PC Volume Control Slider
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 4.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                IconButton(
                    onClick = onToggleMute,
                    modifier = Modifier.size(32.dp)
                ) {
                    Icon(
                        painter = painterResource(
                            id = if (context?.volume?.isMuted == true) R.drawable.ic_volume_off else R.drawable.ic_volume
                        ),
                        contentDescription = "Mute",
                        tint = if (context?.volume?.isMuted == true) MaterialTheme.colorScheme.error else OtoTextSecondary,
                        modifier = Modifier.size(18.dp)
                    )
                }

                Slider(
                    value = currentVolumeSlider,
                    onValueChange = { currentVolumeSlider = it },
                    onValueChangeFinished = { onVolumeChange(currentVolumeSlider.toInt()) },
                    valueRange = 0f..100f,
                    colors = SliderDefaults.colors(
                        thumbColor = OtoPlayerSliderActive,
                        activeTrackColor = OtoPlayerSliderActive,
                        inactiveTrackColor = OtoPlayerSliderInactive
                    ),
                    modifier = Modifier.weight(1f)
                )

                IconButton(
                    onClick = { onVolumeChange(100) },
                    modifier = Modifier.size(32.dp)
                ) {
                    Icon(
                        painter = painterResource(id = R.drawable.ic_volume_up),
                        contentDescription = "Max Volume",
                        tint = OtoTextSecondary,
                        modifier = Modifier.size(18.dp)
                    )
                }
            }

            Spacer(modifier = Modifier.height(14.dp))

            // Media Control Row (Shuffle, Prev, Big Play, Next, Repeat)
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceEvenly,
                verticalAlignment = Alignment.CenterVertically
            ) {
                IconButton(
                    onClick = { onCommandDispatch("reflex \"volume -5\"") },
                    modifier = Modifier.size(44.dp)
                ) {
                    Icon(
                        painter = painterResource(id = R.drawable.ic_shuffle),
                        contentDescription = "Shuffle",
                        tint = OtoPlayerInactiveControl,
                        modifier = Modifier.size(24.dp)
                    )
                }

                IconButton(
                    onClick = onPrev,
                    modifier = Modifier.size(48.dp)
                ) {
                    Icon(
                        painter = painterResource(id = R.drawable.ic_skip_previous),
                        contentDescription = "Previous",
                        tint = OtoPlayerActiveControl,
                        modifier = Modifier.size(34.dp)
                    )
                }

                // Big Circular Play/Pause Button (72dp x 72dp in #D6E3FF)
                Box(
                    modifier = Modifier
                        .size(72.dp)
                        .clip(CircleShape)
                        .background(OtoPlayerPlayBtnBg)
                        .clickable(onClick = onPlayPause),
                    contentAlignment = Alignment.Center
                ) {
                    Icon(
                        painter = painterResource(
                            id = if (isPlaying) R.drawable.ic_pause else R.drawable.ic_play_arrow
                        ),
                        contentDescription = if (isPlaying) "Pause" else "Play",
                        tint = OtoPlayerPlayBtnIcon,
                        modifier = Modifier.size(38.dp)
                    )
                }

                IconButton(
                    onClick = onNext,
                    modifier = Modifier.size(48.dp)
                ) {
                    Icon(
                        painter = painterResource(id = R.drawable.ic_skip_next),
                        contentDescription = "Next",
                        tint = OtoPlayerActiveControl,
                        modifier = Modifier.size(34.dp)
                    )
                }

                IconButton(
                    onClick = onToggleMute,
                    modifier = Modifier.size(44.dp)
                ) {
                    Icon(
                        painter = painterResource(id = R.drawable.ic_refresh),
                        contentDescription = "Repeat",
                        tint = OtoPlayerInactiveControl,
                        modifier = Modifier.size(24.dp)
                    )
                }
            }

            Spacer(modifier = Modifier.height(18.dp))

            // Exact Specs Row ("PIPEWIRE • 6 ms • 48.0 kHz • STEREO")
            val latency = if (connectionState is ConnectionState.Online) connectionState.latencyMs else 5
            Text(
                text = "PIPEWIRE • ${latency} ms • 48.0 kHz • STEREO",
                fontSize = 11.sp,
                fontFamily = FontFamily.Monospace,
                fontWeight = FontWeight.Medium,
                color = OtoTextTertiary,
                letterSpacing = 1.2.sp
            )

            Spacer(modifier = Modifier.height(20.dp))

            // Bottom 5 Action Bar Icons (Lyrics/Transcript, Favorite, Sleep, Queue, More)
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 8.dp),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                IconButton(onClick = { onCommandDispatch("reflex \"unmute\"") }) {
                    Icon(
                        painter = painterResource(id = R.drawable.ic_chat),
                        contentDescription = "Lyrics / Transcript",
                        tint = OtoPlayerInactiveControl,
                        modifier = Modifier.size(22.dp)
                    )
                }

                IconButton(onClick = { onCommandDispatch("reflex \"volume 50\"") }) {
                    Icon(
                        painter = painterResource(id = R.drawable.ic_favorite),
                        contentDescription = "Favorite",
                        tint = OtoPlayerInactiveControl,
                        modifier = Modifier.size(22.dp)
                    )
                }

                IconButton(onClick = { onCommandDispatch("reflex \"lock screen\"") }) {
                    Icon(
                        painter = painterResource(id = R.drawable.ic_bedtime),
                        contentDescription = "Sleep Timer / Lock",
                        tint = OtoPlayerInactiveControl,
                        modifier = Modifier.size(22.dp)
                    )
                }

                IconButton(onClick = { onCommandDispatch("reflex \"workspace 1\"") }) {
                    Icon(
                        painter = painterResource(id = R.drawable.ic_queue_music),
                        contentDescription = "Queue / Workspace",
                        tint = OtoPlayerInactiveControl,
                        modifier = Modifier.size(22.dp)
                    )
                }

                IconButton(onClick = { onCommandDispatch("reflex \"screenshot window\"") }) {
                    Icon(
                        painter = painterResource(id = R.drawable.ic_more_horiz),
                        contentDescription = "More",
                        tint = OtoPlayerInactiveControl,
                        modifier = Modifier.size(22.dp)
                    )
                }
            }
        }
    }
}

// ============================================================================
// COMPONENT 5: Bottom Navigation Bar (Exact Oto Music Style)
// 3 Tabs: Home, Commands, Settings with #3F4565 active pill indicator
// ============================================================================

@Composable
private fun OtoBottomNavigationBar(
    currentTab: ScreenTab,
    onTabSelect: (ScreenTab) -> Unit
) {
    NavigationBar(
        containerColor = OtoDarkSurface,
        tonalElevation = 0.dp,
        modifier = Modifier
            .fillMaxWidth()
            .height(72.dp)
            .border(
                width = 0.8.dp,
                color = OtoDarkOutlineVariant,
                shape = RoundedCornerShape(0.dp)
            )
    ) {
        NavigationBarItem(
            selected = currentTab == ScreenTab.REMOTE,
            onClick = { onTabSelect(ScreenTab.REMOTE) },
            icon = {
                Icon(
                    painter = painterResource(id = R.drawable.ic_home),
                    contentDescription = "Home",
                    modifier = Modifier.size(22.dp)
                )
            },
            label = {
                Text(
                    text = "Home",
                    fontSize = 11.sp,
                    fontWeight = if (currentTab == ScreenTab.REMOTE) FontWeight.Bold else FontWeight.Medium
                )
            },
            colors = NavigationBarItemDefaults.colors(
                indicatorColor = OtoActiveNavPill,
                selectedIconColor = OtoActiveNavIcon,
                selectedTextColor = OtoTextPrimary,
                unselectedIconColor = OtoInactiveNavIcon,
                unselectedTextColor = OtoInactiveNavIcon
            )
        )

        NavigationBarItem(
            selected = currentTab == ScreenTab.DIRECTORY,
            onClick = { onTabSelect(ScreenTab.DIRECTORY) },
            icon = {
                Icon(
                    painter = painterResource(id = R.drawable.ic_music_note),
                    contentDescription = "Commands",
                    modifier = Modifier.size(22.dp)
                )
            },
            label = {
                Text(
                    text = "Commands",
                    fontSize = 11.sp,
                    fontWeight = if (currentTab == ScreenTab.DIRECTORY) FontWeight.Bold else FontWeight.Medium
                )
            },
            colors = NavigationBarItemDefaults.colors(
                indicatorColor = OtoActiveNavPill,
                selectedIconColor = OtoActiveNavIcon,
                selectedTextColor = OtoTextPrimary,
                unselectedIconColor = OtoInactiveNavIcon,
                unselectedTextColor = OtoInactiveNavIcon
            )
        )

        NavigationBarItem(
            selected = currentTab == ScreenTab.SETTINGS,
            onClick = { onTabSelect(ScreenTab.SETTINGS) },
            icon = {
                Icon(
                    painter = painterResource(id = R.drawable.ic_settings),
                    contentDescription = "Settings",
                    modifier = Modifier.size(22.dp)
                )
            },
            label = {
                Text(
                    text = "Settings",
                    fontSize = 11.sp,
                    fontWeight = if (currentTab == ScreenTab.SETTINGS) FontWeight.Bold else FontWeight.Medium
                )
            },
            colors = NavigationBarItemDefaults.colors(
                indicatorColor = OtoActiveNavPill,
                selectedIconColor = OtoActiveNavIcon,
                selectedTextColor = OtoTextPrimary,
                unselectedIconColor = OtoInactiveNavIcon,
                unselectedTextColor = OtoInactiveNavIcon
            )
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
                onHistoryClick = { /* scroll or highlight history */ },
                onFavoritesClick = {
                    uiState.suggestedCommands.firstOrNull()?.let { onCommandDispatch(it.command) }
                },
                onMostPlayedClick = onOpenMediaSheet,
                onShuffleClick = onOpenDirectory
            )
        }

        // Two Hero Carousel Cards: "For You Now" and "Audio Engine"
        item {
            OtoCarouselHeroRow(
                context = uiState.desktopContext,
                onForYouClick = {
                    uiState.suggestedCommands.firstOrNull()?.let { onCommandDispatch(it.command) }
                },
                onAudioClick = onOpenMediaSheet
            )
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
                    text = "DISPATCH HISTORY",
                    fontSize = 11.sp,
                    fontFamily = FontFamily.Monospace,
                    fontWeight = FontWeight.Bold,
                    color = OtoTextSecondary,
                    letterSpacing = 1.2.sp
                )

                if (uiState.history.isNotEmpty()) {
                    Text(
                        text = "CLEAR",
                        fontSize = 11.sp,
                        fontFamily = FontFamily.Monospace,
                        fontWeight = FontWeight.Bold,
                        color = OtoPlayerPlayBtnBg,
                        modifier = Modifier
                            .clickable(onClick = onClearHistory)
                            .padding(4.dp)
                    )
                }
            }
        }

        if (uiState.history.isEmpty()) {
            item {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(vertical = 12.dp),
                    contentAlignment = Alignment.Center
                ) {
                    Text(
                        text = "No commands dispatched yet",
                        fontFamily = FontFamily.Monospace,
                        fontSize = 12.sp,
                        color = OtoTextTertiary
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
// CAROUSEL HERO CARDS (Exact Oto Music "For You Now" & "Late Night Mix")
// ============================================================================

@Composable
private fun OtoCarouselHeroRow(
    context: DesktopContext?,
    onForYouClick: () -> Unit,
    onAudioClick: () -> Unit
) {
    val scrollState = rememberScrollState()

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .horizontalScroll(scrollState),
        horizontalArrangement = Arrangement.spacedBy(14.dp)
    ) {
        // Card 1: "For You Now" - Focus on Active Window
        Card(
            modifier = Modifier
                .width(230.dp)
                .height(230.dp)
                .clip(RoundedCornerShape(26.dp))
                .clickable(onClick = onForYouClick),
            colors = CardDefaults.cardColors(containerColor = Color(0xFF1E2838)),
            shape = RoundedCornerShape(26.dp)
        ) {
            Box(modifier = Modifier.fillMaxSize()) {
                // Background artistic concentric radar circles
                Canvas(modifier = Modifier.fillMaxSize()) {
                    val center = Offset(size.width * 0.75f, size.height * 0.35f)
                    drawCircle(color = Color(0xFF2A364C).copy(alpha = 0.5f), radius = 100f, center = center, style = Stroke(2f))
                    drawCircle(color = Color(0xFF2A364C).copy(alpha = 0.4f), radius = 160f, center = center, style = Stroke(2f))
                    drawCircle(color = Color(0xFF2A364C).copy(alpha = 0.3f), radius = 220f, center = center, style = Stroke(2f))
                    drawCircle(color = Color(0xFF7094C4), radius = 42f, center = center)
                    drawCircle(color = Color(0xFFB5D0F5), radius = 18f, center = center)
                }

                Column(
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(20.dp),
                    verticalArrangement = Arrangement.Bottom
                ) {
                    Text(
                        text = "• TUNED TO RIGHT NOW",
                        fontSize = 11.sp,
                        fontWeight = FontWeight.Bold,
                        fontFamily = FontFamily.Monospace,
                        color = Color(0xFF9EA4B6)
                    )
                    Spacer(modifier = Modifier.height(6.dp))
                    Text(
                        text = context?.activeWindow?.appName?.ifBlank { "For You" } ?: "For You\nNow",
                        fontSize = 24.sp,
                        fontWeight = FontWeight.Bold,
                        color = Color.White,
                        lineHeight = 28.sp
                    )
                }
            }
        }

        // Card 2: "Late Night Mix" - Audio & Volume Control
        Card(
            modifier = Modifier
                .width(230.dp)
                .height(230.dp)
                .clip(RoundedCornerShape(26.dp))
                .clickable(onClick = onAudioClick),
            colors = CardDefaults.cardColors(containerColor = Color(0xFF382622)),
            shape = RoundedCornerShape(26.dp)
        ) {
            Box(modifier = Modifier.fillMaxSize()) {
                // Background artistic clock/dial
                Canvas(modifier = Modifier.fillMaxSize()) {
                    val center = Offset(size.width * 0.8f, size.height * 0.35f)
                    drawCircle(color = Color(0xFFF7D5C8), radius = 55f, center = center)
                    // Clock hands
                    drawLine(color = Color(0xFF382622), start = center, end = Offset(center.x, center.y - 30f), strokeWidth = 3f)
                    drawLine(color = Color(0xFF382622), start = center, end = Offset(center.x + 22f, center.y + 12f), strokeWidth = 3f)
                }

                Column(
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(20.dp),
                    verticalArrangement = Arrangement.Bottom
                ) {
                    Text(
                        text = "• FOR THIS HOUR",
                        fontSize = 11.sp,
                        fontWeight = FontWeight.Bold,
                        fontFamily = FontFamily.Monospace,
                        color = Color(0xFFE2B4A6)
                    )
                    Spacer(modifier = Modifier.height(6.dp))
                    Text(
                        text = "${context?.volume?.percent ?: 50}% Vol\nAudio Mix",
                        fontSize = 24.sp,
                        fontWeight = FontWeight.Bold,
                        color = Color.White,
                        lineHeight = 28.sp
                    )
                }
            }
        }
    }
}

// ============================================================================
// "HIDDEN GEMS" SECTION (Exact Oto Music song tiles list)
// Displays context-adaptive quick shortcuts as styled horizontal cards
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
        verticalArrangement = Arrangement.spacedBy(10.dp)
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                text = "Hidden gems",
                fontSize = 18.sp,
                fontWeight = FontWeight.Bold,
                color = OtoTextPrimary
            )

            Text(
                text = "->",
                fontSize = 18.sp,
                fontWeight = FontWeight.Bold,
                color = OtoSearchIcon
            )
        }

        // Horizontal Grid/List of commands
        LazyRow(
            horizontalArrangement = Arrangement.spacedBy(10.dp),
            contentPadding = PaddingValues(vertical = 4.dp)
        ) {
            items(commands.take(6), key = { it.id }) { item ->
                Surface(
                    color = OtoDarkCard,
                    shape = RoundedCornerShape(16.dp),
                    modifier = Modifier
                        .width(220.dp)
                        .height(72.dp)
                        .clip(RoundedCornerShape(16.dp))
                        .clickable { onCommandClick(item.command) }
                ) {
                    Row(
                        modifier = Modifier
                            .fillMaxSize()
                            .padding(10.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(10.dp)
                    ) {
                        Box(
                            modifier = Modifier
                                .size(48.dp)
                                .clip(RoundedCornerShape(10.dp))
                                .background(OtoDarkSurfaceVariant),
                            contentAlignment = Alignment.Center
                        ) {
                            Icon(
                                painter = painterResource(id = R.drawable.ic_terminal),
                                contentDescription = item.title,
                                tint = OtoPlayerActiveControl,
                                modifier = Modifier.size(22.dp)
                            )
                        }

                        Column(modifier = Modifier.weight(1f)) {
                            Text(
                                text = item.title,
                                fontSize = 13.sp,
                                fontWeight = FontWeight.SemiBold,
                                color = OtoTextPrimary,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis
                            )
                            Spacer(modifier = Modifier.height(2.dp))
                            Text(
                                text = item.command,
                                fontSize = 11.sp,
                                fontFamily = FontFamily.Monospace,
                                color = OtoTextSecondary,
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
    val infiniteTransition = rememberInfiniteTransition(label = "pulse")
    val pulseScale by infiniteTransition.animateFloat(
        initialValue = 1.0f,
        targetValue = 1.25f,
        animationSpec = infiniteRepeatable(
            animation = tween(850, easing = FastOutSlowInEasing),
            repeatMode = RepeatMode.Reverse
        ),
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
        modifier = Modifier.padding(vertical = 8.dp)
    ) {
        Box(
            contentAlignment = Alignment.Center,
            modifier = Modifier.size(150.dp)
        ) {
            if (isListening) {
                Box(
                    modifier = Modifier
                        .size(140.dp)
                        .scale(pulseScale)
                        .clip(CircleShape)
                        .background(OtoFavoritesBg.copy(alpha = 0.5f))
                )
                Box(
                    modifier = Modifier
                        .size(118.dp)
                        .scale(dynamicAudioScale)
                        .clip(CircleShape)
                        .background(OtoFavoritesBg.copy(alpha = 0.8f))
                )
            } else if (isDispatching) {
                Box(
                    modifier = Modifier
                        .size(125.dp)
                        .scale(pulseScale)
                        .clip(CircleShape)
                        .background(OtoHistoryBg.copy(alpha = 0.6f))
                )
            }

            // Big Push-To-Talk Button
            Surface(
                modifier = Modifier
                    .size(88.dp)
                    .clip(CircleShape)
                    .clickable(onClick = onMicClick),
                shape = CircleShape,
                shadowElevation = 8.dp,
                color = Color.Transparent
            ) {
                val buttonGradient = when {
                    isListening -> Brush.linearGradient(listOf(OtoFavoritesBg, Color(0xFF881337)))
                    isDispatching -> Brush.linearGradient(listOf(OtoHistoryBg, Color(0xFF0F2B48)))
                    else -> Brush.linearGradient(listOf(Color(0xFF1B2D46), Color(0xFF0F1B2B)))
                }

                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .background(buttonGradient)
                        .border(1.dp, OtoDarkOutline, CircleShape),
                    contentAlignment = Alignment.Center
                ) {
                    if (isDispatching) {
                        CircularProgressIndicator(
                            modifier = Modifier.size(38.dp),
                            color = OtoPlayerPlayBtnBg,
                            strokeWidth = 3.dp
                        )
                    } else {
                        Icon(
                            painter = painterResource(id = R.drawable.ic_mic),
                            contentDescription = "Voice Input",
                            tint = if (isListening) OtoFavoritesIcon else OtoPlayerPlayBtnBg,
                            modifier = Modifier.size(38.dp)
                        )
                    }
                }
            }
        }

        Spacer(modifier = Modifier.height(4.dp))

        val statusText = when {
            isDispatching -> "DISPATCHING TO PC..."
            isListening -> "LISTENING TO VOICE..."
            else -> "TAP OR HOLD TO SPEAK"
        }

        Text(
            text = statusText,
            fontFamily = FontFamily.Monospace,
            fontSize = 11.sp,
            fontWeight = FontWeight.Bold,
            letterSpacing = 1.4.sp,
            color = if (isListening) OtoFavoritesIcon else OtoTextSecondary
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
            .border(1.dp, OtoDarkOutline, RoundedCornerShape(20.dp)),
        colors = CardDefaults.cardColors(containerColor = OtoDarkSurface),
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
                    text = "TRANSCRIPT & TELEMETRY",
                    fontSize = 10.sp,
                    fontFamily = FontFamily.Monospace,
                    fontWeight = FontWeight.Bold,
                    color = OtoTextSecondary,
                    letterSpacing = 1.2.sp
                )

                if (lastResponse != null) {
                    val badgeColor = if (lastResponse.isSuccess) OtoOnlineGreen else MaterialTheme.colorScheme.error
                    Surface(
                        color = badgeColor.copy(alpha = 0.15f),
                        shape = RoundedCornerShape(999.dp)
                    ) {
                        Text(
                            text = lastResponse.status.uppercase(),
                            fontFamily = FontFamily.Monospace,
                            fontSize = 10.sp,
                            fontWeight = FontWeight.Bold,
                            color = badgeColor,
                            modifier = Modifier.padding(horizontal = 8.dp, vertical = 3.dp)
                        )
                    }
                }
            }

            Spacer(modifier = Modifier.height(8.dp))

            val displayTranscript = when {
                transcript.isNotBlank() -> transcript
                isListening -> "..."
                else -> "Say: \"sound 50\", \"next\", \"unmute\", \"pause\""
            }

            Text(
                text = displayTranscript,
                fontSize = 14.sp,
                fontWeight = FontWeight.Medium,
                color = if (transcript.isNotBlank()) OtoTextPrimary else OtoTextSecondary,
                lineHeight = 20.sp
            )

            if (lastResponse != null) {
                Spacer(modifier = Modifier.height(10.dp))
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(10.dp))
                        .background(OtoDarkCard)
                        .padding(horizontal = 10.dp, vertical = 6.dp),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        text = lastResponse.action ?: lastResponse.reason ?: "Executed",
                        fontSize = 11.sp,
                        fontFamily = FontFamily.Monospace,
                        color = OtoPlayerPlayBtnBg,
                        fontWeight = FontWeight.Bold
                    )
                    Text(
                        text = "${lastResponse.durationUs}us PC / ${lastResponse.roundTripMs}ms Net",
                        fontSize = 10.sp,
                        fontFamily = FontFamily.Monospace,
                        color = OtoTextSecondary
                    )
                }
            }
        }
    }
}

@Composable
private fun HistoryItemCard(item: CommandLogItem) {
    Surface(
        color = OtoDarkCard,
        shape = RoundedCornerShape(14.dp),
        modifier = Modifier
            .fillMaxWidth()
            .border(0.8.dp, OtoDarkOutlineVariant, RoundedCornerShape(14.dp))
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
                    .background(if (item.isError) MaterialTheme.colorScheme.error else OtoOnlineGreen)
            )

            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = item.command,
                    fontSize = 13.sp,
                    fontFamily = FontFamily.Monospace,
                    fontWeight = FontWeight.Bold,
                    color = OtoTextPrimary
                )
                Text(
                    text = item.response?.action ?: item.errorMessage ?: "Success",
                    fontSize = 11.sp,
                    color = OtoTextSecondary,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
            }

            Text(
                text = item.timestamp,
                fontSize = 11.sp,
                fontFamily = FontFamily.Monospace,
                color = OtoTextTertiary
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
        color = Color(0xFF3F1318),
        shape = RoundedCornerShape(14.dp),
        modifier = Modifier
            .fillMaxWidth()
            .border(1.dp, Color(0xFF7F1D1D), RoundedCornerShape(14.dp))
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
                tint = Color(0xFFFCA5A5),
                modifier = Modifier.size(18.dp)
            )
            Text(
                text = text,
                fontSize = 12.sp,
                fontFamily = FontFamily.Monospace,
                color = Color(0xFFFCA5A5),
                modifier = Modifier.weight(1f)
            )
            Icon(
                painter = painterResource(id = R.drawable.ic_close),
                contentDescription = "Dismiss",
                tint = Color(0xFFFCA5A5),
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
    allItems: List<ReflexCommandItem>,
    selectedCategory: CommandCategory,
    searchQuery: String,
    onCategoryChange: (CommandCategory) -> Unit,
    onSearchQueryChange: (String) -> Unit,
    onCommandClick: (ReflexCommandItem) -> Unit
) {
    val filteredItems = remember(allItems, selectedCategory, searchQuery) {
        allItems.filter { item ->
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
        // Category Filter Chips Row
        item {
            LazyRow(
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                contentPadding = PaddingValues(vertical = 6.dp)
            ) {
                items(CommandCategory.values()) { cat ->
                    val isSelected = cat == selectedCategory
                    Surface(
                        color = if (isSelected) OtoActiveNavPill else OtoDarkSurface,
                        shape = RoundedCornerShape(16.dp),
                        modifier = Modifier
                            .border(0.8.dp, if (isSelected) OtoPlayerPlayBtnBg else OtoDarkOutline, RoundedCornerShape(16.dp))
                            .clip(RoundedCornerShape(16.dp))
                            .clickable { onCategoryChange(cat) }
                    ) {
                        Text(
                            text = cat.label,
                            fontSize = 12.sp,
                            fontWeight = if (isSelected) FontWeight.Bold else FontWeight.Medium,
                            color = if (isSelected) OtoActiveNavIcon else OtoTextSecondary,
                            modifier = Modifier.padding(horizontal = 14.dp, vertical = 7.dp)
                        )
                    }
                }
            }
        }

        items(filteredItems, key = { it.id }) { item ->
            Surface(
                color = OtoDarkCard,
                shape = RoundedCornerShape(14.dp),
                modifier = Modifier
                    .fillMaxWidth()
                    .border(0.8.dp, OtoDarkOutlineVariant, RoundedCornerShape(14.dp))
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
                            .background(OtoDarkSurfaceVariant),
                        contentAlignment = Alignment.Center
                    ) {
                        Icon(
                            painter = painterResource(id = R.drawable.ic_terminal),
                            contentDescription = item.title,
                            tint = OtoPlayerActiveControl,
                            modifier = Modifier.size(20.dp)
                        )
                    }

                    Column(modifier = Modifier.weight(1f)) {
                        Text(
                            text = item.title,
                            fontSize = 14.sp,
                            fontWeight = FontWeight.SemiBold,
                            color = OtoTextPrimary
                        )
                        Text(
                            text = item.description,
                            fontSize = 12.sp,
                            color = OtoTextSecondary,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis
                        )
                        Text(
                            text = item.command,
                            fontSize = 11.sp,
                            fontFamily = FontFamily.Monospace,
                            fontWeight = FontWeight.Bold,
                            color = OtoPlayerPlayBtnBg,
                            modifier = Modifier.padding(top = 2.dp)
                        )
                    }

                    Icon(
                        painter = painterResource(id = R.drawable.ic_play_arrow),
                        contentDescription = "Run",
                        tint = OtoTextSecondary,
                        modifier = Modifier.size(20.dp)
                    )
                }
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
    onPing: () -> Unit
) {
    var hostInput by remember(uiState.pcHost) { mutableStateOf(uiState.pcHost) }
    var portInput by remember(uiState.pcPort) { mutableStateOf(uiState.pcPort.toString()) }
    var selectedTheme by remember(uiState.themeMode) { mutableStateOf(uiState.themeMode) }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        Text(
            text = "Settings",
            fontSize = 24.sp,
            fontWeight = FontWeight.Bold,
            color = OtoTextPrimary
        )

        Card(
            modifier = Modifier
                .fillMaxWidth()
                .border(1.dp, OtoDarkOutline, RoundedCornerShape(18.dp)),
            colors = CardDefaults.cardColors(containerColor = OtoDarkSurface),
            shape = RoundedCornerShape(18.dp)
        ) {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(16.dp),
                verticalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                Text(
                    text = "PC Connection",
                    fontSize = 16.sp,
                    fontWeight = FontWeight.Bold,
                    color = OtoTextPrimary
                )

                OutlinedTextField(
                    value = hostInput,
                    onValueChange = { hostInput = it },
                    label = { Text("PC IP Address") },
                    singleLine = true,
                    colors = OutlinedTextFieldDefaults.colors(
                        focusedTextColor = OtoTextPrimary,
                        unfocusedTextColor = OtoTextPrimary,
                        focusedBorderColor = OtoPlayerPlayBtnBg,
                        unfocusedBorderColor = OtoDarkOutline
                    ),
                    modifier = Modifier.fillMaxWidth()
                )

                OutlinedTextField(
                    value = portInput,
                    onValueChange = { portInput = it },
                    label = { Text("Port (default 9421)") },
                    singleLine = true,
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
                    colors = OutlinedTextFieldDefaults.colors(
                        focusedTextColor = OtoTextPrimary,
                        unfocusedTextColor = OtoTextPrimary,
                        focusedBorderColor = OtoPlayerPlayBtnBg,
                        unfocusedBorderColor = OtoDarkOutline
                    ),
                    modifier = Modifier.fillMaxWidth()
                )

                Button(
                    onClick = {
                        val port = portInput.toIntOrNull() ?: 9421
                        onSaveSettings(hostInput, port, selectedTheme)
                    },
                    colors = ButtonDefaults.buttonColors(containerColor = OtoPlayerPlayBtnBg, contentColor = OtoPlayerPlayBtnIcon),
                    shape = RoundedCornerShape(12.dp),
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Text("Save & Connect", fontWeight = FontWeight.Bold)
                }
            }
        }
    }
}
