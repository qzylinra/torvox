package io.torvox

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.os.Build
import android.os.Bundle
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.Box
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.testTagsAsResourceId
import androidx.hilt.navigation.compose.hiltViewModel
import dagger.hilt.android.AndroidEntryPoint
import io.torvox.runtime.TorvoxRuntime
import io.torvox.ui.SettingsScreen
import io.torvox.ui.TerminalScreen
import javax.inject.Inject

@AndroidEntryPoint
class MainActivity : ComponentActivity() {
    @Inject
    lateinit var torvoxRuntime: TorvoxRuntime

    private val terminalDumpReceiver =
        object : BroadcastReceiver() {
            override fun onReceive(
                context: Context,
                intent: Intent,
            ) {
                Thread {
                    try {
                        val bridge = torvoxRuntime.bridge()
                        val text =
                            if (bridge != null) {
                                bridge.getTerminalText() ?: "(empty)"
                            } else {
                                "(no active session)"
                            }
                        val file = java.io.File(context.filesDir, "torvox_terminal.txt")
                        file.writeText(text)
                        Log.d("Torvox", "Terminal dump: ${file.absolutePath} (${text.length} chars)")
                    } catch (e: Exception) {
                        Log.e("Torvox", "Terminal dump failed", e)
                    }
                }.apply {
                    isDaemon = true
                    start()
                }
            }
        }

    private val inputReceiver =
        object : BroadcastReceiver() {
            override fun onReceive(
                context: Context,
                intent: Intent,
            ) {
                val text = intent.getStringExtra("text") ?: return
                Thread {
                    try {
                        Log.d("Torvox", "Input received: '$text' (len=${text.length})")
                        val processed =
                            text
                                .replace("\\n", "\n")
                                .replace("\\r", "\r")
                                .replace("\\t", "\t")
                        val withNewline = processed + "\n"
                        val data = withNewline.byteInputStream().readBytes()
                        torvoxRuntime.writeToPty(data)
                        Log.d("Torvox", "Input sent: ${data.size} bytes")
                    } catch (e: Exception) {
                        Log.e("Torvox", "Input failed", e)
                    }
                }.apply {
                    isDaemon = true
                    start()
                }
            }
        }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        registerReceiver(
            terminalDumpReceiver,
            IntentFilter("io.torvox.DUMP_TERMINAL"),
            Context.RECEIVER_EXPORTED,
        )
        registerReceiver(
            inputReceiver,
            IntentFilter("io.torvox.INPUT"),
            Context.RECEIVER_EXPORTED,
        )
        setContent {
            TorvoxNavHost()
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        try {
            unregisterReceiver(terminalDumpReceiver)
        } catch (_: Exception) {
        }
        try {
            unregisterReceiver(inputReceiver)
        } catch (_: Exception) {
        }
    }
}

@Composable
private fun TorvoxNavHost() {
    val viewModel: TerminalViewModel = hiltViewModel()
    var showSettings by remember { mutableStateOf(false) }
    val materialYouEnabled by viewModel.materialYouEnabled.collectAsState()
    val isDarkTheme = androidx.compose.foundation.isSystemInDarkTheme()

    val colorScheme =
        when {
            materialYouEnabled && Build.VERSION.SDK_INT >= Build.VERSION_CODES.S -> {
                val context = LocalContext.current
                if (isDarkTheme) dynamicDarkColorScheme(context) else dynamicLightColorScheme(context)
            }

            isDarkTheme -> {
                darkColorScheme()
            }

            else -> {
                lightColorScheme()
            }
        }

    Box(Modifier.semantics { testTagsAsResourceId = true }) {
        MaterialTheme(colorScheme = colorScheme) {
            if (showSettings) {
                SettingsScreen(
                    viewModel = viewModel,
                    onBack = { showSettings = false },
                )
            } else {
                TerminalScreen(
                    viewModel = viewModel,
                    onSettings = { showSettings = true },
                )
            }
        }
    }
}
