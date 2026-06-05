package io.torvox.ui

import android.net.Uri
import android.provider.OpenableColumns
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
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
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import io.torvox.R
import io.torvox.TerminalViewModel
import io.torvox.ui.theme.TerminalTheme

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen(
    viewModel: TerminalViewModel,
    onBack: () -> Unit,
) {
    val fontSize by viewModel.fontSize.collectAsState()
    val fontFamily by viewModel.fontFamily.collectAsState()
    val dayThemeName by viewModel.dayThemeName.collectAsState()
    val nightThemeName by viewModel.nightThemeName.collectAsState()
    val themeMode by viewModel.themeMode.collectAsState()
    val selectedShell by viewModel.shell.collectAsState()
    val scrollbackLines by viewModel.scrollbackLines.collectAsState()
    val materialYouEnabled by viewModel.materialYouEnabled.collectAsState()

    val isDark = themeMode == "night" || (themeMode == "follow_system" && isSystemDarkTheme())
    val bgColor = if (isDark) Color(0xFF1E1E2E) else Color(0xFFF5F0EB)
    val textColor = if (isDark) Color(0xFFCDD6F4) else Color(0xFF1C1B1F)
    val secondaryText = if (isDark) Color(0xFFA6ADC8) else Color(0xFF5F5E63)
    val cardBg = if (isDark) Color(0xFF313244) else Color(0xFFFFFFFF)
    val accentColor = if (isDark) Color(0xFF89B4FA) else Color(0xFF1E66F5)
    val sectionTitleColor = if (isDark) Color(0xFF89B4FA) else Color(0xFF1E66F5)

    var availableFonts by remember { mutableStateOf<List<String>>(emptyList()) }
    LaunchedEffect(Unit) {
        val fonts = viewModel.runtime.bridge()?.listFonts() ?: emptyList()
        availableFonts = fonts
    }

    val customFontLauncher =
        rememberLauncherForActivityResult(
            contract = ActivityResultContracts.OpenDocument(),
        ) { uri: Uri? ->
            if (uri != null) {
                val fileName = viewModel.getFileNameFromUri(uri) ?: uri.lastPathSegment ?: "custom"
                viewModel.setFontFamily(fileName)
            }
        }

    Surface(
        modifier = Modifier.fillMaxSize().testTag("SettingsScreen"),
        color = bgColor,
    ) {
        Column(modifier = Modifier.fillMaxSize()) {
            LazyColumn(
                modifier = Modifier.fillMaxSize().padding(horizontal = 16.dp),
                verticalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                item {
                    Spacer(modifier = Modifier.height(8.dp))
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        IconButton(onClick = onBack, modifier = Modifier.testTag("SettingsBackButton")) {
                            Icon(
                                Icons.AutoMirrored.Filled.ArrowBack,
                                contentDescription = stringResource(R.string.back),
                                tint = textColor,
                            )
                        }
                        Spacer(modifier = Modifier.width(8.dp))
                        Text(
                            text = stringResource(R.string.settings),
                            style = MaterialTheme.typography.headlineSmall,
                            color = textColor,
                            fontWeight = FontWeight.Bold,
                        )
                    }
                }

                item {
                    SectionHeader(stringResource(R.string.appearance), sectionTitleColor)
                    SettingsCard(cardBg) {
                        FontSizeSlider(
                            modifier = Modifier.testTag("FontSizeSlider"),
                            value = fontSize,
                            onValueChange = { viewModel.setFontSize(it) },
                            textColor = textColor,
                            secondaryText = secondaryText,
                            accentColor = accentColor,
                        )
                        Spacer(modifier = Modifier.height(12.dp))
                        SystemFontSelector(
                            selectedFamily = fontFamily,
                            onFamilySelected = { viewModel.setFontFamily(it) },
                            textColor = textColor,
                            cardBg = bgColor,
                            accentColor = accentColor,
                            fonts = availableFonts,
                            onPickFontFile = { customFontLauncher.launch(arrayOf("font/*", "application/octet-stream")) },
                        )
                    }
                }

                item {
                    SectionHeader(stringResource(R.string.theme), sectionTitleColor)
                    SettingsCard(cardBg) {
                        MaterialYouToggle(
                            enabled = materialYouEnabled,
                            onToggle = { viewModel.setMaterialYouEnabled(it) },
                            textColor = textColor,
                            secondaryText = secondaryText,
                        )
                        if (!materialYouEnabled) {
                            Spacer(modifier = Modifier.height(8.dp))
                            ThemeModeSelector(
                                selectedMode = themeMode,
                                onModeSelected = { viewModel.setThemeMode(it) },
                                textColor = textColor,
                                secondaryText = secondaryText,
                                cardBg = bgColor,
                                accentColor = accentColor,
                            )
                            if (themeMode == "follow_system") {
                                Spacer(modifier = Modifier.height(8.dp))
                                ThemeSelector(
                                    label = stringResource(R.string.night_theme),
                                    selectedTheme = nightThemeName,
                                    themes = io.torvox.ui.theme.BuiltInThemes.darkThemes,
                                    onThemeSelected = { viewModel.setNightThemeName(it) },
                                    textColor = textColor,
                                    cardBg = bgColor,
                                )
                                Spacer(modifier = Modifier.height(8.dp))
                                ThemeSelector(
                                    label = stringResource(R.string.day_theme),
                                    selectedTheme = dayThemeName,
                                    themes = io.torvox.ui.theme.BuiltInThemes.lightThemes,
                                    onThemeSelected = { viewModel.setDayThemeName(it) },
                                    textColor = textColor,
                                    cardBg = bgColor,
                                )
                            } else {
                                Spacer(modifier = Modifier.height(8.dp))
                                ThemeSelector(
                                    label =
                                        if (themeMode ==
                                            "day"
                                        ) {
                                            stringResource(R.string.day_theme)
                                        } else {
                                            stringResource(R.string.night_theme)
                                        },
                                    selectedTheme = if (themeMode == "day") dayThemeName else nightThemeName,
                                    themes =
                                        if (themeMode ==
                                            "day"
                                        ) {
                                            io.torvox.ui.theme.BuiltInThemes.lightThemes
                                        } else {
                                            io.torvox.ui.theme.BuiltInThemes.darkThemes
                                        },
                                    onThemeSelected = {
                                        if (themeMode == "day") viewModel.setDayThemeName(it) else viewModel.setNightThemeName(it)
                                    },
                                    textColor = textColor,
                                    cardBg = bgColor,
                                )
                            }
                        }
                    }
                }

                item {
                    SectionHeader(stringResource(R.string.terminal), sectionTitleColor)
                    SettingsCard(cardBg) {
                        ShellInput(
                            shellPath = selectedShell,
                            onShellChanged = { viewModel.setShell(it) },
                            textColor = textColor,
                            accentColor = accentColor,
                        )
                        Spacer(modifier = Modifier.height(12.dp))
                        ScrollbackSlider(
                            value = scrollbackLines.toFloat(),
                            onValueChange = { viewModel.setScrollbackLines(it.toInt()) },
                            textColor = textColor,
                            secondaryText = secondaryText,
                            accentColor = accentColor,
                        )
                    }
                }

                item {
                    Spacer(modifier = Modifier.height(24.dp))
                }
            }
        }
    }
}

