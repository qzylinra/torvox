package io.torvox.ui

import android.content.Context
import android.net.Uri
import android.util.Log
import androidx.activity.compose.BackHandler
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
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
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.RadioButton
import androidx.compose.material3.RadioButtonDefaults
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import io.torvox.R
import io.torvox.TerminalViewModel
import io.torvox.ui.theme.TerminalTheme
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

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
    val appThemeMode by viewModel.appThemeMode.collectAsState()
    val selectedShell by viewModel.shell.collectAsState()
    val scrollbackLines by viewModel.scrollbackLines.collectAsState()
    val themeName by viewModel.themeName.collectAsState()
    val useNerdFontGlyphs by viewModel.useNerdFontGlyphs.collectAsState()
    val useSemanticSelection by viewModel.useSemanticSelection.collectAsState()
    val sessionRestore by viewModel.sessionRestore.collectAsState()

    val screenWidthDp = LocalConfiguration.current.screenWidthDp
    val isSmallScreen = screenWidthDp < 400
    val horizontalPadding = if (isSmallScreen) 8.dp else 16.dp

    val bgColor = MaterialTheme.colorScheme.surface
    val textColor = MaterialTheme.colorScheme.onSurface
    val secondaryText = MaterialTheme.colorScheme.onSurfaceVariant
    val cardBg = MaterialTheme.colorScheme.surfaceContainerLow
    val accentColor = MaterialTheme.colorScheme.primary
    val sectionTitleColor = MaterialTheme.colorScheme.primary

    val availableFonts by viewModel.availableFonts.collectAsState()

    val customFontLauncher =
        rememberLauncherForActivityResult(
            contract = ActivityResultContracts.OpenDocument(),
        ) { uri: Uri? ->
            if (uri != null) {
                val fileName = viewModel.getFileNameFromUri(uri) ?: uri.lastPathSegment ?: "custom"
                viewModel.setFontFamily(fileName)
            }
        }

    BackHandler(enabled = true) {
        onBack()
    }

    Surface(
        modifier = Modifier.fillMaxSize().testTag("SettingsScreen"),
        color = bgColor,
    ) {
        Column(
            modifier =
            Modifier
                .fillMaxSize()
                .statusBarsPadding()
                .navigationBarsPadding(),
        ) {
            LazyColumn(
                modifier = Modifier.fillMaxSize().padding(horizontal = horizontalPadding),
                verticalArrangement = Arrangement.spacedBy(if (isSmallScreen) 8.dp else 12.dp),
                contentPadding =
                androidx.compose.foundation.layout
                    .PaddingValues(bottom = 32.dp),
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
                        Spacer(modifier = Modifier.width(4.dp))
                        Text(
                            text = stringResource(R.string.settings),
                            style = if (isSmallScreen) MaterialTheme.typography.titleLarge else MaterialTheme.typography.headlineSmall,
                            color = textColor,
                            fontWeight = FontWeight.Bold,
                        )
                    }
                }

                item {
                    SectionHeader(stringResource(R.string.appearance), sectionTitleColor)
                    SettingsCard(cardBg, isSmallScreen) {
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
                    SectionHeader(stringResource(R.string.software_theme), sectionTitleColor)
                    SettingsCard(cardBg, isSmallScreen) {
                        AppThemeSelector(
                            selectedMode = appThemeMode,
                            onModeSelected = { viewModel.setAppThemeMode(it) },
                            textColor = textColor,
                            cardBg = bgColor,
                            accentColor = accentColor,
                        )
                    }
                }

                item {
                    SectionHeader(stringResource(R.string.theme), sectionTitleColor)
                    SettingsCard(cardBg, isSmallScreen) {
                        TerminalThemeModeSelector(
                            selectedMode = themeMode,
                            onModeSelected = { viewModel.setThemeMode(it) },
                            textColor = textColor,
                            cardBg = bgColor,
                            accentColor = accentColor,
                        )
                        Spacer(modifier = Modifier.height(8.dp))
                        when (themeMode) {
                            "follow_system", "day", "night" -> {
                                ThemeSelector(
                                    label = stringResource(R.string.day_theme),
                                    selectedTheme = dayThemeName,
                                    themes = io.torvox.ui.theme.BuiltInThemes.all,
                                    onThemeSelected = { viewModel.setDayThemeName(it) },
                                    textColor = textColor,
                                    cardBg = bgColor,
                                )
                                Spacer(modifier = Modifier.height(8.dp))
                                ThemeSelector(
                                    label = stringResource(R.string.night_theme),
                                    selectedTheme = nightThemeName,
                                    themes = io.torvox.ui.theme.BuiltInThemes.all,
                                    onThemeSelected = { viewModel.setNightThemeName(it) },
                                    textColor = textColor,
                                    cardBg = bgColor,
                                )
                            }

                            "fixed" -> {
                                ThemeSelector(
                                    label = "",
                                    selectedTheme = themeName,
                                    themes = io.torvox.ui.theme.BuiltInThemes.all,
                                    onThemeSelected = { viewModel.setThemeName(it) },
                                    textColor = textColor,
                                    cardBg = bgColor,
                                )
                            }
                        }
                    }
                }

                item {
                    SectionHeader(stringResource(R.string.terminal), sectionTitleColor)
                    SettingsCard(cardBg, isSmallScreen) {
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
                        Spacer(modifier = Modifier.height(8.dp))
                        SessionRestoreToggle(
                            enabled = sessionRestore,
                            onToggle = { viewModel.setSessionRestore(it) },
                            textColor = textColor,
                            accentColor = accentColor,
                            cardBg = bgColor,
                        )
                        Spacer(modifier = Modifier.height(8.dp))
                        KeyboardModeSelector(
                            selectedMode = viewModel.keyboardMode.collectAsState().value,
                            onModeSelected = { viewModel.setKeyboardMode(it) },
                            textColor = textColor,
                            accentColor = accentColor,
                            cardBg = bgColor,
                        )
                        Spacer(modifier = Modifier.height(8.dp))
                        UsbSerialToggle(
                            enabled = viewModel.usbSerialEnabled.collectAsState().value,
                            onToggle = { viewModel.setUsbSerialEnabled(it) },
                            textColor = textColor,
                            accentColor = accentColor,
                            cardBg = bgColor,
                        )
                        Spacer(modifier = Modifier.height(8.dp))
                        McpServerToggle(
                            enabled = viewModel.mcpServerEnabled.collectAsState().value,
                            onToggle = { viewModel.setMcpServerEnabled(it) },
                            textColor = textColor,
                            accentColor = accentColor,
                            cardBg = bgColor,
                        )
                        Spacer(modifier = Modifier.height(8.dp))
                        VolumeKeyMapToggle(
                            enabled = viewModel.volumeKeyMap.collectAsState().value,
                            onToggle = { viewModel.setVolumeKeyMap(it) },
                            textColor = textColor,
                            accentColor = accentColor,
                            cardBg = bgColor,
                        )
                    }
                }

                item {
                    SectionHeader(stringResource(R.string.bootstrap), sectionTitleColor)
                    SettingsCard(cardBg, isSmallScreen) {
                        BootstrapSection(
                            bootstrapUrl = viewModel.bootstrapUrl.collectAsState().value,
                            onUrlChanged = { viewModel.setBootstrapUrl(it) },
                            textColor = textColor,
                            accentColor = accentColor,
                            secondaryText = secondaryText,
                        )
                    }
                }

                item {
                    SectionHeader(stringResource(R.string.clear_app_data), sectionTitleColor)
                    SettingsCard(cardBg, isSmallScreen) {
                        ClearAppDataSection(textColor = textColor)
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
    isSmallScreen: Boolean = false,
    content: @Composable () -> Unit,
) {
    Column(
        modifier =
        Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(if (isSmallScreen) 8.dp else 12.dp))
            .background(cardBg)
            .padding(if (isSmallScreen) 12.dp else 16.dp),
    ) {
        content()
    }
}

@Composable
private fun SectionHeader(
    title: String,
    textColor: Color,
) {
    val screenWidthDp = LocalConfiguration.current.screenWidthDp
    val isSmallScreen = screenWidthDp < 400
    Text(
        text = title,
        style = if (isSmallScreen) MaterialTheme.typography.titleSmall else MaterialTheme.typography.titleMedium,
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
    val screenWidthDp = LocalConfiguration.current.screenWidthDp
    val isSmallScreen = screenWidthDp < 400
    val labelStyle = if (isSmallScreen) MaterialTheme.typography.bodyMedium else MaterialTheme.typography.bodyLarge
    val valueStyle = if (isSmallScreen) MaterialTheme.typography.bodySmall else MaterialTheme.typography.bodyMedium
    Column(modifier = modifier) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                stringResource(R.string.font_size),
                style = labelStyle,
                color = textColor,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.weight(1f, fill = false),
            )
            Spacer(modifier = Modifier.width(8.dp))
            Text(
                text = "%.0f".format(value),
                style = valueStyle,
                color = secondaryText,
            )
        }
        Slider(
            value = value,
            onValueChange = onValueChange,
            valueRange = 8f..48f,
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
    val systemFonts = remember(fonts) { fonts.distinct().sorted() }
    val screenWidthDp = LocalConfiguration.current.screenWidthDp
    val isSmallScreen = screenWidthDp < 400
    val labelStyle = if (isSmallScreen) MaterialTheme.typography.bodyMedium else MaterialTheme.typography.bodyLarge
    val systemDefaultLabel = stringResource(R.string.system_default)
    val defaultFontName =
        remember(systemDefaultLabel) {
            val tf = android.graphics.Typeface.DEFAULT
            try {
                val method = tf.javaClass.getMethod("getFamilyName")
                (method.invoke(tf) as? String) ?: systemDefaultLabel
            } catch (_: Exception) {
                systemDefaultLabel
            }
        }

    Spacer(modifier = Modifier.height(4.dp))
    Text(stringResource(R.string.font_family), style = labelStyle, color = textColor, maxLines = 1)
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
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                text =
                if (selectedFamily.isEmpty()) {
                    defaultFontName
                } else {
                    selectedFamily
                },
                color = textColor,
                modifier = Modifier.weight(1f),
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Text(text = stringResource(R.string.change), color = accentColor, style = MaterialTheme.typography.bodySmall)
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
                                onFamilySelected("")
                                onDismiss()
                            }.padding(horizontal = 12.dp, vertical = 10.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Text(
                            text = stringResource(R.string.system_default),
                            color = if (selectedFamily.isEmpty()) accentColor else textColor,
                            fontWeight = if (selectedFamily.isEmpty()) FontWeight.Bold else FontWeight.Normal,
                        )
                    }
                }
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
internal fun AppThemeSelector(
    selectedMode: String,
    onModeSelected: (String) -> Unit,
    textColor: Color,
    cardBg: Color,
    accentColor: Color,
) {
    val screenWidthDp = LocalConfiguration.current.screenWidthDp
    val isSmall = screenWidthDp < 400
    Column(modifier = Modifier.testTag("AppThemeSelector")) {
        val buttons =
            listOf(
                "day" to stringResource(R.string.day),
                "night" to stringResource(R.string.night),
                "follow_system" to stringResource(R.string.follow_system),
            )
        val spacing = if (isSmall) 4.dp else 6.dp
        if (isSmall) {
            Column(verticalArrangement = Arrangement.spacedBy(spacing)) {
                buttons.forEach { (mode, label) ->
                    val isSelected = selectedMode == mode
                    Box(
                        modifier =
                        Modifier
                            .testTag("AppTheme_$mode")
                            .fillMaxWidth()
                            .clip(RoundedCornerShape(6.dp))
                            .background(if (isSelected) accentColor else cardBg)
                            .clickable { onModeSelected(mode) }
                            .padding(vertical = 8.dp),
                        contentAlignment = Alignment.Center,
                    ) {
                        Text(text = label, color = if (isSelected) Color.White else textColor, style = MaterialTheme.typography.bodySmall)
                    }
                }
            }
        } else {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(spacing),
            ) {
                buttons.forEach { (mode, label) ->
                    val isSelected = selectedMode == mode
                    Box(
                        modifier =
                        Modifier
                            .testTag("AppTheme_$mode")
                            .weight(1f)
                            .clip(RoundedCornerShape(8.dp))
                            .background(if (isSelected) accentColor else cardBg)
                            .clickable { onModeSelected(mode) }
                            .padding(vertical = 10.dp, horizontal = 4.dp),
                        contentAlignment = Alignment.Center,
                    ) {
                        Text(text = label, color = if (isSelected) Color.White else textColor, style = MaterialTheme.typography.bodyMedium)
                    }
                }
            }
        }
    }
}

