package io.torvox

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.provider.OpenableColumns
import android.view.Surface
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import io.torvox.runtime.TorvoxRuntime
import io.torvox.settings.SettingsRepository
import io.torvox.ui.ModifierKey
import io.torvox.ui.defaultModifierKeys
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

enum class SelectionMode {
    Char,
    Word,
    Line,
    Block,
}

data class SelectionAnchor(
    val row: Int,
    val col: Int,
)

data class SelectionState(
    val active: Boolean = false,
    val start: SelectionAnchor? = null,
    val end: SelectionAnchor? = null,
    val mode: SelectionMode = SelectionMode.Char,
    val selectedText: String = "",
)

data class SessionInfo(
    val id: Long,
    val title: String,
)

data class TerminalState(
    val sessionId: Long = 0L,
    val isRunning: Boolean = false,
    val title: String = "Torvox",
    val selection: SelectionState = SelectionState(),
    val pendingInput: ByteArray? = null,
    val modifierKeys: List<ModifierKey> = defaultModifierKeys,
    val ctrlActive: Boolean = false,
    val altActive: Boolean = false,
    val sessions: List<SessionInfo> = emptyList(),
    val activeSessionId: Long = 0L,
)

@HiltViewModel
class TerminalViewModel
    @Inject
    constructor(
        @ApplicationContext private val context: Context,
        private val settingsRepository: SettingsRepository,
        val runtime: TorvoxRuntime,
    ) : ViewModel() {
        private val _state = MutableStateFlow(TerminalState())
        val state: StateFlow<TerminalState> = _state.asStateFlow()

        var currentSurface: Surface? = null
        var surfaceWidth: Int = 0
        var surfaceHeight: Int = 0

        fun startRuntime(
            surface: Surface,
            width: Int,
            height: Int,
        ) {
            viewModelScope.launch {
                runtime.start(surface, width, height)
            }
        }

        val fontSize: StateFlow<Float> =
            settingsRepository.fontSize
                .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), 14f)

        val themeName: StateFlow<String> =
            settingsRepository.themeName
                .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), "Catppuccin Mocha")

        val shell: StateFlow<String> =
            settingsRepository.shell
                .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), "/system/bin/sh")

        val scrollbackLines: StateFlow<Int> =
            settingsRepository.scrollbackLines
                .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), 50000)

        val fontFamily: StateFlow<String> =
            settingsRepository.fontFamily
                .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), "JetBrains Mono Nerd Font")

        val touchBehavior: StateFlow<String> =
            settingsRepository.touchBehavior
                .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), "right_click")

        val dayThemeName: StateFlow<String> =
            settingsRepository.dayThemeName
                .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), "Catppuccin Latte")

        val nightThemeName: StateFlow<String> =
            settingsRepository.nightThemeName
                .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), "Catppuccin Mocha")

        val themeMode: StateFlow<String> =
            settingsRepository.themeMode
                .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), "follow_system")

        val materialYouEnabled: StateFlow<Boolean> =
            settingsRepository.materialYouEnabled
                .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), false)

        init {
            viewModelScope.launch {
                runtime.state.collect { runtimeState ->
                    val sessions = runtimeState.sessionIds.map { SessionInfo(id = it, title = "Session $it") }
                    val active = runtimeState.activeSessionId
                    if (active != 0L) {
                        val title = sessions.firstOrNull { it.id == active }?.title ?: "Session $active"
                        _state.value =
                            _state.value.copy(
                                sessionId = active,
                                isRunning = runtimeState.isRunning,
                                title = title,
                                sessions = sessions,
                                activeSessionId = active,
                            )
                    } else {
                        _state.value = _state.value.copy(sessions = sessions, activeSessionId = active)
                    }
                }
            }
        }

        fun setFontSize(size: Float) {
            viewModelScope.launch {
                settingsRepository.setFontSize(size)
                runtime.applySettings()
            }
        }

        fun setFontFamily(family: String) {
            viewModelScope.launch {
                settingsRepository.setFontFamily(family)
                runtime.applySettings()
            }
        }

        fun setTouchBehavior(behavior: String) {
            viewModelScope.launch {
                settingsRepository.setTouchBehavior(behavior)
            }
        }

        fun setThemeName(name: String) {
            viewModelScope.launch {
                settingsRepository.setThemeName(name)
            }
        }

        fun setDayThemeName(name: String) {
            viewModelScope.launch {
                settingsRepository.setDayThemeName(name)
                runtime.applySettings()
            }
        }

        fun setNightThemeName(name: String) {
            viewModelScope.launch {
                settingsRepository.setNightThemeName(name)
                runtime.applySettings()
            }
        }

        fun setThemeMode(mode: String) {
            viewModelScope.launch {
                settingsRepository.setThemeMode(mode)
                runtime.applySettings()
            }
        }

        fun setMaterialYouEnabled(enabled: Boolean) {
            viewModelScope.launch {
                settingsRepository.setMaterialYouEnabled(enabled)
            }
        }

        fun getFileNameFromUri(uri: Uri): String? {
            val cursor = context.contentResolver.query(uri, null, null, null, null)
            return cursor?.use {
                if (it.moveToFirst()) {
                    val idx = it.getColumnIndex(OpenableColumns.DISPLAY_NAME)
                    if (idx >= 0) it.getString(idx) else null
                } else {
                    null
                }
            }
        }

        fun setShell(shell: String) {
            viewModelScope.launch {
                settingsRepository.setShell(shell)
            }
        }

        fun setScrollbackLines(lines: Int) {
            viewModelScope.launch {
                settingsRepository.setScrollbackLines(lines)
            }
        }

        fun startSelection(
            row: Int,
            col: Int,
        ) {
            val anchor = SelectionAnchor(row, col)
            _state.value =
                _state.value.copy(
                    selection =
                        SelectionState(
                            active = true,
                            start = anchor,
                            end = anchor,
                            mode = _state.value.selection.mode,
                        ),
                )
        }

        fun updateSelection(
            row: Int,
            col: Int,
        ) {
            val current = _state.value.selection
            if (!current.active) return
            _state.value =
                _state.value.copy(
                    selection = current.copy(end = SelectionAnchor(row, col)),
                )
        }

        fun endSelection() {
            val current = _state.value.selection
            if (!current.active || current.start == null || current.end == null) return
            val text = extractSelectedText(current)
            _state.value =
                _state.value.copy(
                    selection = current.copy(active = false, selectedText = text),
                )
        }

        fun setSelectionMode(mode: SelectionMode) {
            _state.value =
                _state.value.copy(
                    selection = _state.value.selection.copy(mode = mode),
                )
        }

        fun copySelectionToClipboard() {
            val text = _state.value.selection.selectedText
            if (text.isEmpty()) return
            val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
            val clip = ClipData.newPlainText("terminal selection", text)
            clipboard.setPrimaryClip(clip)
        }

        fun openUrl(url: String) {
            try {
                val intent = Intent(Intent.ACTION_VIEW, Uri.parse(url))
                intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                context.startActivity(intent)
            } catch (_: Exception) {
            }
        }

        fun clearSelection() {
            _state.value =
                _state.value.copy(
                    selection = SelectionState(),
                )
        }

        private fun extractSelectedText(selection: SelectionState): String {
            val start = selection.start ?: return ""
            val end = selection.end ?: return ""
            val bridge = runtime.bridge() ?: return ""
            val scrollbackLen = bridge.scrollbackLen().toInt()
            val (lo, hi) =
                if (start.row < end.row || (start.row == end.row && start.col <= end.col)) {
                    start to end
                } else {
                    end to start
                }
            return when (selection.mode) {
                SelectionMode.Char, SelectionMode.Word -> {
                    if (lo.row == hi.row) {
                        val line = bridge.scrollbackLine((scrollbackLen + lo.row).toUInt()) ?: ""
                        line.substring(lo.col.coerceAtMost(line.length), hi.col.coerceAtMost(line.length))
                    } else {
                        val parts = mutableListOf<String>()
                        for (r in lo.row..hi.row) {
                            val line = bridge.scrollbackLine((scrollbackLen + r).toUInt()) ?: ""
                            val startCol = if (r == lo.row) lo.col else 0
                            val endCol = if (r == hi.row) hi.col.coerceAtMost(line.length) else line.length
                            if (startCol < line.length) {
                                parts.add(line.substring(startCol, endCol.coerceAtMost(line.length)))
                            }
                        }
                        parts.joinToString("\n")
                    }
                }

                SelectionMode.Line -> {
                    val parts = mutableListOf<String>()
                    for (r in lo.row..hi.row) {
                        val line = bridge.scrollbackLine((scrollbackLen + r).toUInt()) ?: ""
                        parts.add(line)
                    }
                    parts.joinToString("\n")
                }

                SelectionMode.Block -> {
                    val parts = mutableListOf<String>()
                    for (r in lo.row..hi.row) {
                        val line = bridge.scrollbackLine((scrollbackLen + r).toUInt()) ?: ""
                        val startCol = lo.col.coerceAtMost(line.length)
                        val endCol = hi.col.coerceAtMost(line.length)
                        if (startCol < line.length) {
                            parts.add(line.substring(startCol, endCol))
                        }
                    }
                    parts.joinToString("\n")
                }
            }
        }

        fun writeToPty(data: ByteArray) {
            runtime.writeToPty(data)
        }

        fun consumePendingInput(): ByteArray? {
            val data = _state.value.pendingInput
            _state.value = _state.value.copy(pendingInput = null)
            return data
        }

        fun setModifierKeys(keys: List<ModifierKey>) {
            _state.value = _state.value.copy(modifierKeys = keys)
        }

        fun resetModifierKeys() {
            _state.value = _state.value.copy(modifierKeys = defaultModifierKeys)
        }

        fun setCtrlActive(active: Boolean) {
            _state.value = _state.value.copy(ctrlActive = active)
        }

        fun setAltActive(active: Boolean) {
            _state.value = _state.value.copy(altActive = active)
        }

        fun createSession() {
            val surface = currentSurface ?: return
            val w = surfaceWidth
            val h = surfaceHeight
            if (w == 0 || h == 0) return

            viewModelScope.launch {
                val newId = runtime.createSession(surface, w, h)
                if (newId > 0) {
                    val info = SessionInfo(id = newId, title = "Session $newId")
                    val newSessions = _state.value.sessions + info
                    _state.value =
                        _state.value.copy(
                            sessionId = newId,
                            isRunning = true,
                            title = info.title,
                            selection = SelectionState(),
                            pendingInput = null,
                            sessions = newSessions,
                            activeSessionId = newId,
                        )
                }
            }
        }

        fun switchSession(id: Long) {
            val surface = currentSurface ?: return
            val w = surfaceWidth
            val h = surfaceHeight
            if (w == 0 || h == 0) return

            runtime.switchSession(id, surface, w, h)
            val session = _state.value.sessions.find { it.id == id } ?: return
            _state.value =
                _state.value.copy(
                    sessionId = id,
                    isRunning = true,
                    title = session.title,
                    activeSessionId = id,
                    selection = SelectionState(),
                    pendingInput = null,
                )
        }

        fun closeSession() {
            closeSession(_state.value.activeSessionId)
        }

        fun closeSession(id: Long) {
            runtime.closeSession(id)
            val current = _state.value
            val remaining = current.sessions.filter { it.id != id }
            if (remaining.isEmpty()) {
                _state.value =
                    current.copy(
                        isRunning = false,
                        sessions = emptyList(),
                        activeSessionId = 0L,
                        selection = SelectionState(),
                        pendingInput = null,
                    )
            } else {
                val newActive =
                    if (current.activeSessionId == id) {
                        remaining.last().id
                    } else {
                        current.activeSessionId
                    }
                val activeSession = remaining.find { it.id == newActive }
                _state.value =
                    current.copy(
                        sessions = remaining,
                        activeSessionId = newActive,
                        sessionId = newActive,
                        title = activeSession?.title ?: "Torvox",
                        selection = SelectionState(),
                        pendingInput = null,
                    )
            }
        }

        fun setSessionTitle(title: String) {
            _state.value = _state.value.copy(title = title)
        }

        override fun onCleared() {
            runtime.saveSession()
            super.onCleared()
        }
    }