@Composable
private fun SettingsCard(
    cardBg: Color,
    content: @Composable () -> Unit,
) {
    Box(
        modifier =
            Modifier
                .fillMaxWidth()
                .clip(RoundedCornerShape(12.dp))
                .background(cardBg)
                .padding(16.dp),
    ) {
        content()
    }
}

@Composable
private fun SectionHeader(
    title: String,
    textColor: Color,
) {
    Text(
        text = title,
        style = MaterialTheme.typography.titleMedium,
        fontWeight = FontWeight.Bold,
        color = textColor,
        modifier = Modifier.padding(vertical = 4.dp),
    )
}

@Composable
private fun FontSizeSlider(
    modifier: Modifier = Modifier,
    value: Float,
    onValueChange: (Float) -> Unit,
    textColor: Color,
    secondaryText: Color,
    accentColor: Color,
) {
    Column(modifier = modifier) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
        ) {
            Text(stringResource(R.string.font_size), style = MaterialTheme.typography.bodyLarge, color = textColor)
            Text(
                text = "%.0f".format(value),
                style = MaterialTheme.typography.bodyMedium,
                color = secondaryText,
            )
        }
        Slider(
            value = value,
            onValueChange = onValueChange,
            valueRange = 8f..32f,
            steps = 23,
            colors = SliderDefaults.colors(thumbColor = accentColor, activeTrackColor = accentColor),
        )
    }
}

