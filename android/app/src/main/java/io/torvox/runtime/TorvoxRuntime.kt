package io.torvox.runtime

import android.content.Context
import android.view.Surface
import dagger.hilt.android.qualifiers.ApplicationContext
import io.torvox.bridge.Shell
import io.torvox.bridge.TerminalConfig
import io.torvox.bridge.createBridge
import io.torvox.settings.SettingsRepository
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

        private var bridge: io.torvox.bridge.TorvoxBridge? = null
        private var renderThread: Thread? = null
        private var running = false

        suspend fun start(
            surface: Surface,
            width: Int,
            height: Int,
        ) {
            if (running) return

            val shellPath = settingsRepository.shell.first()
            val scrollbackLines = settingsRepository.scrollbackLines.first()
            val fontSizeTenths = (settingsRepository.fontSize.first() * 10).toInt().toUInt()
            val themeName = settingsRepository.themeName.first()

            val shell =
                if (shellPath == "/system/bin/sh" || shellPath.isEmpty()) {
                    Shell.SystemDefault
                } else {
                    Shell.Custom(shellPath)
                }

            val config =
                TerminalConfig(
                    shell = shell,
                    rows = 24u,
                    cols = 80u,
                    scrollbackLines = scrollbackLines.toUInt(),
                    font_size_tenths = fontSizeTenths,
                    theme =
                        io.torvox.bridge.BridgeTheme(
                            name = themeName,
                            bg = 0x1E1E2Eu.toInt(),
                            fg = 0xCDD6F4u.toInt(),
                            cursor = 0xF5E0DCu.toInt(),
                            selectionBg = 0x45475Au.toInt(),
                            ansi0 = 0x45475Au.toInt(),
                            ansi1 = 0xF38BA8u.toInt(),
                            ansi2 = 0xA6E3A1u.toInt(),
                            ansi3 = 0xF9E2AFu.toInt(),
                            ansi4 = 0x89B4FAu.toInt(),
                            ansi5 = 0xF5C2E7u.toInt(),
                            ansi6 = 0x94E2D5u.toInt(),
                            ansi7 = 0xBAC2DEu.toInt(),
                            ansi8 = 0x585B70u.toInt(),
                            ansi9 = 0xF38BA8u.toInt(),
                            ansi10 = 0xA6E3A1u.toInt(),
                            ansi11 = 0xF9E2AFu.toInt(),
                            ansi12 = 0x89B4FAu.toInt(),
                            ansi13 = 0xF5C2E7u.toInt(),
                            ansi14 = 0x94E2D5u.toInt(),
                            ansi15 = 0xA6ADC8u.toInt(),
                        ),
                )

            bridge = createBridge(config)

            val windowPtr = getNativeWindowPtr(surface)
            bridge?.setNativeWindow(windowPtr)

            val cellWidth = 8f
            val cellHeight = 16f
            val cols = (width / cellWidth).toInt().coerceIn(20, 300)
            val rows = (height / cellHeight).toInt().coerceIn(5, 100)

            bridge?.resize(rows.toUInt(), cols.toUInt())

            running = true
            _state.value =
                RuntimeState(
                    isRunning = true,
                    rows = rows,
                    cols = cols,
                )

            renderThread =
                Thread({
                    while (running) {
                        try {
                            bridge?.render()
                            Thread.sleep(16)
                        } catch (_: Exception) {
                            break
                        }
                    }
                }, "TorvoxRender").apply {
                    isDaemon = true
                    start()
                }
        }

        fun writeToPty(data: ByteArray) {
            bridge?.writeToPty(data)
        }

        fun bridge(): io.torvox.bridge.TorvoxBridge? = bridge

        fun stop() {
            running = false
            renderThread?.join(1000)
            renderThread = null
            bridge?.releaseSurface()
            bridge?.close()
            bridge = null
            _state.value = RuntimeState()
        }

        fun resize(
            rows: Int,
            cols: Int,
        ) {
            bridge?.resize(rows.toUInt(), cols.toUInt())
            _state.value = _state.value.copy(rows = rows, cols = cols)
        }

        fun destroy() {
            stop()
            scope.cancel()
        }

        private fun getNativeWindowPtr(surface: Surface): Long {
            val method = surface.javaClass.getMethod("getNativeWindow")
            return method.invoke(surface) as Long
        }
    }
