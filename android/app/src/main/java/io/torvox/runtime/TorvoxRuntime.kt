package io.torvox.runtime

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.media.ToneGenerator
import android.os.Handler
import android.os.Looper
import android.util.Log
import android.view.Surface
import dagger.hilt.android.qualifiers.ApplicationContext
import io.torvox.bridge.BridgeTheme
import io.torvox.bridge.Shell
import io.torvox.bridge.TerminalConfig
import io.torvox.bridge.TorvoxBridge
import io.torvox.bridge.createBridge
import io.torvox.settings.SettingsRepository
import io.torvox.ui.theme.BuiltInThemes
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.first
import java.util.concurrent.ConcurrentHashMap
import javax.inject.Inject
import javax.inject.Singleton

data class RuntimeState(
    val isRunning: Boolean = false,
    val title: String = "Torvox",
    val rows: Int = 24,
    val cols: Int = 80,
    val activeSessionId: Long = 0L,
    val sessionIds: List<Long> = emptyList(),
)

private class SessionEntry(
    val id: Long,
    var bridge: TorvoxBridge?,
    var renderThread: Thread?,
    @Volatile var running: Boolean,
    val savePath: String,
    @Volatile var blitCallback: (() -> Unit)? = null,
)

@Singleton
class TorvoxRuntime
@Inject
constructor(
    @ApplicationContext private val context: Context,
    private val settingsRepository: SettingsRepository,
) {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private val _state = MutableStateFlow(RuntimeState())
    val state: StateFlow<RuntimeState> = _state.asStateFlow()

    private val sessions = ConcurrentHashMap<Long, SessionEntry>()

    @Volatile private var lastWriteData: ByteArray? = null

    @Volatile private var lastWriteTime: Long = 0L

    private val renderGeneration =
        java.util.concurrent.atomic
            .AtomicInteger(0)

    @Volatile private var activeSessionId: Long = 0L

    @Volatile private var stopped = false

    @Volatile private var starting = false
    private val stopLock = Any()

    private val selectionState =
        java.util.concurrent.atomic.AtomicReference(
            Triple(Pair(0u, 0u), Pair(0u, 0u), false),
        )

    private fun sessionSavePath(id: Long): String {
        val dir = context.getDir("sessions", Context.MODE_PRIVATE)
        return java.io.File(dir, "session_$id.bin").absolutePath
    }

    private suspend fun buildConfig(
        rows: UInt = 24u,
        cols: UInt = 80u,
    ): TerminalConfig {
        val shellPath = settingsRepository.shell.first()
        val scrollbackLines = settingsRepository.scrollbackLines.first()
        val fontSizeTenths = computeFontSizeTenths()
        val themeName = resolveThemeName()
        val resolvedTheme = BuiltInThemes.byName(themeName)
        val shell = resolveShell(shellPath)
        val bridgeTheme = makeBridgeTheme(resolvedTheme)
        val prefixDir = "/data/data/com.termux/files/usr"
        val homeDir = "/data/data/com.termux/files/home"
        val bashFile = java.io.File("$prefixDir/bin/bash")
        val bashComplete =
            bashFile.exists() &&
                java.io.File("$prefixDir/lib").isDirectory &&
                java.io.File("$prefixDir/etc").isDirectory
        val effectivePrefix = if (bashComplete) prefixDir else ""
        val effectiveShell = if (bashComplete) Shell.Custom("$prefixDir/bin/bash") else shell
        val effectiveHome = if (bashComplete) homeDir else context.filesDir.absolutePath
        val effectivePath =
            if (bashComplete) {
                "$prefixDir/bin:/system/bin:/system/xbin"
            } else {
                System.getenv("PATH") ?: "/system/bin:/system/xbin"
            }
        return TerminalConfig(
            shell = effectiveShell,
            rows = rows,
            cols = cols,
            scrollbackLines = scrollbackLines.toUInt(),
            font_size_tenths = fontSizeTenths,
            theme = bridgeTheme,
            home = effectiveHome,
            user = System.getProperty("user.name", "shell"),
            path = effectivePath,
            workingDirectory = effectiveHome,
            prefix = effectivePrefix,
        )
    }

    private companion object {
        private const val LINE_HEIGHT_RATIO = 1.2f
        private const val TENTHS_PER_UNIT = 10
    }

    private suspend fun computeFontSizeTenths(): UInt {
        val userFontSize = settingsRepository.fontSize.first()
        val density = context.resources.displayMetrics.density
        val cellHeightPixels = userFontSize * density
        return (cellHeightPixels * TENTHS_PER_UNIT.toFloat() / LINE_HEIGHT_RATIO).toInt().toUInt()
    }

    private suspend fun resolveThemeName(): String {
        val themeMode = settingsRepository.themeMode.first()
        val dayTheme = settingsRepository.dayThemeName.first()
        val nightTheme = settingsRepository.nightThemeName.first()
        val singleTheme = settingsRepository.themeName.first()
        val systemDark =
            (
                context.resources.configuration.uiMode and
                    android.content.res.Configuration.UI_MODE_NIGHT_MASK
                ) ==
                android.content.res.Configuration.UI_MODE_NIGHT_YES
        return when (themeMode) {
            "day" -> dayTheme
            "night" -> nightTheme
            "fixed" -> singleTheme
            else -> if (systemDark) nightTheme else dayTheme
        }
    }

    private fun resolveShell(shellPath: String): Shell = if (shellPath == "/system/bin/sh" || shellPath.isEmpty()) {
        Shell.SystemDefault
    } else {
        Shell.Custom(shellPath)
    }

    private fun makeBridgeTheme(resolvedTheme: io.torvox.ui.theme.TerminalTheme): BridgeTheme {
        fun colorToInt(c: androidx.compose.ui.graphics.Color): Int = ((c.alpha * 255).toInt() shl 24) or ((c.red * 255).toInt() shl 16) or
            ((c.green * 255).toInt() shl 8) or (c.blue * 255).toInt()
        val bg = colorToInt(resolvedTheme.background)
        val fg = colorToInt(resolvedTheme.foreground)
        val cursor = colorToInt(resolvedTheme.cursor)
        val ansiInts = resolvedTheme.ansi.map(::colorToInt)
        return BridgeTheme(
            name = resolvedTheme.name,
            bg = bg,
            fg = fg,
            cursor = cursor,
            ansi0 = ansiInts[0],
            ansi1 = ansiInts[1],
            ansi2 = ansiInts[2],
            ansi3 = ansiInts[3],
            ansi4 = ansiInts[4],
            ansi5 = ansiInts[5],
            ansi6 = ansiInts[6],
            ansi7 = ansiInts[7],
            ansi8 = ansiInts[8],
            ansi9 = ansiInts[9],
            ansi10 = ansiInts[10],
            ansi11 = ansiInts[11],
            ansi12 = ansiInts[12],
            ansi13 = ansiInts[13],
            ansi14 = ansiInts[14],
            ansi15 = ansiInts[15],
        )
    }

    suspend fun start(
        surface: Surface?,
        width: Int,
        height: Int,
        blitCallback: (() -> Unit)? = null,
    ) {
        if (sessions.isNotEmpty() || starting) return
        starting = true
        stopped = false
        Log.d("TorvoxRuntime", "start() called: surface=$surface width=$width height=$height")
        LogcatFileWriter.write("TorvoxRuntime", "start() called: surface=$surface width=$width height=$height")
        val displayW = context.resources.displayMetrics.widthPixels
        val displayH = context.resources.displayMetrics.heightPixels
        val density = context.resources.displayMetrics.density
        Log.d(
            "TorvoxRuntime",
            "displayMetrics: w=$displayW h=$displayH density=$density",
        )

        if (width <= 0 || height <= 0) {
            Log.e("TorvoxRuntime", "start() called with non-positive dimensions, waiting for surfaceChanged")
            starting = false
            return
        }

        val minWidth = (displayW * 0.6f).toInt().coerceIn(300, 600)
        val minHeight = (displayH * 0.5f).toInt().coerceIn(250, 500)
        if (width < minWidth || height < minHeight) {
            Log.w(
                "TorvoxRuntime",
                "start() called with small surface ${width}x$height (display=${displayW}x$displayH min=${minWidth}x$minHeight), waiting for correct surfaceChanged",
            )
            starting = false
            return
        }

        var windowPointer = 0L
        if (surface != null) {
            windowPointer = getNativeWindowPtr(surface)
            Log.d("TorvoxRuntime", "windowPointer=0x${windowPointer.toString(16)}")
            if (windowPointer == 0L) {
                Log.e("TorvoxRuntime", "getNativeWindowPtr returned 0 - surface invalid!")
                val fallbackPointer = getNativeWindowPtrReflection(surface)
                Log.d("TorvoxRuntime", "reflection fallback: 0x${fallbackPointer.toString(16)}")
                if (fallbackPointer == 0L) {
                    Log.e("TorvoxRuntime", "all methods to get window ptr failed, aborting start")
                    return
                }
                windowPointer = fallbackPointer
            }
        } else {
            Log.d("TorvoxRuntime", "no surface — using GPU offscreen rendering path")
        }

        try {
            // Allow test override via system property (no DataStore dependency)
            val testUrl = System.getProperty("torvox.test.bootstrapUrl")
            val bootstrapUrl = if (testUrl != null) testUrl else settingsRepository.bootstrapUrl.first()
            if (bootstrapUrl.isNotEmpty()) {
                Log.d("TorvoxRuntime", "Bootstrap URL set: $bootstrapUrl")
                val downloader = io.torvox.installer.BootstrapDownloader(context)
                val installer = io.torvox.installer.BootstrapInstaller()
                val secondStage = io.torvox.installer.SecondStageRunner()
                val orchestrator = io.torvox.installer.BootstrapOrchestrator(downloader, installer, secondStage)
                when (orchestrator.getInstallStatus()) {
                    io.torvox.installer.BootstrapOrchestrator.Status.NOT_INSTALLED -> {
                        Log.d("TorvoxRuntime", "Bootstrap not installed, starting install...")
                        val result = orchestrator.ensureBootstrap(bootstrapUrl)
                        Log.d("TorvoxRuntime", "Bootstrap result: $result")
                    }

                    io.torvox.installer.BootstrapOrchestrator.Status.INSTALLED -> {
                        Log.d("TorvoxRuntime", "Bootstrap already installed")
                    }

                    else -> {}
                }
            }
            val config = buildConfig()
            Log.d(
                "TorvoxRuntime",
                "buildConfig: fontSizeTenths=${config.font_size_tenths} rows=${config.rows} cols=${config.cols} theme=${config.theme.name}",
            )
            val bridge = createBridge(config)
            Log.d("TorvoxRuntime", "bridge created: ${bridge.ping()}")

            val sessionId = 1L
            val savePath = sessionSavePath(sessionId)
            bridge.setSavePath(savePath)

            bridge.setNativeWindow(windowPointer, width, height)
            Log.d("TorvoxRuntime", "setNativeWindow OK: width=$width height=$height")

            val spawnResult = bridge.spawnTerminal(config.rows, config.cols)
            Log.d("TorvoxRuntime", "spawnTerminal: rows=${config.rows} cols=${config.cols} result=$spawnResult")

            val shouldRestore = settingsRepository.sessionRestore.first()
            if (shouldRestore && bridge.hasSavedSession(savePath)) {
                Log.d("TorvoxRuntime", "restoring saved session from $savePath")
                try {
                    bridge.restoreSession(savePath)
                } catch (exception: Exception) {
                    Log.e("TorvoxRuntime", "Session restore failed, deleting corrupted file", exception)
                    java.io.File(savePath).delete()
                }
            } else if (!shouldRestore && bridge.hasSavedSession(savePath)) {
                Log.d("TorvoxRuntime", "session_restore=OFF, deleting saved session")
                java.io.File(savePath).delete()
            }

            try {
                val initialFontFamily = settingsRepository.fontFamily.first()
                val effectiveFont =
                    if (initialFontFamily.isNotEmpty() && !initialFontFamily.equals("System Default", true)) {
                        initialFontFamily
                    } else {
                        "monospace"
                    }
                bridge.setFontFamily(effectiveFont)
                bridge.setTheme(config.theme)
                Log.d(
                    "TorvoxRuntime",
                    "settings applied: fontFamily=$effectiveFont theme=${config.theme.name}",
                )
            } catch (exception: Exception) {
                Log.e("TorvoxRuntime", "Failed to apply initial settings", exception)
            }

            val entry =
                SessionEntry(
                    id = sessionId,
                    bridge = bridge,
                    renderThread = null,
                    running = true,
                    savePath = savePath,
                )
            sessions[sessionId] = entry
            activeSessionId = sessionId
            if (blitCallback != null) {
                entry.blitCallback = blitCallback
            }
            startRenderThread(entry)

            _state.value =
                RuntimeState(
                    isRunning = true,
                    rows = config.rows.toInt(),
                    cols = config.cols.toInt(),
                    activeSessionId = sessionId,
                    sessionIds = listOf(sessionId),
                )
            Log.d(
                "TorvoxRuntime",
                "session $sessionId config: rows=${config.rows} cols=${config.cols} fontSizeTenths=${config.font_size_tenths}",
            )
            Log.d("TorvoxRuntime", "session $sessionId started")
        } catch (exception: Throwable) {
            Log.e("TorvoxRuntime", "Failed to start terminal", exception)
            LogcatFileWriter.write("TorvoxRuntime", "FAILED to start terminal: ${exception.message}\n${exception.stackTraceToString()}")
        } finally {
            starting = false
        }
    }

    suspend fun createSession(
        surface: Surface,
        width: Int,
        height: Int,
    ): Long {
        if (width <= 0 || height <= 0) {
            Log.e("TorvoxRuntime", "createSession: invalid dimensions ${width}x$height")
            return -1L
        }
        if (!surface.isValid) {
            Log.e("TorvoxRuntime", "createSession: surface is not valid")
            return -1L
        }
        val nextId = (sessions.keys.maxOrNull() ?: 0L) + 1
        Log.d("TorvoxRuntime", "createSession() id=$nextId")

        try {
            val config = buildConfig()
            val bridge = createBridge(config)

            val savePath = sessionSavePath(nextId)
            bridge.setSavePath(savePath)

            val entry =
                SessionEntry(
                    id = nextId,
                    bridge = bridge,
                    renderThread = null,
                    running = false,
                    savePath = savePath,
                )
            sessions[nextId] = entry

            try {
                switchSessionInternal(
                    nextId,
                    surface,
                    width,
                    height,
                    needsSpawn = true,
                    spawnRows = config.rows,
                    spawnCols = config.cols,
                )
            } catch (exception: Throwable) {
                Log.e("TorvoxRuntime", "Failed to switch to new session $nextId, rolling back", exception)
                sessions.remove(nextId)
                throw exception
            }

            updateState()
            Log.d("TorvoxRuntime", "session $nextId created and activated")
            return nextId
        } catch (exception: Throwable) {
            Log.e("TorvoxRuntime", "Failed to create session $nextId", exception)
            LogcatFileWriter.write(
                "TorvoxRuntime",
                "FAILED to create session $nextId: ${exception.message}\n${exception.stackTraceToString()}",
            )
            return -1L
        }
    }

    fun switchSession(
        id: Long,
        surface: Surface,
        width: Int,
        height: Int,
    ) {
        switchSessionInternal(id, surface, width, height)
        updateState()
    }

    private fun switchSessionInternal(
        id: Long,
        surface: Surface,
        width: Int,
        height: Int,
        needsSpawn: Boolean = false,
        spawnRows: UInt = 24u,
        spawnCols: UInt = 80u,
    ) {
        val target =
            sessions[id] ?: run {
                Log.e("TorvoxRuntime", "switchSession: session $id not found")
                return
            }
        if (id == activeSessionId && !needsSpawn) return

        val windowPointer = getNativeWindowPtr(surface)
        if (windowPointer == 0L) {
            Log.e("TorvoxRuntime", "switchSession: failed to get native window ptr")
            return
        }

        val current = sessions[activeSessionId]
        if (current != null) {
            try {
                stopRenderThread(current)
                // Release the old bridge's GPU surface before the new bridge
                // creates its own on the same ANativeWindow. This avoids
                // VK_ERROR_NATIVE_WINDOW_IN_USE_KHR from the Vulkan driver
                // when two wgpu surfaces share the same ANativeWindow.
                current.bridge?.releaseGpuSurface()
            } catch (exception: Exception) {
                Log.e("TorvoxRuntime", "switchSession: error stopping current session", exception)
            }
        }

        try {
            if (needsSpawn) {
                target.bridge?.setNativeWindow(windowPointer, width, height)
                val spawnResult = target.bridge?.spawnTerminal(spawnRows, spawnCols)
                Log.d(
                    "TorvoxRuntime",
                    "switchSessionInternal spawnTerminal for session $id rows=$spawnRows cols=$spawnCols result=$spawnResult",
                )
            } else {
                target.bridge?.setNativeWindow(windowPointer, width, height)
                // Always update the GPU surface after setNativeWindow.
                // If releaseGpuSurface was called on this bridge during a
                // previous session switch, the wgpu surface is gone and must
                // be recreated from the current ANativeWindow via updateNativeWindow.
                target.bridge?.updateNativeWindow(windowPointer, width, height)
            }
            target.running = true
            startRenderThread(target)
            activeSessionId = id
            target.bridge?.let { syncGridDimensions(it) }
            Log.d("TorvoxRuntime", "switched to session $id")
        } catch (exception: Exception) {
            Log.e("TorvoxRuntime", "switchSession: setNativeWindow failed for session $id", exception)
        }
    }

    fun closeSession(id: Long) {
        val entry = sessions[id] ?: return
        Log.d("TorvoxRuntime", "closeSession($id)")

        if (id == activeSessionId) {
            stopRenderThread(entry)
        }
        sessions.remove(id)

        // If we closed the active session, switch to another
        if (id == activeSessionId) {
            val remaining = sessions.keys.sorted()
            if (remaining.isNotEmpty()) {
                val newId = remaining.last()
                activeSessionId = newId
                val newEntry =
                    sessions[newId] ?: run {
                        Log.w("TorvoxRuntime", "closeSession: new active session $newId already removed")
                        activeSessionId = 0L
                        updateState()
                        return
                    }
                newEntry.running = true
                val bridge =
                    newEntry.bridge ?: run {
                        Log.w("TorvoxRuntime", "closeSession: new active session $newId has no bridge")
                        activeSessionId = 0L
                        updateState()
                        return
                    }
                startRenderThread(newEntry)
                bridge.let { syncGridDimensions(it) }
                Log.d("TorvoxRuntime", "closeSession: restarted render for session $newId")
            } else {
                activeSessionId = 0L
            }
        }
        updateState()
    }

    suspend fun applySettings() {
        val config = buildConfig()
        val fontFamily = settingsRepository.fontFamily.first()
        val effectiveFontFamily =
            if (fontFamily.isNotEmpty() && !fontFamily.equals("System Default", ignoreCase = true)) {
                fontFamily
            } else {
                "monospace"
            }
        sessions.values.forEach { entry ->
            entry.bridge?.setFontSize(config.font_size_tenths)
            entry.bridge?.setFontFamily(effectiveFontFamily)
            entry.bridge?.setTheme(config.theme)
            entry.bridge?.resize(config.rows, config.cols)
        }
    }

    fun writeToPty(data: ByteArray) {
        val now = System.nanoTime()
        if (now - lastWriteTime < 20_000_000L &&
            data.contentEquals(lastWriteData)
        ) {
            Log.d("TorvoxRuntime", "writeToPty: dedup'd duplicate write within 20ms")
            return
        }
        lastWriteData = data
        lastWriteTime = now
        val entry = sessions[activeSessionId]
        if (entry != null && entry.running) {
            entry.bridge?.writeToPty(data)
        }
    }

    fun bridge(): TorvoxBridge? = sessions[activeSessionId]?.bridge

    fun activeSessionBridge(id: Long): TorvoxBridge? = sessions[id]?.bridge

    fun focusChange(focused: Boolean) {
        sessions.forEach { (_, entry) ->
            entry.bridge?.focusEvent(focused)
        }
    }

    fun currentCwd(): String {
        val entry = sessions[activeSessionId] ?: return ""
        return entry.bridge?.cwd() ?: ""
    }

    fun currentSessionIds(): List<Long> = sessions.keys.sorted()

    fun currentActiveSessionId(): Long = activeSessionId

    suspend fun saveSession() {
        val shouldSave = settingsRepository.sessionRestore.first()
        if (!shouldSave) {
            Log.d("TorvoxRuntime", "session_restore=OFF, skipping save")
            return
        }
        val entry = sessions[activeSessionId] ?: return
        entry.bridge?.setSavePath(entry.savePath)
        try {
            entry.bridge?.saveSession(entry.savePath)
            Log.d("TorvoxRuntime", "session saved to ${entry.savePath}")
        } catch (exception: Exception) {
            Log.e("TorvoxRuntime", "Session save failed", exception)
        }
    }

    fun stop() {
        synchronized(stopLock) {
            if (stopped) return
            stopped = true
        }
        sessions.values.forEach { entry ->
            stopRenderThread(entry)
            entry.bridge?.releaseSurface()
            entry.bridge?.close()
        }
        sessions.clear()
        activeSessionId = 0L
        _state.value = RuntimeState()
    }

    fun pauseRendering() {
        sessions.values.forEach { entry ->
            if (entry.running) {
                stopRenderThread(entry)
                entry.running = false
                Log.d("TorvoxRuntime", "pauseRendering: session ${entry.id} stopped")
            }
        }
    }

    fun setSelection(
        startRow: UInt,
        startCol: UInt,
        endRow: UInt,
        endCol: UInt,
        hasSelection: Boolean,
    ) {
        Log.d("TorvoxRuntime", "setSelection: start=($startRow,$startCol) end=($endRow,$endCol) active=$hasSelection")
        selectionState.set(Triple(Pair(startRow, startCol), Pair(endRow, endCol), hasSelection))
    }

    fun resize(
        rows: Int,
        cols: Int,
    ) {
        val bridge = sessions[activeSessionId]?.bridge ?: return
        bridge.resize(rows.toUInt(), cols.toUInt())
        _state.value = _state.value.copy(rows = rows, cols = cols)
    }

    fun recomputeGrid(
        width: Int,
        height: Int,
    ) {
        val bridge = sessions[activeSessionId]?.bridge ?: return
        bridge.recomputeGrid(width.toUInt(), height.toUInt())
        syncGridDimensions(bridge)
    }

    fun updateNativeWindow(
        windowPointer: Long,
        width: Int,
        height: Int,
    ) {
        val entry = sessions[activeSessionId] ?: return
        try {
            entry.bridge?.updateNativeWindow(windowPointer, width, height)
            entry.bridge?.let { syncGridDimensions(it) }
            if (entry.renderThread?.isAlive != true) {
                Log.d("TorvoxRuntime", "updateNativeWindow: render thread dead, restarting for session ${entry.id}")
                startRenderThread(entry)
            }
        } catch (exception: Exception) {
            Log.e("TorvoxRuntime", "updateNativeWindow failed", exception)
        }
    }

    private fun syncGridDimensions(bridge: TorvoxBridge) {
        val rows = bridge.getGridRows()
        val cols = bridge.getGridCols()
        val sizeChanged = rows != _state.value.rows || cols != _state.value.cols
        if (rows > 0 && cols > 0 && sizeChanged) {
            _state.value = _state.value.copy(rows = rows, cols = cols)
        }
    }

    fun setBlitCallback(callback: () -> Unit) {
        val entry = sessions[activeSessionId] ?: return
        entry.blitCallback = callback
    }

    fun destroy() {
        stop()
        scope.cancel()
    }

    private fun startRenderThread(entry: SessionEntry) {
        stopRenderThread(entry)
        val generation = renderGeneration.incrementAndGet()
        entry.running = true
        entry.renderThread =
            Thread({
                var diagCount = 0
                var consecutiveErrors = 0
                Log.d("TorvoxRuntime", "render thread started for session ${entry.id} generation=$generation")
                while (entry.running && renderGeneration.get() == generation) {
                    try {
                        val bridge = entry.bridge ?: break
                        val sel = selectionState.get()
                        bridge.setSelection(
                            sel.first.first,
                            sel.first.second,
                            sel.second.first,
                            sel.second.second,
                            sel.third,
                        )
                        val result = bridge.render()
                        if (result != 0) {
                            consecutiveErrors++
                            if (consecutiveErrors == 1) {
                                Log.e("TorvoxRuntime", "session ${entry.id} render error code=$result")
                                LogcatFileWriter.write(
                                    "TorvoxRuntime",
                                    "session ${entry.id} render error code=$result",
                                )
                            } else if (consecutiveErrors % 60 == 0) {
                                Log.e("TorvoxRuntime", "session ${entry.id} render error $result (x$consecutiveErrors)")
                            }
                            if (consecutiveErrors > 300) {
                                Log.e("TorvoxRuntime", "session ${entry.id} too many render errors, stopping")
                                break
                            }
                            Thread.sleep(100)
                        } else {
                            if (consecutiveErrors > 0) {
                                Log.i(
                                    "TorvoxRuntime",
                                    "session ${entry.id} recovered after $consecutiveErrors errors",
                                )
                            }
                            consecutiveErrors = 0
                            try {
                                if (bridge.pollBel()) {
                                    val tg = ToneGenerator(0, 50)
                                    tg.startTone(24, 200)
                                    tg.release()
                                }
                            } catch (_: Exception) {
                            }
                            try {
                                val notification = bridge.pollNotification()
                                if (notification != null) {
                                    val (title, body) = notification
                                    val toastText = if (title.isNotEmpty()) "$title: $body" else body
                                    Handler(Looper.getMainLooper()).post {
                                        android.widget.Toast
                                            .makeText(context, toastText, android.widget.Toast.LENGTH_LONG)
                                            .show()
                                    }
                                }
                            } catch (_: Exception) {
                            }
                            try {
                                val clipboardText = bridge.pollClipboard()
                                if (clipboardText != null) {
                                    val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
                                    val clip = ClipData.newPlainText("terminal clipboard", clipboardText)
                                    clipboard.setPrimaryClip(clip)
                                }
                            } catch (_: Exception) {
                            }
                            try {
                                entry.blitCallback?.invoke()
                            } catch (exception: Exception) {
                            }
                            diagCount++
                            if (diagCount == 1) {
                                val terminalText =
                                    try {
                                        bridge.getTerminalText()
                                    } catch (exception: Exception) {
                                        null
                                    }
                                val scrollbackLength = bridge.scrollbackLength()
                                val preview = terminalText?.take(80) ?: "null"
                                Log.d(
                                    "TorvoxRuntime",
                                    "session ${entry.id} first render OK: " +
                                        "scrollback=$scrollbackLength " +
                                        "text='$preview'",
                                )
                            }
                            if (diagCount % 60 == 0) {
                                val scrollbackLength = bridge.scrollbackLength()
                                val terminalText =
                                    try {
                                        bridge.getTerminalText()
                                    } catch (exception: Exception) {
                                        null
                                    }
                                Log.d(
                                    "TorvoxRuntime",
                                    "session ${entry.id} scrollback=$scrollbackLength text='${terminalText?.take(80) ?: "N/A"}'",
                                )
                                val title =
                                    try {
                                        bridge.getActiveSessionTitle()
                                    } catch (exception: Exception) {
                                        ""
                                    }
                                if (title.isNotEmpty() && title != _state.value.title) {
                                    _state.value = _state.value.copy(title = title)
                                }
                            }
                            if (diagCount % 600 == 0) {
                                Log.d("TorvoxRuntime", "session ${entry.id} render alive: $diagCount")
                            }
                            Thread.sleep(16)
                        }
                    } catch (exception: Exception) {
                        consecutiveErrors++
                        if (consecutiveErrors == 1) {
                            Log.e("TorvoxRuntime", "session ${entry.id} first render exception", exception)
                        } else if (consecutiveErrors % 60 == 0) {
                            Log.e("TorvoxRuntime", "session ${entry.id} render exception", exception)
                        }
                        if (consecutiveErrors > 300) {
                            Log.e("TorvoxRuntime", "session ${entry.id} too many render exceptions", exception)
                            break
                        }
                        Thread.sleep(100)
                    }
                }
                Log.d("TorvoxRuntime", "render thread stopped for session ${entry.id}")
            }, "TorvoxRender-${entry.id}").apply {
                isDaemon = true
                start()
            }
    }

    private fun stopRenderThread(entry: SessionEntry) {
        entry.running = false
        entry.renderThread?.let { thread ->
            thread.interrupt()
            thread.join(500)
        }
        entry.renderThread = null
    }

    private fun updateState() {
        val currentTitle = sessions[activeSessionId]?.bridge?.getActiveSessionTitle() ?: _state.value.title
        _state.value =
            RuntimeState(
                isRunning = sessions.isNotEmpty(),
                title = currentTitle.ifEmpty { _state.value.title },
                rows = _state.value.rows,
                cols = _state.value.cols,
                activeSessionId = activeSessionId,
                sessionIds = sessions.keys.sorted(),
            )
    }

    fun getNativeWindowPtr(surface: Surface): Long = try {
        if (!io.torvox.bridge.NativeWindow
                .isNativeLoaded()
        ) {
            Log.w("TorvoxRuntime", "Native lib not loaded, using reflection fallback")
            return getNativeWindowPtrReflection(surface)
        }
        io.torvox.bridge.NativeWindow
            .getNativeWindowPtr(surface)
    } catch (exception: Throwable) {
        Log.w("TorvoxRuntime", "JNI getNativeWindowPtr not available, falling back to mNativeObject reflection")
        getNativeWindowPtrReflection(surface)
    }

    private fun getNativeWindowPtrReflection(surface: Surface): Long {
        try {
            val method = surface.javaClass.getMethod("getNativeWindow")
            return method.invoke(surface) as Long
        } catch (_: Exception) {
        }
        try {
            val field = surface.javaClass.getDeclaredField("mNativeObject")
            field.isAccessible = true
            return field.getLong(surface)
        } catch (_: Exception) {
        }
        Log.e("TorvoxRuntime", "All methods to get native window pointer failed")
        return 0L
    }
}
