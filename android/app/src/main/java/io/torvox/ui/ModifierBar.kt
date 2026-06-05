package io.torvox.ui

import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

data class ModifierKey(
    val label: String,
    val vtSequence: String,
    val isToggle: Boolean = false,
    val weight: Float = 1f,
    val isSessionButton: Boolean = false,
)

val defaultModifierKeys: List<ModifierKey> = emptyList()

private val keyFg = Color(0xFFBBBBBB)
private val ctrlActiveColor = Color(0xFF4CAF50)
private val altActiveColor = Color(0xFF2196F3)

@Composable
fun ModifierBar(
    onKeySend: (String) -> Unit,
    modifier: Modifier = Modifier,
    onToggleChanged: ((String, Boolean) -> Unit)? = null,
    onSessionDrawer: (() -> Unit)? = null,
) {
    val activeToggles = remember { mutableStateMapOf<String, Boolean>() }

    Column(
        modifier = modifier.fillMaxWidth().padding(horizontal = 2.dp, vertical = 2.dp),
        verticalArrangement = Arrangement.spacedBy(2.dp),
    ) {
        // Row 1: Function keys (ESC TAB CTRL ALT)
        Row(
            modifier = Modifier.fillMaxWidth().height(36.dp),
            horizontalArrangement = Arrangement.SpaceEvenly,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            ModifierTextButton("ESC", "\u001b", keyFg, activeToggles, onKeySend, onToggleChanged, onSessionDrawer)
            ModifierTextButton("TAB", "\t", keyFg, activeToggles, onKeySend, onToggleChanged, onSessionDrawer)
            ModifierTextButton("CTRL", "", keyFg, activeToggles, onKeySend, onToggleChanged, onSessionDrawer, ctrlActiveColor)
            ModifierTextButton("ALT", "", keyFg, activeToggles, onKeySend, onToggleChanged, onSessionDrawer, altActiveColor)
        }

        // Row 2: Symbol keys ( / - ~ | )
        Row(
            modifier = Modifier.fillMaxWidth().height(36.dp),
            horizontalArrangement = Arrangement.SpaceEvenly,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Spacer(modifier = Modifier.weight(1f))
            ModifierTextButton("/", "/", keyFg, activeToggles, onKeySend, onToggleChanged, onSessionDrawer)
            ModifierTextButton("-", "-", keyFg, activeToggles, onKeySend, onToggleChanged, onSessionDrawer)
            ModifierTextButton("~", "~", keyFg, activeToggles, onKeySend, onToggleChanged, onSessionDrawer)
            ModifierTextButton("|", "|", keyFg, activeToggles, onKeySend, onToggleChanged, onSessionDrawer)
            Spacer(modifier = Modifier.weight(1f))
        }

        // Row 3: Directional pad - UP arrow centered
        Row(
            modifier = Modifier.fillMaxWidth().height(36.dp),
            horizontalArrangement = Arrangement.Center,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            ArrowButton(modifier = Modifier.weight(1f), "\u2191", "\u001b[A", "Up", keyFg, onKeySend)
        }

        // Row 4: LEFT DOWN RIGHT ENT MENU
        Row(
            modifier = Modifier.fillMaxWidth().height(36.dp),
            horizontalArrangement = Arrangement.SpaceEvenly,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            ArrowButton(modifier = Modifier.weight(1f), "\u2190", "\u001b[D", "Left", keyFg, onKeySend)
            ArrowButton(modifier = Modifier.weight(1f), "\u2193", "\u001b[B", "Down", keyFg, onKeySend)
            ArrowButton(modifier = Modifier.weight(1f), "\u2192", "\u001b[C", "Right", keyFg, onKeySend)
            Box(modifier = Modifier.weight(1f), contentAlignment = Alignment.Center) {
                Text(
                    text = "ENT",
                    modifier =
                        Modifier
                            .clickable(
                                interactionSource = remember { MutableInteractionSource() },
                                indication = null,
                            ) { onKeySend("\r") }
                            .testTag("Key_ENT"),
                    color = keyFg,
                    fontSize = 14.sp,
                    textAlign = TextAlign.Center,
                )
            }
            Box(modifier = Modifier.weight(1f), contentAlignment = Alignment.Center) {
                Text(
                    text = "\u2630",
                    modifier =
                        Modifier
                            .clickable(
                                interactionSource = remember { MutableInteractionSource() },
                                indication = null,
                            ) { onSessionDrawer?.invoke() }
                            .testTag("Key_Session"),
                    color = keyFg,
                    fontSize = 20.sp,
                    textAlign = TextAlign.Center,
                )
            }
        }
    }
}

@Composable
private fun ArrowButton(
    modifier: Modifier = Modifier,
    text: String,
    sequence: String,
    tag: String,
    color: Color,
    onKeySend: (String) -> Unit,
) {
    Box(modifier = modifier, contentAlignment = Alignment.Center) {
        Text(
            text = text,
            modifier =
                Modifier
                    .clickable(
                        interactionSource = remember { MutableInteractionSource() },
                        indication = null,
                    ) { onKeySend(sequence) }
                    .testTag("Key_$tag"),
            color = color,
            fontSize = 22.sp,
            textAlign = TextAlign.Center,
        )
    }
}

@Composable
private fun RowScope.ModifierTextButton(
    label: String,
    vtSequence: String,
    defaultColor: Color,
    activeToggles: MutableMap<String, Boolean>,
    onKeySend: (String) -> Unit,
    onToggleChanged: ((String, Boolean) -> Unit)?,
    onSessionDrawer: (() -> Unit)?,
    toggleActiveColor: Color? = null,
) {
    val isToggle = vtSequence.isEmpty()
    val isActive = activeToggles[label] == true
    val textColor =
        when {
            label == "CTRL" && isActive -> toggleActiveColor ?: defaultColor
            label == "ALT" && isActive -> toggleActiveColor ?: defaultColor
            else -> defaultColor
        }

    Box(
        modifier = Modifier.weight(1f),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = label,
            modifier =
                Modifier
                    .clickable(
                        interactionSource = remember { MutableInteractionSource() },
                        indication = null,
                    ) {
                        when {
                            isToggle -> {
                                val current = activeToggles[label] == true
                                val newValue = !current
                                activeToggles[label] = newValue
                                onToggleChanged?.invoke(label, newValue)
                            }

                            else -> {
                                onKeySend(vtSequence)
                            }
                        }
                    }.testTag("Key_$label"),
            color = textColor,
            fontSize = 13.sp,
            textAlign = TextAlign.Center,
        )
    }
}