@Composable
private fun SystemFontSelector(
    selectedFamily: String,
    onFamilySelected: (String) -> Unit,
    textColor: Color,
    cardBg: Color,
    accentColor: Color,
    fonts: List<String> = emptyList(),
    onPickFontFile: (() -> Unit)? = null,
) {
    val systemFonts = remember(fonts) { (fonts + "default").distinct().sorted() }

    Spacer(modifier = Modifier.height(8.dp))
    Text(stringResource(R.string.font_family), style = MaterialTheme.typography.bodyLarge, color = textColor)
    Spacer(modifier = Modifier.height(4.dp))
    var showFontPicker by remember { mutableStateOf(false) }

    Box {
        Row(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(8.dp))
                    .background(cardBg)
                    .clickable {
                        showFontPicker = true
                    }.padding(horizontal = 12.dp, vertical = 12.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(text = if (selectedFamily.isEmpty()) stringResource(R.string.system_default) else selectedFamily, color = textColor)
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(text = stringResource(R.string.change), color = accentColor, style = MaterialTheme.typography.bodySmall)
            }
        }

        if (showFontPicker) {
            FontPickerDialog(
                fonts = systemFonts,
                selectedFamily = selectedFamily,
                onFamilySelected = { font ->
                    onFamilySelected(font)
                    showFontPicker = false
                },
                onDismiss = { showFontPicker = false },
                textColor = textColor,
                cardBg = cardBg,
                accentColor = accentColor,
                onPickFontFile = { onPickFontFile?.invoke() },
            )
        }
    }
}

@Composable
private fun FontPickerDialog(
    fonts: List<String>,
    selectedFamily: String,
    onFamilySelected: (String) -> Unit,
    onDismiss: () -> Unit,
    textColor: Color,
    cardBg: Color,
    accentColor: Color,
    onPickFontFile: (() -> Unit)? = null,
) {
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(stringResource(R.string.select_font_family), color = textColor) },
        text = {
            LazyColumn {
                item {
                    Row(
                        modifier =
                            Modifier
                                .fillMaxWidth()
                                .clip(RoundedCornerShape(6.dp))
                                .clickable {
                                    onPickFontFile?.invoke()
                                    onDismiss()
                                }.padding(horizontal = 12.dp, vertical = 10.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Text(text = stringResource(R.string.pick_font_file), color = accentColor, fontWeight = FontWeight.Bold)
                    }
                }
                items(fonts) { font ->
                    Row(
                        modifier =
                            Modifier
                                .fillMaxWidth()
                                .clip(RoundedCornerShape(6.dp))
                                .clickable { onFamilySelected(font) }
                                .background(
                                    if (selectedFamily == font) {
                                        accentColor.copy(alpha = 0.2f)
                                    } else {
                                        Color.Transparent
                                    },
                                ).padding(horizontal = 12.dp, vertical = 10.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Text(
                            text = font,
                            color = if (selectedFamily == font) accentColor else textColor,
                            fontWeight = if (selectedFamily == font) FontWeight.Bold else FontWeight.Normal,
                        )
                    }
                }
            }
        },
        confirmButton = { TextButton(onClick = onDismiss) { Text(stringResource(R.string.cancel), color = textColor) } },
        containerColor = cardBg,
    )
}

@Composable
private fun ThemeModeSelector(
    selectedMode: String,
    onModeSelected: (String) -> Unit,
    textColor: Color,
    secondaryText: Color,
    cardBg: Color,
    accentColor: Color,
) {
    Column {
        Text(stringResource(R.string.theme_mode), style = MaterialTheme.typography.bodyLarge, color = textColor)
        Spacer(modifier = Modifier.height(4.dp))
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            listOf(
                "follow_system" to stringResource(R.string.follow_system),
                "day" to stringResource(R.string.day),
                "night" to stringResource(R.string.night),
            ).forEach { (mode, label) ->
                val isSelected = selectedMode == mode
                Box(
                    modifier =
                        Modifier
                            .weight(
                                1f,
                            ).clip(RoundedCornerShape(8.dp))
                            .background(if (isSelected) accentColor else cardBg)
                            .clickable {
                                onModeSelected(mode)
                            }.padding(vertical = 12.dp),
                    contentAlignment = Alignment.Center,
                ) {
                    Text(text = label, color = if (isSelected) Color.White else textColor, style = MaterialTheme.typography.bodyMedium)
                }
            }
        }
    }
}

