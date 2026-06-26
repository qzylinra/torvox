@file:Suppress("ktlint:standard:function-naming")

// @TorvoxBridge JNA binding, IMPL_ANDR_KT_001, impl, [REQ_ANDR_001]
// @need-ids: REQ_ANDR_001, REQ_ANDR_003

package io.torvox.bridge

import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer

// ── Wire encoding helpers ──────────────────────────────────────────────

internal class WireWriter {
    private val buffer = mutableListOf<Byte>()

    fun writeByte(v: Byte) {
        buffer.add(v)
    }

    fun writeI32(v: Int) {
        buffer.add((v and 0xFF).toByte())
        buffer.add(((v shr 8) and 0xFF).toByte())
        buffer.add(((v shr 16) and 0xFF).toByte())
        buffer.add(((v shr 24) and 0xFF).toByte())
    }

    fun writeU32(v: UInt) = writeI32(v.toInt())

    fun writeString(v: String) {
        val bytes = v.toByteArray(Charsets.UTF_8)
        writeI32(bytes.size)
        buffer.addAll(bytes.toList())
    }

    fun toByteArray(): ByteArray = buffer.toByteArray()
}

class WireReader(
    data: ByteArray,
) {
    private val data = data
    private var position = 0

    fun readBool(): Boolean {
        val value = data[position]
        position += 1
        return value != 0.toByte()
    }

    fun readByte(): Byte {
        val value = data[position]
        position += 1
        return value
    }

    fun readI32(): Int {
        val value =
            (data[position].toInt() and 0xFF) or ((data[position + 1].toInt() and 0xFF) shl 8) or
                ((data[position + 2].toInt() and 0xFF) shl 16) or ((data[position + 3].toInt() and 0xFF) shl 24)
        position += 4
        return value
    }

    fun readU32(): UInt = readI32().toUInt()

    fun readString(): String {
        val length = readI32()
        if (length == 0) return ""
        val bytes = data.copyOfRange(position, position + length)
        position += length
        return String(bytes, Charsets.UTF_8)
    }

    fun <T> readOptional(reader: (WireReader) -> T): T? {
        val tag = readByte()
        return if (tag != 0.toByte()) reader(this) else null
    }

    fun <T> readList(reader: (WireReader) -> T): List<T> {
        val length = readI32()
        return (0 until length).map { reader(this) }
    }
}

// ── Data types matching Rust #[boltffi::data] ─────────────────────────

sealed class Shell {
    object SystemDefault : Shell()

    data class Custom(
        val path: String,
    ) : Shell()

    internal fun wireEncode(writer: WireWriter) {
        when (this) {
            is SystemDefault -> {
                writer.writeI32(0)
            }

            is Custom -> {
                writer.writeI32(1)
                writer.writeString(path)
            }
        }
    }
}

data class BridgeTheme(
    val name: String = "",
    val bg: Int = 0,
    val fg: Int = 0,
    val cursor: Int = 0,
    val selectionBg: Int = 0,
    val ansi0: Int = 0,
    val ansi1: Int = 0,
    val ansi2: Int = 0,
    val ansi3: Int = 0,
    val ansi4: Int = 0,
    val ansi5: Int = 0,
    val ansi6: Int = 0,
    val ansi7: Int = 0,
    val ansi8: Int = 0,
    val ansi9: Int = 0,
    val ansi10: Int = 0,
    val ansi11: Int = 0,
    val ansi12: Int = 0,
    val ansi13: Int = 0,
    val ansi14: Int = 0,
    val ansi15: Int = 0,
) {
    internal fun wireEncode(writer: WireWriter) {
        writer.writeString(name)
        writer.writeI32(bg)
        writer.writeI32(fg)
        writer.writeI32(cursor)
        writer.writeI32(selectionBg)
        writer.writeI32(ansi0)
        writer.writeI32(ansi1)
        writer.writeI32(ansi2)
        writer.writeI32(ansi3)
        writer.writeI32(ansi4)
        writer.writeI32(ansi5)
        writer.writeI32(ansi6)
        writer.writeI32(ansi7)
        writer.writeI32(ansi8)
        writer.writeI32(ansi9)
        writer.writeI32(ansi10)
        writer.writeI32(ansi11)
        writer.writeI32(ansi12)
        writer.writeI32(ansi13)
        writer.writeI32(ansi14)
        writer.writeI32(ansi15)
    }

    fun wireEncodeBytes(): ByteArray {
        val writer = WireWriter()
        wireEncode(writer)
        return writer.toByteArray()
    }

    companion object {
        @Suppress("FunctionNaming")
        fun wireDecode(r: WireReader): BridgeTheme = BridgeTheme(
            name = r.readString(),
            bg = r.readI32(),
            fg = r.readI32(),
            cursor = r.readI32(),
            selectionBg = r.readI32(),
            ansi0 = r.readI32(),
            ansi1 = r.readI32(),
            ansi2 = r.readI32(),
            ansi3 = r.readI32(),
            ansi4 = r.readI32(),
            ansi5 = r.readI32(),
            ansi6 = r.readI32(),
            ansi7 = r.readI32(),
            ansi8 = r.readI32(),
            ansi9 = r.readI32(),
            ansi10 = r.readI32(),
            ansi11 = r.readI32(),
            ansi12 = r.readI32(),
            ansi13 = r.readI32(),
            ansi14 = r.readI32(),
            ansi15 = r.readI32(),
        )
    }
}