@Composable
internal fun TerminalThemeModeSelector(
    selectedMode: String,
    onModeSelected: (String) -> Unit,
    textColor: Color,
    cardBg: Color,
    accentColor: Color,
) {
    val isFollowSystem = selectedMode != "fixed"
    Row(
        modifier = Modifier.fillMaxWidth().testTag("TerminalThemeModeSelector"),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            text = stringResource(R.string.follow_system),
            style = MaterialTheme.typography.bodyLarge,
            color = textColor,
        )
        Switch(
            checked = isFollowSystem,
            onCheckedChange = { checked ->
                onModeSelected(if (checked) "follow_system" else "fixed")
            },
            modifier = Modifier.testTag("TerminalThemeFollowSystemSwitch"),
            colors =
            SwitchDefaults.colors(
                checkedThumbColor = Color.White,
                checkedTrackColor = accentColor,
                uncheckedThumbColor = textColor.copy(alpha = 0.6f),
                uncheckedTrackColor = cardBg,
            ),
        )
    }
}

@Composable
private fun ShellInput(
    shellPath: String,
    onShellChanged: (String) -> Unit,
    textColor: Color,
    accentColor: Color,
) {
    val screenWidthDp = LocalConfiguration.current.screenWidthDp
    val isSmallScreen = screenWidthDp < 400
    val labelStyle = if (isSmallScreen) MaterialTheme.typography.bodyMedium else MaterialTheme.typography.bodyLarge
    Column {
        Text(stringResource(R.string.shell), style = labelStyle, color = textColor)
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
    val screenWidthDp = LocalConfiguration.current.screenWidthDp
    val isSmallScreen = screenWidthDp < 400
    val labelStyle = if (isSmallScreen) MaterialTheme.typography.bodyMedium else MaterialTheme.typography.bodyLarge
    val valueStyle = if (isSmallScreen) MaterialTheme.typography.bodySmall else MaterialTheme.typography.bodyMedium
    Column {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                stringResource(R.string.scrollback_lines),
                style = labelStyle,
                color = textColor,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.weight(1f, fill = false),
            )
            Spacer(modifier = Modifier.width(8.dp))
            Text(
                text =
                value.toInt().let {
                    if (it >= 1000) {
                        "${it / 1000}K"
                    } else {
                        "$it"
                    }
                },
                style = valueStyle,
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
internal fun ThemeSelector(
    label: String,
    selectedTheme: String,
    themes: List<TerminalTheme>,
    onThemeSelected: (String) -> Unit,
    textColor: Color,
    cardBg: Color,
) {
    val screenWidthDp = LocalConfiguration.current.screenWidthDp
    val isSmallScreen = screenWidthDp < 400
    val labelStyle = if (isSmallScreen) MaterialTheme.typography.bodyMedium else MaterialTheme.typography.bodyLarge
    Column(modifier = Modifier.testTag("ThemeSelector")) {
        if (label.isNotEmpty()) {
            Text(label, style = labelStyle, color = textColor)
            Spacer(modifier = Modifier.height(4.dp))
        }
        LazyRow(horizontalArrangement = Arrangement.spacedBy(if (isSmallScreen) 6.dp else 8.dp)) {
            items(themes) { theme ->
                ThemePreview(
                    theme = theme,
                    isSelected = theme.name == selectedTheme,
                    onClick = { onThemeSelected(theme.name) },
                    isSmallScreen = isSmallScreen,
                )
            }
        }
    }
}

@Composable
private fun ThemePreview(
    theme: TerminalTheme,
    isSelected: Boolean,
    onClick: () -> Unit,
    isSmallScreen: Boolean = false,
) {
    val previewWidth = if (isSmallScreen) 72.dp else 88.dp
    val dotSize = if (isSmallScreen) 6.dp else 8.dp
    val padding = if (isSmallScreen) 4.dp else 6.dp
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        modifier =
        Modifier
            .testTag("theme_preview_${theme.name}")
            .width(previewWidth)
            .clickable(onClick = onClick),
    ) {
        Box(
            modifier =
            Modifier
                .fillMaxWidth()
                .clip(RoundedCornerShape(8.dp))
                .background(if (isSelected) theme.background else theme.background.copy(alpha = 0.7f))
                .padding(padding),
        ) {
            Row(horizontalArrangement = Arrangement.spacedBy(2.dp)) {
                theme.ansi.take(8).forEach { color ->
                    Box(modifier = Modifier.size(dotSize).clip(RoundedCornerShape(2.dp)).background(color))
                }
            }
        }
        Spacer(modifier = Modifier.height(4.dp))
        Text(
            text = theme.name,
            style = MaterialTheme.typography.labelSmall,
            color = MaterialTheme.colorScheme.onSurface,
            maxLines = 2,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

@Composable
private fun BootstrapSection(
    bootstrapUrl: String,
    onUrlChanged: (String) -> Unit,
    textColor: Color,
    accentColor: Color,
    secondaryText: Color,
) {
    var url by remember { mutableStateOf(bootstrapUrl) }
    LaunchedEffect(bootstrapUrl) { url = bootstrapUrl }

    OutlinedTextField(
        value = url,
        onValueChange = {
            url = it
            onUrlChanged(it)
        },
        label = { Text(stringResource(R.string.bootstrap_url_label)) },
        placeholder = { Text(stringResource(R.string.bootstrap_url_placeholder)) },
        singleLine = true,
        modifier = Modifier.fillMaxWidth(),
        colors =
        OutlinedTextFieldDefaults.colors(
            focusedBorderColor = accentColor,
            unfocusedBorderColor = textColor.copy(alpha = 0.5f),
            cursorColor = accentColor,
            focusedLabelColor = accentColor,
        ),
    )
    Spacer(modifier = Modifier.height(4.dp))
    Text(
        text = stringResource(R.string.bootstrap_desc),
        style = MaterialTheme.typography.bodySmall,
        color = secondaryText,
    )
    Spacer(modifier = Modifier.height(8.dp))
    Text(
        text = stringResource(R.string.bootstrap_presets),
        style = MaterialTheme.typography.bodyMedium,
        color = textColor,
    )
    Spacer(modifier = Modifier.height(4.dp))

    val arch =
        when (
            android.os.Build.SUPPORTED_ABIS
                .firstOrNull()
        ) {
            "arm64-v8a" -> "aarch64"
            "armeabi-v7a" -> "arm"
            "x86_64" -> "x86_64"
            "x86" -> "i686"
            else -> "aarch64"
        }
    val termuxUrl = "https://github.com/termux/termux-packages/releases/download/bootstrap/bootstrap-$arch.tar.xz"

    val presets =
        listOf(
            Triple(
                stringResource(R.string.bootstrap_preset_termux),
                termuxUrl,
                stringResource(R.string.bootstrap_preset_termux_desc),
            ),
        )
    presets.forEach { preset ->
        BootstrapPresetItem(
            preset = preset,
            colors = PresetColors(accentColor, textColor, secondaryText),
            onAction = {
                url = preset.second
                onUrlChanged(preset.second)
            },
        )
    }
}

private data class PresetColors(
    val accent: Color,
    val text: Color,
    val secondary: Color,
)

@Composable
private fun BootstrapPresetItem(
    preset: Triple<String, String, String>,
    colors: PresetColors,
    onAction: () -> Unit,
) {
    val (label, _, description) = preset
    Surface(
        onClick = onAction,
        shape = RoundedCornerShape(8.dp),
        color = colors.accent.copy(alpha = 0.08f),
        border = BorderStroke(1.dp, colors.accent),
        modifier = Modifier.fillMaxWidth().padding(vertical = 2.dp),
    ) {
        Row(
            modifier = Modifier.padding(12.dp).fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween,
        ) {
            Column(modifier = Modifier.weight(1f)) {
                Text(text = label, style = MaterialTheme.typography.bodyMedium, color = colors.accent)
                Text(text = description, style = MaterialTheme.typography.bodySmall, color = colors.secondary)
            }
            Text(
                text = stringResource(R.string.install),
                style = MaterialTheme.typography.labelMedium,
                color = colors.accent,
                modifier = Modifier.padding(start = 8.dp),
            )
        }
    }
}

internal fun fallbackSystemFonts(): List<String> {
    val fonts = mutableListOf<String>()
    val seen = mutableSetOf<String>()
    val dir = java.io.File("/system/fonts/")
    if (dir.isDirectory) {
        dir
            .listFiles()
            ?.filter { it.name.endsWith(".ttf", true) || it.name.endsWith(".otf", true) }
            ?.forEach { file ->
                val name =
                    file.nameWithoutExtension
                        .replace('_', ' ')
                        .replace('-', ' ')
                        .trim()
                if (name.isNotEmpty() && seen.add(name.lowercase())) {
                    fonts.add(name)
                }
            }
    }
    for (
    known in
    listOf(
        "JetBrainsMono Nerd Font",
        "Droid Sans Mono",
        "Noto Sans Mono",
        "Noto Sans SC",
        "Noto Sans CJK SC",
        "Noto Sans TC",
        "Noto Sans CJK TC",
        "Noto Sans JP",
        "Noto Sans KR",
        "DroidSansFallback",
        "Roboto Mono",
        "Source Code Pro",
        "Fira Code",
        "Ubuntu Mono",
    )
    ) {
        if (seen.add(known.lowercase())) {
            fonts.add(known)
        }
    }
    fonts.sort()
    return fonts
}

@Composable
private fun NerdFontToggle(
    enabled: Boolean,
    onToggle: (Boolean) -> Unit,
    textColor: Color,
    accentColor: Color,
    cardBg: Color,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            text = stringResource(R.string.use_nerd_font_glyphs),
            style = MaterialTheme.typography.bodyLarge,
            color = textColor,
        )
        Switch(
            checked = enabled,
            onCheckedChange = onToggle,
            colors =
            SwitchDefaults.colors(
                checkedThumbColor = Color.White,
                checkedTrackColor = accentColor,
                uncheckedThumbColor = textColor.copy(alpha = 0.6f),
                uncheckedTrackColor = cardBg,
            ),
        )
    }
}

@Composable
private fun SemanticSelectionToggle(
    enabled: Boolean,
    onToggle: (Boolean) -> Unit,
    textColor: Color,
    accentColor: Color,
    cardBg: Color,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            text = stringResource(R.string.semantic_selection),
            style = MaterialTheme.typography.bodyLarge,
            color = textColor,
        )
        Switch(
            checked = enabled,
            onCheckedChange = onToggle,
            colors =
            SwitchDefaults.colors(
                checkedThumbColor = Color.White,
                checkedTrackColor = accentColor,
                uncheckedThumbColor = textColor.copy(alpha = 0.6f),
                uncheckedTrackColor = cardBg,
            ),
        )
    }
}

@Composable
private fun SessionRestoreToggle(
    enabled: Boolean,
    onToggle: (Boolean) -> Unit,
    textColor: Color,
    accentColor: Color,
    cardBg: Color,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = stringResource(R.string.session_restore),
                style = MaterialTheme.typography.bodyLarge,
                color = textColor,
            )
            Text(
                text = stringResource(R.string.session_restore_desc),
                style = MaterialTheme.typography.bodySmall,
                color = textColor.copy(alpha = 0.6f),
            )
        }
        Switch(
            checked = enabled,
            onCheckedChange = onToggle,
            colors =
            SwitchDefaults.colors(
                checkedThumbColor = Color.White,
                checkedTrackColor = accentColor,
                uncheckedThumbColor = textColor.copy(alpha = 0.6f),
                uncheckedTrackColor = cardBg,
            ),
        )
    }
}

