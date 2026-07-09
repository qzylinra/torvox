package io.torvox

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.graphics.Color
import android.graphics.PixelFormat
import android.graphics.drawable.ColorDrawable
import android.os.Build
import android.os.Bundle
import android.util.Log
import android.view.KeyEvent
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.viewModels
import androidx.compose.foundation.layout.Box
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.testTagsAsResourceId
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.lifecycleScope
import dagger.hilt.android.AndroidEntryPoint
import io.torvox.runtime.TorvoxRuntime
import io.torvox.ui.SettingsScreen
import io.torvox.ui.TerminalScreen
import kotlinx.coroutines.launch
import java.io.BufferedWriter
import java.io.File
import java.io.FileWriter
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import javax.inject.Inject

@AndroidEntryPoint
class MainActivity : ComponentActivity() {
    companion object {
        private const val TAG = "MainActivity"
        private const val LOGCAT_RETRY_DELAY_MS = 5_000
    }

    @Inject
    lateinit var torvoxRuntime: TorvoxRuntime

    private var logFile: File? = null
    private var logWriter: BufferedWriter? = null
    private val logcatThread =
        Thread({
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
                Log.w(
                    "Torvox",
                    "Logcat capture not supported on Android 11+ — READ_LOGS permission unavailable; this path is expected to fail",
                )
                return@Thread
            }
            while (!Thread.currentThread().isInterrupted) {
                try {
                    val process = Runtime.getRuntime().exec(arrayOf("logcat", "-v", "time", "*:D"))
                    val reader = process.inputStream.bufferedReader()
                    var line: String?
                    while (reader.readLine().also { line = it } != null) {
                        val currentLine = line ?: continue
                        @Suppress("ComplexCondition")
                        if (currentLine.contains(
                                "Torvox",
                            ) || currentLine.contains("TerminalSurface") || currentLine.contains("TorvoxRuntime") ||
                            currentLine.contains("AndroidRuntime")
                        ) {
                            val timestamp = SimpleDateFormat("HH:mm:ss.SSS", Locale.US).format(Date())
                            synchronized(logLock) {
                                logWriter?.write("$timestamp $currentLine\n")
                                logWriter?.flush()
                            }
                        }
                    }
                    Log.w("Torvox", "Logcat stream ended, restarting in 5s")
                    Thread.sleep(LOGCAT_RETRY_DELAY_MS.toLong())
                } catch (e: InterruptedException) {
                    Thread.currentThread().interrupt()
                    break
                } catch (e: Exception) {
                    Log.e("Torvox", "Logcat capture failed, retrying in 5s: ${e.message}")
                    try {
                        Thread.sleep(LOGCAT_RETRY_DELAY_MS.toLong())
                    } catch (_: InterruptedException) {
                        Thread.currentThread().interrupt()
                        break
                    }
                }
            }
        }, "TorvoxFileLog").apply { isDaemon = true }

    private val logLock = Any()

    private fun initFileLogging() {
        try {
            val logDir = getDir("logs", Context.MODE_PRIVATE)
            logDir.mkdirs()
            val timestamp = SimpleDateFormat("yyyyMMdd_HHmmss", Locale.US).format(Date())
            val logFilePath = File(logDir, "torvox_$timestamp.log")
            logFile = logFilePath
            logWriter = BufferedWriter(FileWriter(logFilePath, true), 8192)
            logcatThread.start()
            Log.d("Torvox", "File logging: ${logFilePath.absolutePath}")
        } catch (exception: Exception) {
            Log.e("Torvox", "Failed to init file logging", exception)
        }
    }

    private fun stopFileLogging() {
        try {
            logWriter?.close()
            logWriter = null
        } catch (exception: Exception) {
            Log.w(TAG, "stopFileLogging failed", exception)
        }
    }

    private fun tryUnregisterReceiver(
        receiver: BroadcastReceiver,
        name: String,
    ) {
        try {
            unregisterReceiver(receiver)
        } catch (exception: IllegalArgumentException) {
            Log.w(TAG, "$name not registered", exception)
        }
    }

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
                        val file = java.io.File(context.cacheDir, "torvox_terminal.txt")
                        file.writeText(text)
                        Log.d("Torvox", "Terminal dump: ${file.absolutePath} (${text.length} chars)")
                    } catch (exception: Exception) {
                        Log.e("Torvox", "Terminal dump failed", exception)
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
                terminalViewModel?.clearSelection()
                Thread {
                    try {
                        Log.d("Torvox", "Input received: '$text' (len=${text.length})")
                        val processed =
                            text
                                .replace("\\n", "\n")
                                .replace("\\r", "\r")
                                .replace("\\t", "\t")
                        val data = (processed + "\n").byteInputStream().readBytes()
                        torvoxRuntime.writeToPty(data)
                        Log.d("Torvox", "Input sent: ${data.size} bytes")
                    } catch (exception: Exception) {
                        Log.e("Torvox", "Input failed", exception)
                    }
                }.apply {
                    isDaemon = true
                    start()
                }
            }
        }

    private var terminalViewModel: io.torvox.TerminalViewModel? = null

    private var previousNightMode: Int? = null

    private val terminalVm: io.torvox.TerminalViewModel by viewModels()

    private val selectAllReceiver =
        object : BroadcastReceiver() {
            override fun onReceive(
                context: Context,
                intent: Intent,
            ) {
                val viewModel = terminalViewModel ?: return
                viewModel.selectAll()
                Log.d("Torvox", "selectAll called via broadcast, active=${viewModel.state.value.selection.active}")
            }
        }

    private val partialSelectReceiver =
        object : BroadcastReceiver() {
            override fun onReceive(
                context: Context,
                intent: Intent,
            ) {
                val viewModel = terminalViewModel ?: return
                val startRow = intent.getIntExtra("startRow", 0)
                val startCol = intent.getIntExtra("startCol", 0)
                val endRow = intent.getIntExtra("endRow", 2)
                val endCol = intent.getIntExtra("endCol", 10)
                viewModel.startSelection(startRow, startCol)
                viewModel.updateSelection(endRow, endCol)
                viewModel.endSelection()
                Log.d("Torvox", "partialSelect: ($startRow,$startCol)->($endRow,$endCol)")
            }
        }

    private val showPasteReceiver =
        object : BroadcastReceiver() {
            override fun onReceive(
                context: Context,
                intent: Intent,
            ) {
                val viewModel = terminalViewModel ?: return
                val row = intent.getIntExtra("row", 10)
                val col = intent.getIntExtra("col", 0)
                viewModel.showPastePopup(row, col)
                Log.d("Torvox", "showPaste: row=$row col=$col")
            }
        }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        previousNightMode =
            resources.configuration.uiMode and android.content.res.Configuration.UI_MODE_NIGHT_MASK
        androidx.core.view.WindowCompat
            .setDecorFitsSystemWindows(window, false)
        window.setBackgroundDrawable(ColorDrawable(Color.TRANSPARENT))
        window.setFormat(PixelFormat.TRANSPARENT)
        initFileLogging()
        registerReceiver(
            terminalDumpReceiver,
            IntentFilter("io.torvox.DUMP_TERMINAL"),
            Context.RECEIVER_NOT_EXPORTED,
        )
        registerReceiver(
            inputReceiver,
            IntentFilter("io.torvox.INPUT"),
            Context.RECEIVER_NOT_EXPORTED,
        )
        registerReceiver(
            selectAllReceiver,
            IntentFilter("io.torvox.SELECT_ALL"),
            Context.RECEIVER_NOT_EXPORTED,
        )
        registerReceiver(
            partialSelectReceiver,
            IntentFilter("io.torvox.PARTIAL_SELECT"),
            Context.RECEIVER_NOT_EXPORTED,
        )
        registerReceiver(
            showPasteReceiver,
            IntentFilter("io.torvox.SHOW_PASTE"),
            Context.RECEIVER_NOT_EXPORTED,
        )
        io.torvox.service.TerminalForegroundService
            .start(this)
        terminalViewModel = terminalVm
        setContent {
            TorvoxNavHost(viewModelReady = { terminalViewModel = it })
        }
    }

    override fun onDestroy() {
        lifecycleScope.launch {
            try {
                terminalViewModel?.runtime?.saveAllSessions()
            } catch (exception: Exception) {
                Log.e(TAG, "Failed to save sessions during destroy", exception)
            }
        }
        super.onDestroy()
        stopFileLogging()
        tryUnregisterReceiver(terminalDumpReceiver, "terminalDumpReceiver")
        tryUnregisterReceiver(inputReceiver, "inputReceiver")
        tryUnregisterReceiver(selectAllReceiver, "selectAllReceiver")
        tryUnregisterReceiver(partialSelectReceiver, "partialSelectReceiver")
        tryUnregisterReceiver(showPasteReceiver, "showPasteReceiver")
    }

    override fun onConfigurationChanged(newConfig: android.content.res.Configuration) {
        super.onConfigurationChanged(newConfig)
        val currentNightMode =
            newConfig.uiMode and android.content.res.Configuration.UI_MODE_NIGHT_MASK
        if (currentNightMode != previousNightMode) {
            lifecycleScope.launch(kotlinx.coroutines.Dispatchers.IO) {
                torvoxRuntime.applySettings()
            }
        }
        previousNightMode = currentNightMode
    }

    override fun dispatchKeyEvent(event: KeyEvent): Boolean {
        val handled = terminalViewModel?.handleLayoutAwareHardwareKey(event) ?: false
        if (handled) {
            Log.d(TAG, "dispatchKeyEvent: consumed physical-key layout-aware char")
            return true
        }
        return super.dispatchKeyEvent(event)
    }

    @Deprecated("Use View.OnKeyListener pattern")
    override fun onKeyDown(
        keyCode: Int,
        event: KeyEvent?,
    ): Boolean {
        if (event != null && isVolumeKeyMappingEnabled()) {
            when (keyCode) {
                KeyEvent.KEYCODE_VOLUME_UP -> {
                    val viewModel = terminalViewModel
                    if (viewModel != null) {
                        val ctrlLocked = viewModel.state.value.ctrlState == io.torvox.ui.ModifierState.Locked
                        val altLocked = viewModel.state.value.altState == io.torvox.ui.ModifierState.Locked
                        if (!ctrlLocked && !altLocked) {
                            viewModel.setModifierKeys(listOf(io.torvox.ui.ModifierKey("ctrl", "CTRL", ctrl = true)))
                        }
                        return true
                    }
                }

                KeyEvent.KEYCODE_VOLUME_DOWN -> {
                    val viewModel = terminalViewModel
                    if (viewModel != null) {
                        val ctrlLocked = viewModel.state.value.ctrlState == io.torvox.ui.ModifierState.Locked
                        if (!ctrlLocked) {
                            viewModel.setModifierKeys(listOf(io.torvox.ui.ModifierKey("alt", "ALT", alt = true)))
                        }
                        return true
                    }
                }
            }
        }
        return super.onKeyDown(keyCode, event)
    }

    @Deprecated("Use View.OnKeyListener pattern")
    override fun onKeyUp(
        keyCode: Int,
        event: KeyEvent?,
    ): Boolean {
        if (event != null && isVolumeKeyMappingEnabled()) {
            when (keyCode) {
                KeyEvent.KEYCODE_VOLUME_UP, KeyEvent.KEYCODE_VOLUME_DOWN -> {
                    val viewModel = terminalViewModel
                    if (viewModel != null) {
                        val ctrlOnce = viewModel.state.value.ctrlState == io.torvox.ui.ModifierState.Once
                        val altOnce = viewModel.state.value.altState == io.torvox.ui.ModifierState.Once
                        if (ctrlOnce || altOnce) {
                            viewModel.consumeOneShotModifiers()
                        }
                    }
                    return true
                }
            }
        }
        return super.onKeyUp(keyCode, event)
    }

    private fun isVolumeKeyMappingEnabled(): Boolean {
        val viewModel = terminalViewModel
        return viewModel?.volumeKeyMap?.value == true
    }
}

