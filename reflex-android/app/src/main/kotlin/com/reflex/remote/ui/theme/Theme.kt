package com.reflex.remote.ui.theme

import android.app.Activity
import android.os.Build
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.SideEffect
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalView
import androidx.core.view.WindowCompat

enum class ThemeMode {
    SYSTEM,
    LIGHT,
    DARK
}

val OtoDarkColorScheme = darkColorScheme(
    primary = OtoPlayerPlayBtnBg,
    onPrimary = OtoPlayerPlayBtnIcon,
    primaryContainer = OtoActiveNavPill,
    onPrimaryContainer = OtoActiveNavIcon,
    secondary = OtoHistoryIcon,
    onSecondary = OtoDarkBackground,
    secondaryContainer = OtoHistoryBg,
    onSecondaryContainer = OtoHistoryIcon,
    tertiary = OtoShuffleIcon,
    onTertiary = OtoDarkBackground,
    tertiaryContainer = OtoShuffleBg,
    onTertiaryContainer = OtoShuffleIcon,
    error = OtoErrorRed,
    onError = Color.White,
    errorContainer = Color(0xFF450A0A),
    onErrorContainer = Color(0xFFFCA5A5),
    background = OtoDarkBackground,
    onBackground = OtoTextPrimary,
    surface = OtoDarkSurface,
    onSurface = OtoTextPrimary,
    surfaceVariant = OtoDarkSurfaceVariant,
    onSurfaceVariant = OtoTextSecondary,
    surfaceContainerLowest = OtoDarkBackground,
    surfaceContainerLow = OtoDarkCard,
    surfaceContainer = OtoDarkSurface,
    surfaceContainerHigh = OtoDarkElevated,
    surfaceContainerHighest = Color(0xFF2C374D),
    outline = OtoDarkOutline,
    outlineVariant = OtoDarkOutlineVariant
)

val OtoLightColorScheme = lightColorScheme(
    primary = Color(0xFF005DA6),
    onPrimary = Color.White,
    primaryContainer = OtoLightActiveNavPill,
    onPrimaryContainer = OtoLightActiveNavIcon,
    secondary = Color(0xFF18355D),
    onSecondary = Color.White,
    secondaryContainer = Color(0xFFD6E3FF),
    onSecondaryContainer = Color(0xFF001D36),
    tertiary = Color(0xFF00684A),
    onTertiary = Color.White,
    tertiaryContainer = Color(0xFFA5F3D1),
    onTertiaryContainer = Color(0xFF002114),
    error = Color(0xFFBA1A1A),
    onError = Color.White,
    errorContainer = Color(0xFFFFDAD6),
    onErrorContainer = Color(0xFF410002),
    background = OtoLightBackground,
    onBackground = OtoLightTextPrimary,
    surface = OtoLightSurface,
    onSurface = OtoLightTextPrimary,
    surfaceVariant = OtoLightSurfaceVariant,
    onSurfaceVariant = OtoLightTextSecondary,
    surfaceContainerLowest = Color.White,
    surfaceContainerLow = OtoLightCard,
    surfaceContainer = OtoLightSurface,
    surfaceContainerHigh = OtoLightElevated,
    surfaceContainerHighest = Color(0xFFD4DAE6),
    outline = OtoLightOutline,
    outlineVariant = OtoLightOutlineVariant
)

@Composable
fun ReflexTheme(
    themeMode: ThemeMode = ThemeMode.SYSTEM,
    // The palette is the brand. Wallpaper-derived colors would replace it, so it stays off by default.
    dynamicColor: Boolean = false,
    content: @Composable () -> Unit
) {
    val darkTheme = when (themeMode) {
        ThemeMode.SYSTEM -> isSystemInDarkTheme()
        ThemeMode.LIGHT -> false
        ThemeMode.DARK -> true
    }

    val context = LocalContext.current
    val dynamicScheme = if (dynamicColor && Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
        if (darkTheme) dynamicDarkColorScheme(context) else dynamicLightColorScheme(context)
    } else null
    val colorScheme = dynamicScheme ?: if (darkTheme) OtoDarkColorScheme else OtoLightColorScheme

    val view = LocalView.current
    if (!view.isInEditMode) {
        SideEffect {
            val window = (view.context as? Activity)?.window ?: return@SideEffect
            WindowCompat.setDecorFitsSystemWindows(window, false)
            val insetsController = WindowCompat.getInsetsController(window, view)
            insetsController.isAppearanceLightStatusBars = !darkTheme
            insetsController.isAppearanceLightNavigationBars = !darkTheme
        }
    }

    MaterialTheme(
        colorScheme = colorScheme,
        typography = Typography,
        content = content
    )
}