@Composable
private fun KeyboardModeSelector(
    selectedMode: String,
    onModeSelected: (String) -> Unit,
    textColor: Color,
    accentColor: Color,
    cardBg: Color,
) {
    val modes =
        listOf(
            Triple("secure", stringResource(R.string.keyboard_secure), stringResource(R.string.keyboard_secure_desc)),
            Triple("standard", stringResource(R.string.keyboard_standard), stringResource(R.string.keyboard_standard_desc)),
            Triple("raw", stringResource(R.string.keyboard_raw), stringResource(R.string.keyboard_raw_desc)),
        )
    Column(modifier = Modifier.fillMaxWidth()) {
        Text(
            text = stringResource(R.string.keyboard_mode),
            style = MaterialTheme.typography.bodyLarge,
            color = textColor,
        )
        Spacer(modifier = Modifier.height(4.dp))
        modes.forEach { (key, label, desc) ->
            Row(
                modifier =
                Modifier
                    .fillMaxWidth()
                    .clickable { onModeSelected(key) }
                    .padding(vertical = 4.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                RadioButton(
                    selected = selectedMode == key,
                    onClick = { onModeSelected(key) },
                    colors =
                    RadioButtonDefaults.colors(
                        selectedColor = accentColor,
                        unselectedColor = textColor.copy(alpha = 0.6f),
                    ),
                )
                Spacer(modifier = Modifier.width(8.dp))
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        text = label,
                        style = MaterialTheme.typography.bodyMedium,
                        color = textColor,
                    )
                    Text(
                        text = desc,
                        style = MaterialTheme.typography.bodySmall,
                        color = textColor.copy(alpha = 0.6f),
                    )
                }
            }
        }
    }
}