data class TerminalConfig(
    val shell: Shell = Shell.SystemDefault,
    val rows: UInt = 24u,
    val cols: UInt = 80u,
    val scrollbackLines: UInt = 50000u,
    val font_size_tenths: UInt = 140u,
    val theme: BridgeTheme = BridgeTheme(),
    val home: String = "",
    val user: String = "",
    val path: String = "",
    val workingDirectory: String = "",
    val prefix: String = "",
) {
    fun wireEncode(): ByteArray {
        val writer = WireWriter()
        shell.wireEncode(writer)
        writer.writeU32(rows)
        writer.writeU32(cols)
        writer.writeU32(scrollbackLines)
        writer.writeU32(font_size_tenths)
        theme.wireEncode(writer)
        writer.writeString(home)
        writer.writeString(user)
        writer.writeString(path)
        writer.writeString(workingDirectory)
        writer.writeString(prefix)
        return writer.toByteArray()
    }

    companion object {
        @Suppress("FunctionNaming")
        fun wireDecode(r: WireReader): TerminalConfig = TerminalConfig(
            shell =
            when (r.readI32()) {
                0 -> Shell.SystemDefault
                1 -> Shell.Custom(r.readString())
                else -> Shell.SystemDefault
            },
            rows = r.readU32(),
            cols = r.readU32(),
            scrollbackLines = r.readU32(),
            font_size_tenths = r.readU32(),
            theme = BridgeTheme.wireDecode(r),
            home = r.readString(),
            user = r.readString(),
            path = r.readString(),
            workingDirectory = r.readString(),
            prefix = r.readString(),
        )
    }
}

// ── JNA native interface ──────────────────────────────────────────────

@Suppress("FunctionNaming")
private interface TorvoxNative : Library {
    fun boltffi_torvox_bridge_new(
        config_ptr: ByteArray?,
        config_len: Int,
    ): Long

    fun boltffi_torvox_bridge_free(handle: Long)

    fun torvox_bridge_ping(handle: Long): Int

    // Raw C-ABI wrappers for methods with scalar parameters
    fun torvox_bridge_set_native_window(
        handle: Long,
        window_ptr_low: Int,
        window_ptr_high: Int,
        width: Int,
        height: Int,
    ): Int

    fun torvox_bridge_resize(
        handle: Long,
        rows: Int,
        cols: Int,
    ): Int

    fun torvox_bridge_recompute_grid(
        handle: Long,
        width: Int,
        height: Int,
    ): Int

    fun torvox_bridge_update_native_window(
        handle: Long,
        window_ptr_low: Int,
        window_ptr_high: Int,
        width: Int,
        height: Int,
    ): Int

    fun torvox_bridge_spawn_terminal(
        handle: Long,
        rows: Int,
        cols: Int,
    ): Int

    fun torvox_bridge_release_surface(handle: Long)

    fun torvox_bridge_release_gpu_surface(handle: Long)

    fun boltffi_torvox_bridge_get_config(handle: Long): Pointer?

    fun boltffi_torvox_bridge_get_theme_names(handle: Long): Pointer?

    fun boltffi_torvox_bridge_list_fonts(handle: Long): Pointer?

    fun boltffi_torvox_bridge_get_theme(
        handle: Long,
        name: String?,
    ): Pointer?

    // Raw C-ABI wrappers
    fun torvox_bridge_render(handle: Long): Int

    fun torvox_bridge_poll_bel(handle: Long): Int

    fun torvox_bridge_save_test_frame(
        handle: Long,
        dataDir: Pointer,
    ): Int

    fun torvox_bridge_poll_clipboard(handle: Long): Long

    fun torvox_bridge_poll_notification(handle: Long): Long

