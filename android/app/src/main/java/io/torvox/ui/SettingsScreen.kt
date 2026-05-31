package io.torvox.ui

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
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import io.torvox.TerminalViewModel
import io.torvox.ui.theme.BuiltInThemes

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen(
    viewModel: TerminalViewModel,
    onBack: () -> Unit,
) {
    val fontSize by viewModel.fontSize.collectAsState()
    val fontFamily by viewModel.fontFamily.collectAsState()
    val themeName by viewModel.themeName.collectAsState()
    val selectedShell by viewModel.shell.collectAsState()
    val scrollbackLines by viewModel.scrollbackLines.collectAsState()
    val touchBehavior by viewModel.touchBehavior.collectAsState()

    Column(modifier = Modifier.fillMaxSize()) {
        TopAppBar(
            title = { Text("Settings") },
            navigationIcon = {
                IconButton(onClick = onBack) {
                    Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "Back")
                }
            },
            colors =
                TopAppBarDefaults.topAppBarColors(
                    containerColor = MaterialTheme.colorScheme.surface,
                ),
        )

        LazyColumn(
            modifier =
                Modifier
                    .fillMaxSize()
                    .padding(horizontal = 16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            item {
                SectionHeader("Appearance")
                FontSizeSlider(
                    value = fontSize,
                    onValueChange = { viewModel.setFontSize(it) },
                )
                FontFamilySelector(
                    selectedFamily = fontFamily,
                    onFamilySelected = { viewModel.setFontFamily(it) },
                )
            }

            item {
                ThemeSelector(
                    selectedTheme = themeName,
                    onThemeSelected = { viewModel.setThemeName(it) },
                )
            }

            item {
                Spacer(modifier = Modifier.height(8.dp))
                SectionHeader("Terminal")
                ShellSelector(
                    selectedShell = selectedShell,
                    onShellSelected = { viewModel.setShell(it) },
                )
            }

            item {
                ScrollbackSlider(
                    value = scrollbackLines.toFloat(),
                    onValueChange = { viewModel.setScrollbackLines(it.toInt()) },
                )
            }

            item {
                Spacer(modifier = Modifier.height(8.dp))
                SectionHeader("Input")
                TouchBehaviorSelector(
                    selectedBehavior = touchBehavior,
                    onBehaviorSelected = { viewModel.setTouchBehavior(it) },
                )
            }

            item {
                Spacer(modifier = Modifier.height(8.dp))
                SectionHeader("Session")
                SessionActions(
                    isRunning =
                        viewModel.state
                            .collectAsState()
                            .value.isRunning,
                    onCreate = { viewModel.createSession() },
                    onClose = { viewModel.closeSession() },
                )
            }

            item {
                Spacer(modifier = Modifier.height(16.dp))
            }
        }
    }
}

@Composable
private fun SectionHeader(title: String) {
    Text(
        text = title,
        style = MaterialTheme.typography.titleSmall,
        fontWeight = FontWeight.Bold,
        color = MaterialTheme.colorScheme.primary,
        modifier = Modifier.padding(vertical = 8.dp),
    )
}

@Composable
private fun FontSizeSlider(
    value: Float,
    onValueChange: (Float) -> Unit,
) {
    Column {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
        ) {
            Text("Font Size", style = MaterialTheme.typography.bodyLarge)
            Text(
                text = "%.0f sp".format(value),
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        Slider(
            value = value,
            onValueChange = onValueChange,
            valueRange = 8f..32f,
            steps = 23,
            colors =
                SliderDefaults.colors(
                    thumbColor = MaterialTheme.colorScheme.primary,
                ),
        )
    }
}

@Composable
private fun FontFamilySelector(
    selectedFamily: String,
    onFamilySelected: (String) -> Unit,
) {
    val fontFamilies =
        listOf(
            "JetBrains Mono Nerd Font",
            "Fira Code",
            "Source Code Pro",
            "Cascadia Code",
            "Hack",
            "MesloLGS NF",
            "Iosevka",
            "DejaVu Sans Mono",
            "Droid Sans Mono",
            "monospace",
        )

    Spacer(modifier = Modifier.height(8.dp))
    Text("Font Family", style = MaterialTheme.typography.bodyLarge)
    Spacer(modifier = Modifier.height(4.dp))
    fontFamilies.forEach { family ->
        Row(
            modifier =
                Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(8.dp))
                    .clickable { onFamilySelected(family) }
                    .padding(vertical = 10.dp, horizontal = 8.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Box(
                modifier =
                    Modifier
                        .size(16.dp)
                        .clip(RoundedCornerShape(50))
                        .background(
                            if (selectedFamily == family) {
                                MaterialTheme.colorScheme.primary
                            } else {
                                MaterialTheme.colorScheme.outline
                            },
                        ),
            )
            Spacer(modifier = Modifier.width(12.dp))
            Text(family, style = MaterialTheme.typography.bodyLarge)
        }
    }
}

@Composable
private fun ThemeSelector(
    selectedTheme: String,
    onThemeSelected: (String) -> Unit,
) {
    Column {
        Text("Theme", style = MaterialTheme.typography.bodyLarge)
        Spacer(modifier = Modifier.height(8.dp))
        LazyRow(
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            items(BuiltInThemes.all) { theme ->
                ThemePreview(
                    theme = theme,
                    isSelected = theme.name == selectedTheme,
                    onClick = { onThemeSelected(theme.name) },
                )
            }
        }
    }
}

@Composable
private fun ThemePreview(
    theme: io.torvox.ui.theme.TerminalTheme,
    isSelected: Boolean,
    onClick: () -> Unit,
) {
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        modifier =
            Modifier
                .width(80.dp)
                .clip(RoundedCornerShape(8.dp))
                .background(theme.background)
                .clickable(onClick = onClick)
                .padding(8.dp),
    ) {
        Row(
            horizontalArrangement = Arrangement.spacedBy(2.dp),
        ) {
            theme.ansi.take(8).forEach { color ->
                Box(
                    modifier =
                        Modifier
                            .size(8.dp)
                            .clip(RoundedCornerShape(2.dp))
                            .background(color),
                )
            }
        }
        Spacer(modifier = Modifier.height(4.dp))
        Text(
            text = theme.name,
            style = MaterialTheme.typography.labelSmall,
            color = theme.foreground,
            maxLines = 1,
        )
    }
}

