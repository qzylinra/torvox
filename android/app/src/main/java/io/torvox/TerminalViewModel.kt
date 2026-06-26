// @Terminal ViewModel state management, IMPL_ANDR_KT_002, impl, [REQ_ANDR_003]
// @need-ids: REQ_ANDR_003

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
import io.torvox.ui.ModifierState
import io.torvox.ui.defaultModifierKeys
import io.torvox.ui.next
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

enum class SelectionMode {
    Char,
    Word,
    Line,
    Block,
    Semantic,
}

data class SelectionAnchor(
    val row: Int,
    val col: Int,
)

data class SelectionState(
    val active: Boolean = false,
    val dragging: Boolean = false,
    val start: SelectionAnchor? = null,
    val end: SelectionAnchor? = null,
    val mode: SelectionMode = SelectionMode.Char,
    val selectedText: String = "",
)

data class PastePopupRequest(
    val row: Int,
    val col: Int,
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
    val ctrlState: ModifierState = ModifierState.Off,
    val altState: ModifierState = ModifierState.Off,
    val scrollActive: Boolean = false,
    val sessions: List<SessionInfo> = emptyList(),
    val activeSessionId: Long = 0L,
    val pastePopupRequest: PastePopupRequest? = null,
    val keyboardMode: String = "secure",
)