@Composable
private fun TorvoxNavHost(viewModelReady: (TerminalViewModel) -> Unit = {}) {
    val viewModel: TerminalViewModel = hiltViewModel()
    LaunchedEffect(viewModel) { viewModelReady(viewModel) }
    var showSettings by remember { mutableStateOf(false) }
    val appThemeMode by viewModel.appThemeMode.collectAsState()
    val isDarkTheme = androidx.compose.foundation.isSystemInDarkTheme()
    val context = LocalContext.current

    val forceDark =
        when (appThemeMode) {
            "night" -> true
            "day" -> false
            else -> isDarkTheme
        }

    val colorScheme =
        when {
            appThemeMode == "follow_system" && Build.VERSION.SDK_INT >= Build.VERSION_CODES.S -> {
                if (isDarkTheme) dynamicDarkColorScheme(context) else dynamicLightColorScheme(context)
            }

            forceDark -> {
                darkColorScheme()
            }

            else -> {
                lightColorScheme()
            }
        }

    Box(Modifier.semantics { testTagsAsResourceId = true }) {
        MaterialTheme(colorScheme = colorScheme) {
            TerminalScreen(
                viewModel = viewModel,
                onSettings = { showSettings = true },
                isOverlayVisible = showSettings,
            )
            if (showSettings) {
                SettingsScreen(
                    viewModel = viewModel,
                    onBack = { showSettings = false },
                )
            }
        }
    }
}