@Composable
private fun UsbSerialToggle(
    enabled: Boolean,
    onToggle: (Boolean) -> Unit,
    textColor: Color,
    accentColor: Color,
    cardBg: Color,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = stringResource(R.string.usb_serial),
                style = MaterialTheme.typography.bodyLarge,
                color = textColor,
            )
            Text(
                text = stringResource(R.string.usb_serial_desc),
                style = MaterialTheme.typography.bodySmall,
                color = textColor.copy(alpha = 0.6f),
            )
        }
        Switch(
            checked = enabled,
            onCheckedChange = onToggle,
            colors =
            SwitchDefaults.colors(
                checkedThumbColor = Color.White,
                checkedTrackColor = accentColor,
                uncheckedThumbColor = textColor.copy(alpha = 0.6f),
                uncheckedTrackColor = cardBg,
            ),
        )
    }
}

@Composable
private fun McpServerToggle(
    enabled: Boolean,
    onToggle: (Boolean) -> Unit,
    textColor: Color,
    accentColor: Color,
    cardBg: Color,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = stringResource(R.string.mcp_server),
                style = MaterialTheme.typography.bodyLarge,
                color = textColor,
            )
            Text(
                text = stringResource(R.string.mcp_server_desc),
                style = MaterialTheme.typography.bodySmall,
                color = textColor.copy(alpha = 0.6f),
            )
        }
        Switch(
            checked = enabled,
            onCheckedChange = onToggle,
            colors =
            SwitchDefaults.colors(
                checkedThumbColor = Color.White,
                checkedTrackColor = accentColor,
                uncheckedThumbColor = textColor.copy(alpha = 0.6f),
                uncheckedTrackColor = cardBg,
            ),
        )
    }
}