internal fun shouldCreateDefaultSession(
    surfaceValid: Boolean,
    surfaceWidth: Int,
    surfaceHeight: Int,
    uiSessions: List<SessionInfo>,
    runtimeSessionIds: List<Long>,
): Boolean = surfaceValid &&
    surfaceWidth > 0 &&
    surfaceHeight > 0 &&
    uiSessions.isEmpty() &&
    runtimeSessionIds.isEmpty()

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
        surface: Surface?,
        width: Int,
        height: Int,
    ) {
        currentSurface = surface
        surfaceWidth = width
        surfaceHeight = height
        viewModelScope.launch {
            runtime.start(surface, width, height, null)
        }
    }

    val fontSize: StateFlow<Float> =
        settingsRepository.fontSize
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), 18f)

    val themeName: StateFlow<String> =
        settingsRepository.themeName
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), "Dracula Plus")

    val shell: StateFlow<String> =
        settingsRepository.shell
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), "/system/bin/sh")

    val scrollbackLines: StateFlow<Int> =
        settingsRepository.scrollbackLines
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), 50000)

    val fontFamily: StateFlow<String> =
        settingsRepository.fontFamily
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), "")

    val touchBehavior: StateFlow<String> =
        settingsRepository.touchBehavior
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), "right_click")

    val bootstrapUrl: StateFlow<String> =
        settingsRepository.bootstrapUrl
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), "")

    val useNerdFontGlyphs: StateFlow<Boolean> =
        settingsRepository.useNerdFontGlyphs
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), true)

    val useSemanticSelection: StateFlow<Boolean> =
        settingsRepository.useSemanticSelection
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), false)

    val sessionRestore: StateFlow<Boolean> =
        settingsRepository.sessionRestore
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), false)

    val keyboardMode: StateFlow<String> =
        settingsRepository.keyboardMode
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), "secure")

    val dayThemeName: StateFlow<String> =
        settingsRepository.dayThemeName
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), "Catppuccin Latte")

    val nightThemeName: StateFlow<String> =
        settingsRepository.nightThemeName
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), "Dracula Plus")

    val themeMode: StateFlow<String> =
        settingsRepository.themeMode
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), "fixed")

    val appThemeMode: StateFlow<String> =
        settingsRepository.appThemeMode
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), "follow_system")

    val usbSerialEnabled: StateFlow<Boolean> =
        settingsRepository.usbSerialEnabled
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), false)

    val mcpServerEnabled: StateFlow<Boolean> =
        settingsRepository.mcpServerEnabled
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), false)

    val volumeKeyMap: StateFlow<Boolean> =
        settingsRepository.volumeKeyMap
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), false)

    private val _availableFonts = MutableStateFlow<List<String>>(emptyList())
    val availableFonts: StateFlow<List<String>> = _availableFonts.asStateFlow()

    init {
        viewModelScope.launch {
            runtime.state.collect { runtimeState ->
                val sortedIds = runtimeState.sessionIds.sorted()
                val sessions =
                    sortedIds.mapIndexed { index, id ->
                        SessionInfo(id = id, title = "Session ${index + 1}")
                    }
                val active = runtimeState.activeSessionId
                if (active != 0L) {
                    val displayIndex = sortedIds.indexOf(active) + 1
                    val title =
                        if (runtimeState.title.isNotEmpty()) {
                            runtimeState.title
                        } else {
                            "Session $displayIndex"
                        }
                    _state.value =
                        _state.value.copy(
                            sessionId = active,
                            isRunning = runtimeState.isRunning,
                            title = title,
                            sessions = sessions,
                            activeSessionId = active,
                        )
                    if (_availableFonts.value.isEmpty() &&
                        runtime.state.value.sessionIds
                            .isNotEmpty()
                    ) {
                        loadFonts()
                    }
                } else {
                    _state.value = _state.value.copy(sessions = sessions, activeSessionId = active)
                }
            }
        }
        viewModelScope.launch {
            settingsRepository.keyboardMode.collect { mode ->
                _state.value = _state.value.copy(keyboardMode = mode)
            }
        }
    }

    private fun loadFonts() {
        viewModelScope.launch {
            try {
                val fonts = io.torvox.ui.fallbackSystemFonts()
                _availableFonts.value = fonts
            } catch (_: Exception) {
                _availableFonts.value = emptyList()
            }
        }
    }

    fun ensureDefaultSession() {
        if (!shouldCreateDefaultSession(
                surfaceValid = currentSurface?.isValid == true,
                surfaceWidth = surfaceWidth,
                surfaceHeight = surfaceHeight,
                uiSessions = _state.value.sessions,
                runtimeSessionIds = runtime.state.value.sessionIds,
            )
        ) {
            return
        }
        android.util.Log.d("TerminalViewModel", "ensureDefaultSession: creating default session")
        createSession()
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

    fun setBootstrapUrl(url: String) {
        viewModelScope.launch {
            settingsRepository.setBootstrapUrl(url)
        }
    }

    fun setUseNerdFontGlyphs(enabled: Boolean) {
        viewModelScope.launch {
            settingsRepository.setUseNerdFontGlyphs(enabled)
        }
    }

    fun setUseSemanticSelection(enabled: Boolean) {
        viewModelScope.launch {
            settingsRepository.setUseSemanticSelection(enabled)
        }
    }

    fun setSessionRestore(enabled: Boolean) {
        viewModelScope.launch {
            settingsRepository.setSessionRestore(enabled)
        }
    }

    fun setKeyboardMode(mode: String) {
        viewModelScope.launch {
            settingsRepository.setKeyboardMode(mode)
        }
    }

    fun setUsbSerialEnabled(enabled: Boolean) {
        viewModelScope.launch {
            settingsRepository.setUsbSerialEnabled(enabled)
        }
    }

    fun setMcpServerEnabled(enabled: Boolean) {
        viewModelScope.launch {
            settingsRepository.setMcpServerEnabled(enabled)
        }
    }

    fun setVolumeKeyMap(enabled: Boolean) {
        viewModelScope.launch {
            settingsRepository.setVolumeKeyMap(enabled)
        }
    }

    fun setThemeName(name: String) {
        viewModelScope.launch {
            settingsRepository.setThemeName(name)
            runtime.applySettings()
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

    fun setAppThemeMode(mode: String) {
        viewModelScope.launch {
            settingsRepository.setAppThemeMode(mode)
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
                    dragging = true,
                    start = anchor,
                    end = anchor,
                    mode = _state.value.selection.mode,
                ),
            )
        syncSelectionToNative()
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
        syncSelectionToNative()
    }

    fun updateSelectionStart(
        row: Int,
        col: Int,
    ) {
        val current = _state.value.selection
        if (!current.active) return
        _state.value =
            _state.value.copy(
                selection = current.copy(start = SelectionAnchor(row, col)),
            )
        syncSelectionToNative()
    }

    fun endSelection() {
        val current = _state.value.selection
        if (!current.active || current.start == null || current.end == null) return
        val text = extractSelectedText(current)
        _state.value =
            _state.value.copy(
                selection = current.copy(dragging = false, selectedText = text),
            )
        syncSelectionToNative()
    }

    fun setSelectionMode(mode: SelectionMode) {
        _state.value =
            _state.value.copy(
                selection = _state.value.selection.copy(mode = mode),
            )
    }

    fun copySelectionToClipboard() {
        val rawText = _state.value.selection.selectedText
        if (rawText.isEmpty()) return
        val text = if (rawText.length > 100_000) rawText.substring(0, 100_000) else rawText
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

    fun moveSelectionAnchor(
        moveEnd: Boolean,
        direction: Int,
    ) {
        val current = _state.value.selection
        if (!current.active) return
        val anchor = if (moveEnd) current.end else current.start ?: return
        if (anchor == null) return
        val newCol = anchor.col + direction
        val newAnchor =
            if (newCol < 0) {
                SelectionAnchor(maxOf(0, anchor.row - 1), 0)
            } else {
                SelectionAnchor(anchor.row, newCol)
            }
        _state.value =
            if (moveEnd) {
                val updatedSelection = current.copy(end = newAnchor)
                val text = extractSelectedText(updatedSelection)
                _state.value.copy(selection = updatedSelection.copy(selectedText = text))
            } else {
                _state.value.copy(selection = current.copy(start = newAnchor))
            }
        syncSelectionToNative()
    }

    fun clearSelection() {
        _state.value =
            _state.value.copy(
                selection = SelectionState(),
            )
        syncSelectionToNative()
    }

    fun showPastePopup(
        row: Int,
        col: Int,
    ) {
        _state.value =
            _state.value.copy(
                pastePopupRequest = PastePopupRequest(row, col),
            )
    }

    fun consumePastePopupRequest(): PastePopupRequest? {
        val req = _state.value.pastePopupRequest ?: return null
        _state.value = _state.value.copy(pastePopupRequest = null)
        return req
    }

    private fun syncSelectionToNative() {
        val sel = _state.value.selection
        if (sel.active && sel.start != null && sel.end != null) {
            val start = sel.start
            val end = sel.end
            val loRow = minOf(start.row, end.row)
            val hiRow = maxOf(start.row, end.row)
            val loCol: Int
            val hiCol: Int
            if (start.row <= end.row) {
                loCol = start.col
                hiCol = end.col
            } else {
                loCol = end.col
                hiCol = start.col
            }
            runtime.setSelection(loRow.toUInt(), loCol.toUInt(), hiRow.toUInt(), hiCol.toUInt(), true)
        } else {
            runtime.setSelection(0u, 0u, 0u, 0u, false)
        }
    }

    fun shareSelection() {
        val text = _state.value.selection.selectedText
        if (text.isEmpty()) return
        val sendIntent =
            Intent(Intent.ACTION_SEND).apply {
                type = "text/plain"
                putExtra(Intent.EXTRA_TEXT, text)
            }
        val shareIntent = Intent.createChooser(sendIntent, null)
        shareIntent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        context.startActivity(shareIntent)
    }

    fun selectAll() {
        val bridge = runtime.bridge() ?: return
        val cfg = runtime.state.value
        val rows = cfg.rows.coerceAtLeast(1)
        val cols = cfg.cols.coerceAtLeast(1)
        val scrollbackLength = bridge.scrollbackLength().toInt()
        var lastNonEmptyRow = 0
        for (r in 0 until rows) {
            val line = bridge.scrollbackLine((scrollbackLength + r).toUInt()) ?: ""
            if (line.isNotBlank()) {
                lastNonEmptyRow = r
            }
        }
        val start = SelectionAnchor(row = 0, col = 0)
        val end = SelectionAnchor(row = lastNonEmptyRow, col = cols - 1)
        val sel =
            SelectionState(
                active = true,
                dragging = false,
                start = start,
                end = end,
                mode = SelectionMode.Char,
            )
        val text = extractSelectedText(sel)
        _state.value = _state.value.copy(selection = sel.copy(selectedText = text))
        syncSelectionToNative()
    }

    private fun extractSelectedText(selection: SelectionState): String {
        val start = selection.start ?: return ""
        val end = selection.end ?: return ""
        val bridge = runtime.bridge() ?: return ""
        val scrollbackLength = bridge.scrollbackLength().toInt()
        val visibleCols =
            runtime.state.value.cols
                .coerceAtLeast(1)
        val (lo, hi) =
            if (start.row < end.row || (start.row == end.row && start.col <= end.col)) {
                start to end
            } else {
                end to start
            }
        return when (selection.mode) {
            SelectionMode.Char, SelectionMode.Word -> {
                if (lo.row == hi.row) {
                    val line = bridge.scrollbackLine((scrollbackLength + lo.row).toUInt()) ?: ""
                    val visLine = if (line.length > visibleCols) line.substring(0, visibleCols) else line
                    visLine.substring(lo.col.coerceAtMost(visLine.length), hi.col.coerceAtMost(visLine.length))
                } else {
                    val parts = mutableListOf<String>()
                    for (r in lo.row..hi.row) {
                        val line = bridge.scrollbackLine((scrollbackLength + r).toUInt()) ?: ""
                        val visLine = if (line.length > visibleCols) line.substring(0, visibleCols) else line
                        val startCol = if (r == lo.row) lo.col else 0
                        val endCol = if (r == hi.row) hi.col.coerceAtMost(visLine.length) else visLine.length
                        if (startCol < visLine.length) {
                            parts.add(visLine.substring(startCol, endCol.coerceAtMost(visLine.length)))
                        }
                    }
                    smartJoinLines(parts)
                }
            }

            SelectionMode.Line -> {
                val parts = mutableListOf<String>()
                for (r in lo.row..hi.row) {
                    val line = bridge.scrollbackLine((scrollbackLength + r).toUInt()) ?: ""
                    val visLine = if (line.length > visibleCols) line.substring(0, visibleCols) else line
                    parts.add(visLine)
                }
                parts.joinToString("\n")
            }

            SelectionMode.Block -> {
                val parts = mutableListOf<String>()
                for (r in lo.row..hi.row) {
                    val line = bridge.scrollbackLine((scrollbackLength + r).toUInt()) ?: ""
                    val visLine = if (line.length > visibleCols) line.substring(0, visibleCols) else line
                    val startCol = lo.col.coerceAtMost(visLine.length)
                    val endCol = hi.col.coerceAtMost(visLine.length)
                    if (startCol < visLine.length) {
                        parts.add(visLine.substring(startCol, endCol))
                    }
                }
                parts.joinToString("\n")
            }

            SelectionMode.Semantic -> {
                val parts = mutableListOf<String>()
                for (r in lo.row..hi.row) {
                    val line = bridge.scrollbackLine((scrollbackLength + r).toUInt()) ?: ""
                    val visLine = if (line.length > visibleCols) line.substring(0, visibleCols) else line
                    parts.add(visLine)
                }
                parts.joinToString("\n")
            }
        }
    }

    private fun smartJoinLines(parts: List<String>): String {
        if (parts.size <= 1) return parts.joinToString("")
        val result = StringBuilder(parts[0])
        for (index in 1 until parts.size) {
            val previousLine = parts[index - 1]
            val currentLine = parts[index]
            if (isContinuationUrl(previousLine)) {
                result.append(currentLine)
            } else if (isUrlStart(currentLine)) {
                result.append("\n").append(currentLine)
            } else if (isPathOrProtocol(currentLine)) {
                result.append(currentLine)
            } else if (isTuiBorder(currentLine)) {
                break
            } else if (shouldJoinWithNewline(previousLine, currentLine)) {
                result.append("\n").append(currentLine)
            } else {
                result.append(currentLine)
            }
        }
        return result.toString()
    }

    private fun isContinuationUrl(line: String): Boolean = line.endsWith("https://") || line.endsWith("http://")

    private fun isUrlStart(line: String): Boolean = line.startsWith("https://") || line.startsWith("http://")

    private fun isPathOrProtocol(line: String): Boolean = line.startsWith("/") || line.startsWith("http")

    private fun shouldJoinWithNewline(
        previousLine: String,
        currentLine: String,
    ): Boolean {
        if (previousLine.isBlank() || currentLine.isBlank()) return false
        if (currentLine.startsWith(" ")) return false
        if (previousLine.endsWith(" ")) return false
        return true
    }

    private fun isTuiBorder(line: String): Boolean {
        val trimmed = line.trim()
        if (trimmed.isEmpty()) return false
        val uniqueChars = trimmed.toSet().size
        if (uniqueChars <= 2 && trimmed.all { it in "│─╭╮╰╯┌┐└┘┬┴├┤┼═║╗╝╚╔╠╣╦╩╬ " }) {
            return true
        }
        return false
    }

    fun pasteFromClipboard(): Int {
        val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
        if (!clipboard.hasPrimaryClip()) return 0
        val clip = clipboard.primaryClip?.getItemAt(0)?.text ?: return 0
        val text = clip.toString()
        val data = text.replace("\n", "\r").toByteArray()
        runtime.writeToPty(data)
        return text.length
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

    fun cycleCtrlState() {
        _state.value = _state.value.copy(ctrlState = _state.value.ctrlState.next())
    }

    fun cycleAltState() {
        _state.value = _state.value.copy(altState = _state.value.altState.next())
    }

    fun consumeOneShotModifiers() {
        val currentState = _state.value
        var newCtrl = currentState.ctrlState
        var newAlt = currentState.altState
        if (newCtrl == ModifierState.Once) newCtrl = ModifierState.Off
        if (newAlt == ModifierState.Once) newAlt = ModifierState.Off
        if (newCtrl != currentState.ctrlState || newAlt != currentState.altState) {
            _state.value = currentState.copy(ctrlState = newCtrl, altState = newAlt)
        }
    }

    fun setScrollActive(active: Boolean) {
        _state.value = _state.value.copy(scrollActive = active)
    }

    fun toggleScrollMode() {
        _state.value = _state.value.copy(scrollActive = !_state.value.scrollActive)
    }

    fun createSession() {
        val surface = currentSurface
        if (surface == null || !surface.isValid) {
            android.util.Log.e("TerminalViewModel", "createSession: surface null or invalid, currentSurface=$currentSurface")
            return
        }
        val surfaceWidthPixels = surfaceWidth
        val surfaceHeightPixels = surfaceHeight
        if (surfaceWidthPixels <= 0 || surfaceHeightPixels <= 0) {
            android.util.Log.e("TerminalViewModel", "createSession: invalid dimensions ${surfaceWidthPixels}x$surfaceHeightPixels")
            return
        }

        viewModelScope.launch {
            val currentSurfaceNow = currentSurface
            if (currentSurfaceNow == null || !currentSurfaceNow.isValid) {
                android.util.Log.e("TerminalViewModel", "createSession: surface became invalid before launch")
                return@launch
            }
            try {
                val newId = runtime.createSession(currentSurfaceNow, surfaceWidthPixels, surfaceHeightPixels)
                if (newId > 0) {
                    val sortedIds = (_state.value.sessions.map { it.id } + newId).sorted()
                    val displayIndex = sortedIds.indexOf(newId) + 1
                    val sessions =
                        sortedIds.mapIndexed { index, id ->
                            SessionInfo(id = id, title = "Session ${index + 1}")
                        }
                    _state.value =
                        _state.value.copy(
                            sessionId = newId,
                            isRunning = true,
                            title = "Session $displayIndex",
                            selection = SelectionState(),
                            pendingInput = null,
                            sessions = sessions,
                            activeSessionId = newId,
                        )
                } else {
                    android.util.Log.e("TerminalViewModel", "createSession: runtime returned invalid id=$newId")
                }
            } catch (exception: Exception) {
                android.util.Log.e("TerminalViewModel", "createSession failed", exception)
            }
        }
    }

    fun switchSession(id: Long) {
        val surface = currentSurface
        if (surface == null || !surface.isValid) {
            android.util.Log.e("TerminalViewModel", "switchSession: surface null or invalid, currentSurface=$currentSurface")
            return
        }
        val surfaceWidthPixels = surfaceWidth
        val surfaceHeightPixels = surfaceHeight
        if (surfaceWidthPixels == 0 || surfaceHeightPixels == 0) {
            android.util.Log.e("TerminalViewModel", "switchSession: invalid dimensions ${surfaceWidthPixels}x$surfaceHeightPixels")
            return
        }

        try {
            runtime.switchSession(id, surface, surfaceWidthPixels, surfaceHeightPixels)
        } catch (exception: Exception) {
            android.util.Log.e("TerminalViewModel", "switchSession failed for id=$id", exception)
            return
        }
        _state.value =
            _state.value.copy(
                sessionId = id,
                isRunning = true,
                title =
                runtime.state.value.title
                    .ifEmpty {
                        val sortedIds =
                            _state.value.sessions
                                .map { it.id }
                                .sorted()
                        "Session ${sortedIds.indexOf(id) + 1}"
                    },
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
            val renumbered =
                remaining.sortedBy { it.id }.mapIndexed { index, session ->
                    session.copy(title = "Session ${index + 1}")
                }
            val newActive =
                if (current.activeSessionId == id) {
                    remaining.last().id
                } else {
                    current.activeSessionId
                }
            val newActiveIndex = renumbered.indexOfFirst { it.id == newActive }
            _state.value =
                current.copy(
                    sessions = renumbered,
                    activeSessionId = newActive,
                    sessionId = newActive,
                    title =
                    runtime.state.value.title
                        .ifEmpty { "Session ${newActiveIndex + 1}" },
                    selection = SelectionState(),
                    pendingInput = null,
                )
        }
    }

    fun setSessionTitle(title: String) {
        _state.value = _state.value.copy(title = title)
    }

    override fun onCleared() {
        viewModelScope.launch { runtime.saveSession() }
        super.onCleared()
    }
}