@Composable
private fun ShellSelector(
    selectedShell: String,
    onShellSelected: (String) -> Unit,
) {
    val shells =
        listOf(
            "/system/bin/sh" to "sh (default)",
            "/system/bin/bash" to "bash",
            "/system/bin/zsh" to "zsh",
            "/system/bin/fish" to "fish",
        )

    Column {
        shells.forEach { (path, label) ->
            Row(
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(8.dp))
                        .clickable { onShellSelected(path) }
                        .padding(vertical = 12.dp, horizontal = 8.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Box(
                    modifier =
                        Modifier
                            .size(16.dp)
                            .clip(RoundedCornerShape(50))
                            .background(
                                if (selectedShell == path) {
                                    MaterialTheme.colorScheme.primary
                                } else {
                                    MaterialTheme.colorScheme.outline
                                },
                            ),
                )
                Spacer(modifier = Modifier.width(12.dp))
                Text(label, style = MaterialTheme.typography.bodyLarge)
            }
        }
    }
}

@Composable
private fun SessionActions(
    isRunning: Boolean,
    onCreate: () -> Unit,
    onClose: () -> Unit,
) {
    Column {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .clip(RoundedCornerShape(8.dp))
                .clickable { onCreate() }
                .padding(vertical = 12.dp, horizontal = 8.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                "+ New Session",
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.primary,
            )
        }
        if (isRunning) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(8.dp))
                    .clickable { onClose() }
                    .padding(vertical = 12.dp, horizontal = 8.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    "Close Session",
                    style = MaterialTheme.typography.bodyLarge,
                    color = MaterialTheme.colorScheme.error,
                )
            }
        }
    }
}
@Composable
private fun ScrollbackSlider(
    value: Float,
    onValueChange: (Float) -> Unit,
) {
    Column {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
        ) {
            Text("Scrollback Lines", style = MaterialTheme.typography.bodyLarge)
            Text(
                text =
                    value.toInt().let {
                        if (it >= 1000) "${it / 1000}K" else "$it"
                    },
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        Slider(
            value = value,
            onValueChange = onValueChange,
            valueRange = 1000f..100000f,
            steps = 98,
            colors =
                SliderDefaults.colors(
                    thumbColor = MaterialTheme.colorScheme.primary,
                ),
        )
    }
}

@Composable
private fun TouchBehaviorSelector(
    selectedBehavior: String,
    onBehaviorSelected: (String) -> Unit,
) {
    val behaviors =
        listOf(
            "right_click" to "Right click (paste)",
            "middle_click" to "Middle click (paste)",
            "none" to "No action",
        )

    Column {
        behaviors.forEach { (value, label) ->
            Row(
                modifier =
                    Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(8.dp))
                        .clickable { onBehaviorSelected(value) }
                        .padding(vertical = 12.dp, horizontal = 8.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Box(
                    modifier =
                        Modifier
                            .size(16.dp)
                            .clip(RoundedCornerShape(50))
                            .background(
                                if (selectedBehavior == value) {
                                    MaterialTheme.colorScheme.primary
                                } else {
                                    MaterialTheme.colorScheme.outline
                                },
                            ),
                )
                Spacer(modifier = Modifier.width(12.dp))
                Text(label, style = MaterialTheme.typography.bodyLarge)
            }
        }
    }
}