@Composable
private fun VolumeKeyMapToggle(
    enabled: Boolean,
    onToggle: (Boolean) -> Unit,
    textColor: Color,
    accentColor: Color,
    cardBg: Color,
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = stringResource(R.string.volume_key_map),
                style = MaterialTheme.typography.bodyLarge,
                color = textColor,
            )
            Text(
                text = stringResource(R.string.volume_key_map_desc),
                style = MaterialTheme.typography.bodySmall,
                color = textColor.copy(alpha = 0.6f),
            )
        }
        Switch(
            checked = enabled,
            onCheckedChange = onToggle,
            colors =
            SwitchDefaults.colors(
                checkedThumbColor = Color.White,
                checkedTrackColor = accentColor,
                uncheckedThumbColor = textColor.copy(alpha = 0.6f),
                uncheckedTrackColor = cardBg,
            ),
        )
    }
}

@Composable
private fun ClearAppDataSection(textColor: Color) {
    val context = LocalContext.current
    var showConfirmDialog by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()

    if (showConfirmDialog) {
        AlertDialog(
            onDismissRequest = { showConfirmDialog = false },
            title = { Text(stringResource(R.string.clear_app_data)) },
            text = { Text(stringResource(R.string.clear_app_data_confirm)) },
            confirmButton = {
                TextButton(
                    onClick = {
                        showConfirmDialog = false
                        scope.launch(Dispatchers.IO) {
                            try {
                                context.getDir("prefs", Context.MODE_PRIVATE).deleteRecursively()
                                context.getDir("sessions", Context.MODE_PRIVATE).deleteRecursively()
                                context.getDir("logs", Context.MODE_PRIVATE).deleteRecursively()
                                context.getDir("logs_root", Context.MODE_PRIVATE).deleteRecursively()
                                context.getDir("bin", Context.MODE_PRIVATE).deleteRecursively()
                                context.cacheDir.listFiles()?.forEach { it.delete() }
                            } catch (exception: Exception) {
                                Log.e("ClearAppData", "Failed to clear app data", exception)
                            }
                        }
                    },
                    colors = ButtonDefaults.textButtonColors(contentColor = Color.Red),
                ) {
                    Text(stringResource(R.string.clear_app_data_action))
                }
            },
            dismissButton = {
                TextButton(onClick = { showConfirmDialog = false }) {
                    Text(stringResource(R.string.cancel))
                }
            },
        )
    }

    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = stringResource(R.string.clear_app_data),
                style = MaterialTheme.typography.bodyLarge,
                color = textColor,
            )
            Text(
                text = stringResource(R.string.clear_app_data_desc),
                style = MaterialTheme.typography.bodySmall,
                color = textColor.copy(alpha = 0.6f),
            )
        }
        TextButton(onClick = { showConfirmDialog = true }) {
            Text(
                text = stringResource(R.string.clear_app_data_action),
                color = Color.Red,
            )
        }
    }
}
