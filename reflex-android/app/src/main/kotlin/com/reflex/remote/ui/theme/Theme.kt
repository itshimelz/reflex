package com.reflex.remote.ui.theme

import android.app.Activity
import android.os.Build
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.ColorScheme
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

val PixelLightColorScheme = lightColorScheme(
    primary = PixelPrimaryLight,
    onPrimary = PixelOnPrimaryLight,
    primaryContainer = PixelPrimaryContainerLight,
    onPrimaryContainer = PixelOnPrimaryContainerLight,
    secondary = PixelSecondaryLight,
    onSecondary = PixelOnSecondaryLight,
    secondaryContainer = PixelSecondaryContainerLight,
    onSecondaryContainer = PixelOnSecondaryContainerLight,
    tertiary = PixelTertiaryLight,
    onTertiary = PixelOnTertiaryLight,
    tertiaryContainer = PixelTertiaryContainerLight,
    onTertiaryContainer = PixelOnTertiaryContainerLight,
    error = PixelErrorLight,
    onError = PixelOnErrorLight,
    errorContainer = PixelErrorContainerLight,
    onErrorContainer = PixelOnErrorContainerLight,
    background = PixelBackgroundLight,
    onBackground = PixelOnBackgroundLight,
    surface = PixelSurfaceLight,
    onSurface = PixelOnSurfaceLight,
    surfaceVariant = PixelSurfaceVariantLight,
    onSurfaceVariant = PixelOnSurfaceVariantLight,
    surfaceDim = PixelSurfaceDimLight,
    surfaceBright = PixelSurfaceBrightLight,
    surfaceContainerLowest = PixelSurfaceContainerLowestLight,
    surfaceContainerLow = PixelSurfaceContainerLowLight,
    surfaceContainer = PixelSurfaceContainerLight,
    surfaceContainerHigh = PixelSurfaceContainerHighLight,
    surfaceContainerHighest = PixelSurfaceContainerHighestLight,
    outline = PixelOutlineLight,
    outlineVariant = PixelOutlineVariantLight
)

val PixelDarkColorScheme = darkColorScheme(
    primary = PixelPrimaryDark,
    onPrimary = PixelOnPrimaryDark,
    primaryContainer = PixelPrimaryContainerDark,
    onPrimaryContainer = PixelOnPrimaryContainerDark,
    secondary = PixelSecondaryDark,
    onSecondary = PixelOnSecondaryDark,
    secondaryContainer = PixelSecondaryContainerDark,
    onSecondaryContainer = PixelOnSecondaryContainerDark,
    tertiary = PixelTertiaryDark,
    onTertiary = PixelOnTertiaryDark,
    tertiaryContainer = PixelTertiaryContainerDark,
    onTertiaryContainer = PixelOnTertiaryContainerDark,
    error = PixelErrorDark,
    onError = PixelOnErrorDark,
    errorContainer = PixelErrorContainerDark,
    onErrorContainer = PixelOnErrorContainerDark,
    background = PixelBackgroundDark,
    onBackground = PixelOnBackgroundDark,
    surface = PixelSurfaceDark,
    onSurface = PixelOnSurfaceDark,
    surfaceVariant = PixelSurfaceVariantDark,
    onSurfaceVariant = PixelOnSurfaceVariantDark,
    surfaceDim = PixelSurfaceDimDark,
    surfaceBright = PixelSurfaceBrightDark,
    surfaceContainerLowest = PixelSurfaceContainerLowestDark,
    surfaceContainerLow = PixelSurfaceContainerLowDark,
    surfaceContainer = PixelSurfaceContainerDark,
    surfaceContainerHigh = PixelSurfaceContainerHighDark,
    surfaceContainerHighest = PixelSurfaceContainerHighestDark,
    outline = PixelOutlineDark,
    outlineVariant = PixelOutlineVariantDark
)

@Composable
fun ReflexTheme(
    themeMode: ThemeMode = ThemeMode.SYSTEM,
    dynamicColor: Boolean = false,
    content: @Composable () -> Unit
) {
    val darkTheme = when (themeMode) {
        ThemeMode.SYSTEM -> isSystemInDarkTheme()
        ThemeMode.LIGHT -> false
        ThemeMode.DARK -> true
    }

    val context = LocalContext.current
    val colorScheme = when {
        dynamicColor && Build.VERSION.SDK_INT >= Build.VERSION_CODES.S -> {
            if (darkTheme) dynamicDarkColorScheme(context) else dynamicLightColorScheme(context)
        }
        darkTheme -> PixelDarkColorScheme
        else -> PixelLightColorScheme
    }

    val view = LocalView.current
    if (!view.isInEditMode) {
        SideEffect {
            val window = (view.context as? Activity)?.window
            if (window != null) {
                WindowCompat.getInsetsController(window, view).apply {
                    isAppearanceLightStatusBars = !darkTheme
                    isAppearanceLightNavigationBars = !darkTheme
                }
            }
        }
    }

    MaterialTheme(
        colorScheme = colorScheme,
        typography = Typography,
        content = content
    )
}