@Composable
private fun MaterialYouToggle(
    enabled: Boolean,
    onToggle: (Boolean) -> Unit,
    textColor: Color,
    secondaryText: Color,
) {
    Row(
        modifier =
            Modifier
                .fillMaxWidth()
                .clip(RoundedCornerShape(8.dp))
                .clickable {
                    onToggle(!enabled)
                }.padding(vertical = 12.dp, horizontal = 8.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceBetween,
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text("Material You", style = MaterialTheme.typography.bodyLarge, color = textColor)
            Text(
                text = "Use wallpaper-based dynamic colors (Android 12+)",
                style = MaterialTheme.typography.bodySmall,
                color = secondaryText,
            )
        }
        androidx.compose.material3.Switch(checked = enabled, onCheckedChange = onToggle)
    }
}

@Composable
private fun ShellInput(
    shellPath: String,
    onShellChanged: (String) -> Unit,
    textColor: Color,
    accentColor: Color,
) {
    Column {
        Text(stringResource(R.string.shell), style = MaterialTheme.typography.bodyLarge, color = textColor)
        Spacer(modifier = Modifier.height(4.dp))
        var text by remember(shellPath) { mutableStateOf(shellPath) }
        OutlinedTextField(
            value = text,
            onValueChange = {
                text = it
                onShellChanged(it)
            },
            modifier = Modifier.fillMaxWidth(),
            singleLine = true,
            placeholder = { Text(stringResource(R.string.shell_placeholder), color = textColor.copy(alpha = 0.5f)) },
            textStyle = MaterialTheme.typography.bodyLarge.copy(color = textColor),
            colors =
                OutlinedTextFieldDefaults.colors(
                    focusedTextColor = textColor,
                    unfocusedTextColor = textColor,
                    cursorColor = accentColor,
                    focusedBorderColor = accentColor,
                    unfocusedBorderColor = textColor.copy(alpha = 0.5f),
                ),
        )
    }
}

@Composable
private fun ScrollbackSlider(
    value: Float,
    onValueChange: (Float) -> Unit,
    textColor: Color,
    secondaryText: Color,
    accentColor: Color,
) {
    Column {
        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Text(stringResource(R.string.scrollback_lines), style = MaterialTheme.typography.bodyLarge, color = textColor)
            Text(
                text =
                    value.toInt().let {
                        if (it >=
                            1000
                        ) {
                            "${it / 1000}K"
                        } else {
                            "$it"
                        }
                    },
                style = MaterialTheme.typography.bodyMedium,
                color = secondaryText,
            )
        }
        Slider(
            value = value,
            onValueChange = onValueChange,
            valueRange = 1000f..100000f,
            steps = 98,
            colors = SliderDefaults.colors(thumbColor = accentColor, activeTrackColor = accentColor),
        )
    }
}

@Composable
private fun ThemeSelector(
    label: String,
    selectedTheme: String,
    themes: List<TerminalTheme>,
    onThemeSelected: (String) -> Unit,
    textColor: Color,
    cardBg: Color,
) {
    Column {
        Text(label, style = MaterialTheme.typography.bodyLarge, color = textColor)
        Spacer(modifier = Modifier.height(8.dp))
        LazyRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            items(themes) { theme ->
                ThemePreview(theme = theme, isSelected = theme.name == selectedTheme, onClick = { onThemeSelected(theme.name) })
            }
        }
    }
}

@Composable
private fun ThemePreview(
    theme: TerminalTheme,
    isSelected: Boolean,
    onClick: () -> Unit,
) {
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        modifier =
            Modifier
                .width(
                    80.dp,
                ).clip(
                    RoundedCornerShape(8.dp),
                ).background(if (isSelected) theme.background else theme.background.copy(alpha = 0.7f))
                .clickable(onClick = onClick)
                .padding(8.dp),
    ) {
        Row(horizontalArrangement = Arrangement.spacedBy(2.dp)) {
            theme.ansi.take(8).forEach { color ->
                Box(modifier = Modifier.size(8.dp).clip(RoundedCornerShape(2.dp)).background(color))
            }
        }
        Spacer(modifier = Modifier.height(4.dp))
        Text(text = theme.name, style = MaterialTheme.typography.labelSmall, color = theme.foreground, maxLines = 1)
    }
}

@Composable
private fun isSystemDarkTheme(): Boolean {
    val context = LocalContext.current
    return (context.resources.configuration.uiMode and android.content.res.Configuration.UI_MODE_NIGHT_MASK) ==
        android.content.res.Configuration.UI_MODE_NIGHT_YES
}
