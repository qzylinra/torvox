package io.torvox.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

data class ModifierKey(
    val label: String,
    val vtSequence: String,
    val isToggle: Boolean = false,
)

val defaultModifierKeys =
    listOf(
        ModifierKey("Esc", "\u001b"),
        ModifierKey("Tab", "\t"),
        ModifierKey("Ctrl", "", isToggle = true),
        ModifierKey("Alt", "", isToggle = true),
        ModifierKey("\u2190", "\u001b[D"),
        ModifierKey("\u2191", "\u001b[A"),
        ModifierKey("\u2193", "\u001b[B"),
        ModifierKey("\u2192", "\u001b[C"),
        ModifierKey("Home", "\u001b[H"),
        ModifierKey("End", "\u001b[F"),
        ModifierKey("PgUp", "\u001b[5~"),
        ModifierKey("PgDn", "\u001b[6~"),
        ModifierKey("Ins", "\u001b[2~"),
        ModifierKey("Del", "\u001b[3~"),
    )

@Composable
fun ModifierBar(
    onKeySend: (String) -> Unit,
    modifier: Modifier = Modifier,
    keys: List<ModifierKey> = defaultModifierKeys,
) {
    val scrollState = rememberScrollState()
    val activeToggles = remember { mutableStateMapOf<String, Boolean>() }

    Row(
        modifier =
            modifier
                .fillMaxWidth()
                .height(44.dp)
                .background(Color(0xFF1E1E1E))
                .horizontalScroll(scrollState)
                .padding(horizontal = 4.dp),
        horizontalArrangement = Arrangement.spacedBy(4.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        keys.forEach { key ->
            val isActive = activeToggles[key.label] == true
            val bgColor =
                when {
                    key.label == "Ctrl" && isActive -> Color(0xFF4CAF50)
                    key.label == "Alt" && isActive -> Color(0xFF2196F3)
                    else -> Color(0xFF333333)
                }
            val textColor =
                when {
                    key.label == "Ctrl" && isActive -> Color.White
                    key.label == "Alt" && isActive -> Color.White
                    else -> Color(0xFFCCCCCC)
                }

            Box(
                modifier =
                    Modifier
                        .height(36.dp)
                        .background(bgColor, MaterialTheme.shapes.small)
                        .pointerInput(key.label) {
                            detectTapGestures(
                                onTap = {
                                    if (key.isToggle) {
                                        val current = activeToggles[key.label] == true
                                        activeToggles[key.label] = !current
                                    } else {
                                        val prefix =
                                            buildString {
                                                if (activeToggles["Alt"] == true) append("\u001b")
                                            }
                                        onKeySend(prefix + key.vtSequence)
                                    }
                                },
                                onDoubleTap = {
                                    if (!key.isToggle) {
                                        onKeySend(key.vtSequence)
                                        activeToggles["Ctrl"] = false
                                        activeToggles["Alt"] = false
                                    }
                                },
                            )
                        }.padding(horizontal = 12.dp, vertical = 6.dp),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    text = key.label,
                    color = textColor,
                    fontSize = 14.sp,
                    fontWeight = if (isActive) FontWeight.Bold else FontWeight.Normal,
                )
            }
        }
    }
}
