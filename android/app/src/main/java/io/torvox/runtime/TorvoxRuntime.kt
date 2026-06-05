package io.torvox.runtime

import android.content.Context
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

private data class SessionEntry(
    val id: Long,
    val bridge: TorvoxBridge,
    var renderThread: Thread?,
    @Volatile var running: Boolean,
    val savePath: String,
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

        private val sessions = mutableMapOf<Long, SessionEntry>()
        private var activeSessionId: Long = 0L

        @Volatile private var stopped = false
        private val stopLock = Any()

        private fun sessionSavePath(id: Long): String = "${context.filesDir.absolutePath}/session_$id.bin"

        private suspend fun buildConfig(
            rows: UInt = 24u,
            cols: UInt = 80u,
        ): TerminalConfig {
            val shellPath = settingsRepository.shell.first()
            val scrollbackLines = settingsRepository.scrollbackLines.first()
            val fontSizeTenths = (settingsRepository.fontSize.first() * 10).toInt().toUInt()
            val themeMode = settingsRepository.themeMode.first()
            val dayTheme = settingsRepository.dayThemeName.first()
            val nightTheme = settingsRepository.nightThemeName.first()
            val systemDark =
                (context.resources.configuration.uiMode and android.content.res.Configuration.UI_MODE_NIGHT_MASK) ==
                    android.content.res.Configuration.UI_MODE_NIGHT_YES
            val themeName =
                when (themeMode) {
                    "day" -> dayTheme
                    "night" -> nightTheme
                    else -> if (systemDark) nightTheme else dayTheme
                }
            val shell =
                if (shellPath == "/system/bin/sh" || shellPath.isEmpty()) {
                    Shell.SystemDefault
                } else {
                    Shell.Custom(shellPath)
                }
            val resolvedTheme = BuiltInThemes.byName(themeName)
            val colorToInt: (androidx.compose.ui.graphics.Color) -> Int = { c ->
                ((c.alpha * 255).toInt() shl 24) or ((c.red * 255).toInt() shl 16) or
                    ((c.green * 255).toInt() shl 8) or (c.blue * 255).toInt()
            }
            val bg = colorToInt(resolvedTheme.background)
            val fg = colorToInt(resolvedTheme.foreground)
            val cursor = colorToInt(resolvedTheme.cursor)
            val ansiInts = resolvedTheme.ansi.map(colorToInt)
            return TerminalConfig(
                shell = shell,
                rows = rows,
                cols = cols,
                scrollbackLines = scrollbackLines.toUInt(),
                font_size_tenths = fontSizeTenths,
                theme =
                    BridgeTheme(
                        name = themeName,
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
                    ),
            )
        }

        suspend fun start(
            surface: Surface,
            width: Int,
            height: Int,
        ) {
            if (sessions.isNotEmpty()) return
            stopped = false
            Log.d("TorvoxRuntime", "start() called: width=$width height=$height")

            // Capture window pointer BEFORE any suspend point to avoid race with surfaceDestroyed
            val windowPtr = getNativeWindowPtr(surface)
            Log.d("TorvoxRuntime", "windowPtr=0x${windowPtr.toString(16)}")
            if (windowPtr == 0L) {
                Log.e("TorvoxRuntime", "getNativeWindowPtr returned 0 - surface invalid!")
                return
            }

            try {
                val config = buildConfig()
                val bridge = createBridge(config)
                Log.d("TorvoxRuntime", "bridge created: ${bridge.ping()}")

                val sessionId = 1L
                val savePath = sessionSavePath(sessionId)
                bridge.setSavePath(savePath)

                bridge.setNativeWindow(windowPtr, width, height)
                Log.d("TorvoxRuntime", "setNativeWindow OK: width=$width height=$height")

                // Apply settings that were not applied during setNativeWindow
                try {
                    bridge.setFontSize(config.font_size_tenths)
                    val initialFontFamily = settingsRepository.fontFamily.first()
                    if (initialFontFamily.isNotEmpty() && !initialFontFamily.equals("System Default", true)) {
                        bridge.setFontFamily(initialFontFamily)
                    }
                    bridge.setTheme(config.theme)
                    Log.d(
                        "TorvoxRuntime",
                        "settings applied: fontSize=${config.font_size_tenths} fontFamily=$initialFontFamily theme=${config.theme.name}",
                    )
                } catch (e: Exception) {
                    Log.e("TorvoxRuntime", "Failed to apply initial settings", e)
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
                startRenderThread(entry)

                _state.value =
                    RuntimeState(
                        isRunning = true,
                        rows = config.rows.toInt(),
                        cols = config.cols.toInt(),
                        activeSessionId = sessionId,
                        sessionIds = listOf(sessionId),
                    )
                Log.d("TorvoxRuntime", "session $sessionId started")
            } catch (e: Exception) {
                Log.e("TorvoxRuntime", "Failed to start terminal", e)
            }
        }

        suspend fun createSession(
            surface: Surface,
            width: Int,
            height: Int,
        ): Long {
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

                // Switch to the new session
                switchSessionInternal(nextId, surface, width, height)

                updateState()
                Log.d("TorvoxRuntime", "session $nextId created and activated")
                return nextId
            } catch (e: Exception) {
                Log.e("TorvoxRuntime", "Failed to create session $nextId", e)
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
        ) {
            val target =
                sessions[id] ?: run {
                    Log.e("TorvoxRuntime", "switchSession: session $id not found")
                    return
                }
            if (id == activeSessionId) return

            // Stop current active session's render thread
            val current = sessions[activeSessionId]
            if (current != null) {
                stopRenderThread(current)
                current.bridge.releaseSurface()
            }

            // Attach surface to new session
            val windowPtr = getNativeWindowPtr(surface)
            if (windowPtr != 0L) {
                target.bridge.setNativeWindow(windowPtr, width, height)
            }

            activeSessionId = id
            target.running = true
            startRenderThread(target)
            Log.d("TorvoxRuntime", "switched to session $id")
        }

        fun closeSession(id: Long) {
            val entry = sessions[id] ?: return
            Log.d("TorvoxRuntime", "closeSession($id)")

            if (id == activeSessionId) {
                stopRenderThread(entry)
                entry.bridge.releaseSurface()
            }
            entry.bridge.close()
            sessions.remove(id)

            // If we closed the active session, switch to another
            if (id == activeSessionId) {
                val remaining = sessions.keys.sorted()
                if (remaining.isNotEmpty()) {
                    val newId = remaining.last()
                    activeSessionId = newId
                    val newEntry = sessions[newId]!!
                    newEntry.running = true
                    // Surface will be re-attached on next render call
                } else {
                    activeSessionId = 0L
                }
            }
            updateState()
        }

        suspend fun applySettings() {
            val config = buildConfig()
            val fontFamily = settingsRepository.fontFamily.first()
            sessions.values.forEach { entry ->
                entry.bridge.setFontSize(config.font_size_tenths)
                if (fontFamily.isNotEmpty() && !fontFamily.equals("System Default", true)) {
                    entry.bridge.setFontFamily(fontFamily)
                }
                entry.bridge.setTheme(config.theme)
            }
        }

        fun writeToPty(data: ByteArray) {
            sessions[activeSessionId]?.bridge?.writeToPty(data)
        }

        fun bridge(): TorvoxBridge? = sessions[activeSessionId]?.bridge

        fun activeSessionBridge(id: Long): TorvoxBridge? = sessions[id]?.bridge

        fun currentSessionIds(): List<Long> = sessions.keys.sorted()

        fun currentActiveSessionId(): Long = activeSessionId

        fun saveSession() {
            val entry = sessions[activeSessionId] ?: return
            entry.bridge.setSavePath(entry.savePath)
            entry.bridge.saveSession(entry.savePath)
        }

        fun stop() {
            synchronized(stopLock) {
                if (stopped) return
                stopped = true
            }
            sessions.values.forEach { entry ->
                stopRenderThread(entry)
                entry.bridge.releaseSurface()
                entry.bridge.close()
            }
            sessions.clear()
            activeSessionId = 0L
            _state.value = RuntimeState()
        }

        fun resize(
            rows: Int,
            cols: Int,
        ) {
            sessions[activeSessionId]?.bridge?.resize(rows.toUInt(), cols.toUInt())
            _state.value = _state.value.copy(rows = rows, cols = cols)
        }

        fun destroy() {
            stop()
            scope.cancel()
        }

        private fun startRenderThread(entry: SessionEntry) {
            stopRenderThread(entry)
            entry.running = true
            entry.renderThread =
                Thread({
                    var diagCount = 0
                    var consecutiveErrors = 0
                    while (entry.running) {
                        try {
                            val result = entry.bridge.render()
                            if (result != 0) {
                                consecutiveErrors++
                                if (consecutiveErrors > 300) {
                                    Log.e("TorvoxRuntime", "session ${entry.id} too many render errors, stopping")
                                    break
                                }
                                if (consecutiveErrors == 1 || consecutiveErrors % 60 == 0) {
                                    Log.e("TorvoxRuntime", "session ${entry.id} render error $result (x$consecutiveErrors)")
                                }
                                Thread.sleep(100)
                            } else {
                                consecutiveErrors = 0
                                diagCount++
                                if (diagCount == 60) {
                                    val sl = entry.bridge.scrollbackLen()
                                    Log.d("TorvoxRuntime", "session ${entry.id} scrollback=$sl")
                                }
                                if (diagCount % 600 == 0) {
                                    Log.d("TorvoxRuntime", "session ${entry.id} render alive: $diagCount")
                                }
                                Thread.sleep(16)
                            }
                        } catch (e: Exception) {
                            consecutiveErrors++
                            if (consecutiveErrors > 300) {
                                Log.e("TorvoxRuntime", "session ${entry.id} too many render exceptions", e)
                                break
                            }
                            if (consecutiveErrors % 60 == 0) {
                                Log.e("TorvoxRuntime", "session ${entry.id} render exception", e)
                            }
                            Thread.sleep(100)
                        }
                    }
                }, "TorvoxRender-${entry.id}").apply {
                    isDaemon = true
                    start()
                }
        }

        private fun stopRenderThread(entry: SessionEntry) {
            entry.running = false
            entry.renderThread?.join(500)
            entry.renderThread = null
        }

        private fun updateState() {
            _state.value =
                RuntimeState(
                    isRunning = sessions.isNotEmpty(),
                    rows = _state.value.rows,
                    cols = _state.value.cols,
                    activeSessionId = activeSessionId,
                    sessionIds = sessions.keys.sorted(),
                )
        }

        private fun getNativeWindowPtr(surface: Surface): Long =
            try {
                io.torvox.bridge.NativeWindow
                    .getNativeWindowPtr(surface)
            } catch (e: UnsatisfiedLinkError) {
                Log.w("TorvoxRuntime", "JNI getNativeWindowPtr not available, falling back to reflection")
                val method = surface.javaClass.getMethod("getNativeWindow")
                method.invoke(surface) as Long
            }
    }