    fun torvox_bridge_poll_shell_integration(handle: Long): Int

    fun torvox_bridge_poll_sync_active(handle: Long): Int

    fun torvox_bridge_cwd(handle: Long): Pointer?

    fun torvox_bridge_free_cstring(s: Pointer?)

    fun torvox_bridge_focus_event(
        handle: Long,
        focused: Int,
    )

    fun torvox_bridge_scrollback_len(handle: Long): Long

    // Raw C-ABI wrappers for string/byte-array/scalar methods

    fun torvox_bridge_set_save_path(
        handle: Long,
        path_ptr: ByteArray?,
        path_len: Int,
    ): Int

    fun torvox_bridge_has_saved_session(
        handle: Long,
        path_ptr: ByteArray?,
        path_len: Int,
    ): Boolean

    fun torvox_bridge_save_session(
        handle: Long,
        path_ptr: ByteArray?,
        path_len: Int,
    ): Int

    fun torvox_bridge_restore_session(
        handle: Long,
        path_ptr: ByteArray?,
        path_len: Int,
    ): Int

    fun torvox_bridge_write_to_pty(
        handle: Long,
        data_ptr: ByteArray?,
        data_len: Int,
    ): Int

    fun torvox_bridge_set_font_size(
        handle: Long,
        size_tenths: Int,
    ): Int

    @Suppress("LongParameterList", "FunctionParameterNaming")
    fun torvox_bridge_set_selection(
        handle: Long,
        start_row: Int,
        start_col: Int,
        end_row: Int,
        end_col: Int,
        active: Int,
    ): Int

    fun torvox_bridge_set_font_family(
        handle: Long,
        family_ptr: ByteArray?,
        family_len: Int,
    ): Int

    fun torvox_bridge_set_theme(
        handle: Long,
        theme_ptr: ByteArray?,
        theme_len: Int,
    ): Int

    fun torvox_bridge_scrollback_line(
        handle: Long,
        index: Int,
    ): Long

    fun torvox_bridge_get_terminal_text(handle: Long): Long

    fun torvox_bridge_get_active_session_title(handle: Long): Long

    fun torvox_bridge_get_grid_rows(handle: Long): Int

    fun torvox_bridge_get_grid_cols(handle: Long): Int

    fun torvox_bridge_free_string(s: Long)

    fun torvox_bridge_search_in_scrollback(
        handle: Long,
        query_ptr: ByteArray?,
        query_len: Int,
    ): Long

    fun boltffi_last_error_message(): ByteArray?
}

private var nativeLib: TorvoxNative? = null
private val libLock = Any()

private fun ensureLib(): TorvoxNative = nativeLib ?: synchronized(libLock) {
    nativeLib ?: Native.load("torvox_android", TorvoxNative::class.java).also { nativeLib = it }
}

// ── FfiBuf reader ─────────────────────────────────────────────────────

private data class FfiBuf(
    val pointer: Long,
    val length: Int,
)

private fun readFfiBuf(pointer: Pointer?): FfiBuf {
    if (pointer == null) return FfiBuf(0, 0)
    return FfiBuf(pointer.getLong(0), pointer.getLong(8).toInt())
}

private fun readWireBytes(buffer: FfiBuf): ByteArray {
    if (buffer.pointer == 0L || buffer.length == 0) return ByteArray(0)
    return Pointer(buffer.pointer).getByteArray(0, buffer.length)
}

// ── TorvoxBridge ──────────────────────────────────────────────────────

class TorvoxBridge(
    private val handle: Long,
) : AutoCloseable {
    private var closed = false

    private fun <T> callOk(
        functionPointer: (TorvoxNative) -> Pointer?,
        decode: (WireReader) -> T,
    ): T {
        val lib = ensureLib()
        val pointer = functionPointer(lib) ?: throw RuntimeException("Native call returned null")
        val buffer = readFfiBuf(pointer)
        if (buffer.pointer == 0L) throw RuntimeException("Empty response from native call")
        val reader = WireReader(readWireBytes(buffer))
        val tag = reader.readByte()
        return if (tag == 0.toByte()) decode(reader) else throw RuntimeException(reader.readString())
    }

    private fun callUnit(functionPointer: (TorvoxNative) -> Pointer?) {
        callOk(functionPointer) { }
    }

    fun ping(): String {
        val lib = ensureLib()
        val result = lib.torvox_bridge_ping(handle)
        return if (result == 0) "pong" else "error:$result"
    }

    fun spawnTerminal(
        rows: UInt,
        cols: UInt,
    ): Int {
        val lib = ensureLib()
        return lib.torvox_bridge_spawn_terminal(handle, rows.toInt(), cols.toInt())
    }

    fun setNativeWindow(
        windowPointer: Long,
        width: Int,
        height: Int,
    ) {
        val lib = ensureLib()
        val low = (windowPointer and 0xFFFFFFFFL).toInt()
        val high = ((windowPointer shr 32) and 0xFFFFFFFFL).toInt()
        val ret = lib.torvox_bridge_set_native_window(handle, low, high, width, height)
        if (ret != 0) throw RuntimeException("setNativeWindow failed with code $ret")
    }

    fun render(): Int = ensureLib().torvox_bridge_render(handle)

    fun saveTestFrame(dataDir: String): Int {
        val cStr =
            com.sun.jna.Native
                .toByteArray(dataDir)
        val mem = com.sun.jna.Memory(cStr.size.toLong() + 1)
        mem.write(0, cStr, 0, cStr.size)
        mem.setByte(cStr.size.toLong(), 0)
        return ensureLib().torvox_bridge_save_test_frame(handle, mem)
    }

    fun pollBel(): Boolean = ensureLib().torvox_bridge_poll_bel(handle) != 0

    fun pollShellIntegration(): Int = ensureLib().torvox_bridge_poll_shell_integration(handle)

    fun pollSyncActive(): Boolean = ensureLib().torvox_bridge_poll_sync_active(handle) != 0

    fun pollClipboard(): String? {
        val pointer = ensureLib().torvox_bridge_poll_clipboard(handle)
        if (pointer == 0L) return null
        val text = Pointer(pointer).getString(0, "UTF-8")
        ensureLib().torvox_bridge_free_string(pointer)
        return text
    }

    fun pollNotification(): Pair<String, String>? {
        val pointer = ensureLib().torvox_bridge_poll_notification(handle)
        if (pointer == 0L) return null
        val combined = Pointer(pointer).getString(0, "UTF-8")
        ensureLib().torvox_bridge_free_string(pointer)
        val parts = combined.split("\u0000", limit = 2)
        return if (parts.size == 2) parts[0] to parts[1] else null
    }

    fun cwd(): String {
        val pointer = ensureLib().torvox_bridge_cwd(handle)
        if (pointer == null) return ""
        val str = pointer.getString(0, "UTF-8")
        ensureLib().torvox_bridge_free_cstring(pointer)
        return str
    }

    fun focusEvent(focused: Boolean) {
        ensureLib().torvox_bridge_focus_event(handle, if (focused) 1 else 0)
    }

    fun resize(
        rows: UInt,
        cols: UInt,
    ) {
        val lib = ensureLib()
        val ret = lib.torvox_bridge_resize(handle, rows.toInt(), cols.toInt())
        if (ret != 0) throw RuntimeException("resize failed with code $ret")
    }

    fun recomputeGrid(
        width: UInt,
        height: UInt,
    ) {
        val lib = ensureLib()
        val ret = lib.torvox_bridge_recompute_grid(handle, width.toInt(), height.toInt())
        if (ret != 0) throw RuntimeException("recomputeGrid failed with code $ret")
    }

    fun updateNativeWindow(
        windowPointer: Long,
        width: Int,
        height: Int,
    ) {
        val lib = ensureLib()
        val low = (windowPointer and 0xFFFFFFFFL).toInt()
        val high = ((windowPointer shr 32) and 0xFFFFFFFFL).toInt()
        val ret = lib.torvox_bridge_update_native_window(handle, low, high, width, height)
        if (ret != 0) throw RuntimeException("updateNativeWindow failed with code $ret")
    }

    fun releaseSurface() {
        val lib = ensureLib()
        lib.torvox_bridge_release_surface(handle)
    }

    fun releaseGpuSurface() {
        val lib = ensureLib()
        lib.torvox_bridge_release_gpu_surface(handle)
    }

    fun setSelection(
        startRow: UInt,
        startCol: UInt,
        endRow: UInt,
        endCol: UInt,
        active: Boolean,
    ) {
        val lib = ensureLib()
        lib.torvox_bridge_set_selection(handle, startRow.toInt(), startCol.toInt(), endRow.toInt(), endCol.toInt(), if (active) 1 else 0)
    }

    fun scrollbackLength(): UInt = ensureLib().torvox_bridge_scrollback_len(handle).toUInt()

    fun writeToPty(data: ByteArray) {
        ensureLib().torvox_bridge_write_to_pty(handle, data, data.size)
    }

    fun setFontSize(sizeTenths: UInt) {
        ensureLib().torvox_bridge_set_font_size(handle, sizeTenths.toInt())
    }

    fun setFontFamily(familyName: String) {
        val bytes = familyName.toByteArray(Charsets.UTF_8)
        val ret = ensureLib().torvox_bridge_set_font_family(handle, bytes, bytes.size)
        if (ret != 0) throw RuntimeException("setFontFamily failed with code $ret")
    }

    fun setTheme(theme: BridgeTheme) {
        val bytes = theme.wireEncodeBytes()
        ensureLib().torvox_bridge_set_theme(handle, bytes, bytes.size)
    }

    fun setSavePath(path: String) {
        val bytes = path.toByteArray(Charsets.UTF_8)
        ensureLib().torvox_bridge_set_save_path(handle, bytes, bytes.size)
    }

    fun getConfig(): TerminalConfig = callOk({ it.boltffi_torvox_bridge_get_config(handle) }) { TerminalConfig.wireDecode(it) }

    fun getThemeNames(): List<String> {
        val pointer = ensureLib().boltffi_torvox_bridge_get_theme_names(handle) ?: return emptyList()
        return WireReader(readWireBytes(readFfiBuf(pointer))).readList { it.readString() }
    }

    fun listFonts(): List<String> {
        val pointer = ensureLib().boltffi_torvox_bridge_list_fonts(handle) ?: return emptyList()
        return WireReader(readWireBytes(readFfiBuf(pointer))).readList { it.readString() }
    }

    fun saveSession(path: String) {
        val bytes = path.toByteArray(Charsets.UTF_8)
        val ret = ensureLib().torvox_bridge_save_session(handle, bytes, bytes.size)
        if (ret != 0) throw RuntimeException("saveSession failed with code $ret")
    }

    fun restoreSession(path: String) {
        val bytes = path.toByteArray(Charsets.UTF_8)
        val ret = ensureLib().torvox_bridge_restore_session(handle, bytes, bytes.size)
        if (ret != 0) throw RuntimeException("restoreSession failed with code $ret")
    }

    fun hasSavedSession(path: String): Boolean {
        val bytes = path.toByteArray(Charsets.UTF_8)
        return ensureLib().torvox_bridge_has_saved_session(handle, bytes, bytes.size)
    }

    fun scrollbackLine(index: UInt): String? {
        val pointer = ensureLib().torvox_bridge_scrollback_line(handle, index.toInt())
        if (pointer == 0L) return null
        val text = Pointer(pointer).getString(0, "UTF-8")
        ensureLib().torvox_bridge_free_string(pointer)
        return text
    }

    fun getTerminalText(): String? {
        val pointer = ensureLib().torvox_bridge_get_terminal_text(handle)
        if (pointer == 0L) return null
        val text = Pointer(pointer).getString(0, "UTF-8")
        ensureLib().torvox_bridge_free_string(pointer)
        return text
    }

    fun getActiveSessionTitle(): String {
        val pointer = ensureLib().torvox_bridge_get_active_session_title(handle)
        if (pointer == 0L) return ""
        val text = Pointer(pointer).getString(0, "UTF-8")
        ensureLib().torvox_bridge_free_string(pointer)
        return text
    }

    fun getGridRows(): Int = ensureLib().torvox_bridge_get_grid_rows(handle)

    fun getGridCols(): Int = ensureLib().torvox_bridge_get_grid_cols(handle)

    fun searchInScrollback(query: String): Pair<Int, Int>? {
        val queryBytes = query.toByteArray(Charsets.UTF_8)
        val pointer = ensureLib().torvox_bridge_search_in_scrollback(handle, queryBytes, queryBytes.size)
        if (pointer == 0L) return null
        val text = Pointer(pointer).getString(0, "UTF-8")
        ensureLib().torvox_bridge_free_string(pointer)
        val parts = text.split(',')
        return if (parts.size == 2) parts[0].toInt() to parts[1].toInt() else null
    }

    override fun close() {
        if (!closed) {
            closed = true
            nativeLib?.boltffi_torvox_bridge_free(handle)
        }
    }

    protected fun finalize() = close()
}

fun createBridge(config: TerminalConfig): TorvoxBridge {
    val lib = ensureLib()
    val wireBytes = config.wireEncode()
    val handle = lib.boltffi_torvox_bridge_new(wireBytes, wireBytes.size)
    if (handle == 0L) {
        val errMsg = lib.boltffi_last_error_message()?.toString(Charsets.UTF_8) ?: "unknown"
        throw RuntimeException("Failed to create TorvoxBridge: $errMsg")
    }
    return TorvoxBridge(handle)
}
