package io.torvox.bridge

import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer

// ── Wire encoding helpers ──────────────────────────────────────────────

internal class WireWriter {
    private val buf = mutableListOf<Byte>()

    fun writeByte(v: Byte) {
        buf.add(v)
    }

    fun writeI32(v: Int) {
        buf.add((v and 0xFF).toByte())
        buf.add(((v shr 8) and 0xFF).toByte())
        buf.add(((v shr 16) and 0xFF).toByte())
        buf.add(((v shr 24) and 0xFF).toByte())
    }

    fun writeU32(v: UInt) = writeI32(v.toInt())

    fun writeString(v: String) {
        val bytes = v.toByteArray(Charsets.UTF_8)
        writeI32(bytes.size)
        buf.addAll(bytes.toList())
    }

    fun toByteArray(): ByteArray = buf.toByteArray()
}

class WireReader(
    data: ByteArray,
) {
    private val data = data
    private var pos = 0

    fun readBool(): Boolean {
        val v = data[pos]
        pos += 1
        return v != 0.toByte()
    }

    fun readByte(): Byte {
        val v = data[pos]
        pos += 1
        return v
    }

    fun readI32(): Int {
        val v =
            (data[pos].toInt() and 0xFF) or ((data[pos + 1].toInt() and 0xFF) shl 8) or
                ((data[pos + 2].toInt() and 0xFF) shl 16) or ((data[pos + 3].toInt() and 0xFF) shl 24)
        pos += 4
        return v
    }

    fun readU32(): UInt = readI32().toUInt()

    fun readString(): String {
        val len = readI32()
        if (len == 0) return ""
        val bytes = data.copyOfRange(pos, pos + len)
        pos += len
        return String(bytes, Charsets.UTF_8)
    }

    fun <T> readOptional(reader: (WireReader) -> T): T? {
        val tag = readByte()
        return if (tag != 0.toByte()) reader(this) else null
    }

    fun <T> readList(reader: (WireReader) -> T): List<T> {
        val len = readI32()
        return (0 until len).map { reader(this) }
    }
}

// ── Data types matching Rust #[boltffi::data] ─────────────────────────

sealed class Shell {
    object SystemDefault : Shell()

    data class Custom(
        val path: String,
    ) : Shell()

    internal fun wireEncode(w: WireWriter) {
        when (this) {
            is SystemDefault -> {
                w.writeI32(0)
            }

            is Custom -> {
                w.writeI32(1)
                w.writeString(path)
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
    internal fun wireEncode(w: WireWriter) {
        w.writeString(name)
        w.writeI32(bg)
        w.writeI32(fg)
        w.writeI32(cursor)
        w.writeI32(selectionBg)
        w.writeI32(ansi0)
        w.writeI32(ansi1)
        w.writeI32(ansi2)
        w.writeI32(ansi3)
        w.writeI32(ansi4)
        w.writeI32(ansi5)
        w.writeI32(ansi6)
        w.writeI32(ansi7)
        w.writeI32(ansi8)
        w.writeI32(ansi9)
        w.writeI32(ansi10)
        w.writeI32(ansi11)
        w.writeI32(ansi12)
        w.writeI32(ansi13)
        w.writeI32(ansi14)
        w.writeI32(ansi15)
    }

    fun wireEncodeBytes(): ByteArray {
        val w = WireWriter()
        wireEncode(w)
        return w.toByteArray()
    }

    companion object {
        fun wireDecode(r: WireReader): BridgeTheme =
            BridgeTheme(
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
) {
    fun wireEncode(): ByteArray {
        val w = WireWriter()
        shell.wireEncode(w)
        w.writeU32(rows)
        w.writeU32(cols)
        w.writeU32(scrollbackLines)
        w.writeU32(font_size_tenths)
        theme.wireEncode(w)
        return w.toByteArray()
    }

    companion object {
        fun wireDecode(r: WireReader): TerminalConfig =
            TerminalConfig(
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
            )
    }
}

// ── JNA native interface ──────────────────────────────────────────────

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

    fun torvox_bridge_spawn_terminal(
        handle: Long,
        rows: Int,
        cols: Int,
    ): Int

    fun torvox_bridge_release_surface(handle: Long)

    fun boltffi_torvox_bridge_get_config(handle: Long): Pointer?

    fun boltffi_torvox_bridge_get_theme_names(handle: Long): Pointer?

    fun boltffi_torvox_bridge_list_fonts(handle: Long): Pointer?

    fun boltffi_torvox_bridge_get_theme(
        handle: Long,
        name: String?,
    ): Pointer?

    // Raw C-ABI wrappers
    fun torvox_bridge_render(handle: Long): Int

    fun torvox_bridge_render_software(handle: Long): Int

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

private fun ensureLib(): TorvoxNative =
    nativeLib ?: synchronized(libLock) {
        nativeLib ?: Native.load("torvox_android", TorvoxNative::class.java).also { nativeLib = it }
    }

// ── FfiBuf reader ─────────────────────────────────────────────────────

private data class FfiBuf(
    val ptr: Long,
    val len: Int,
)

private fun readFfiBuf(p: Pointer?): FfiBuf {
    if (p == null) return FfiBuf(0, 0)
    return FfiBuf(p.getLong(0), p.getLong(8).toInt())
}

private fun readWireBytes(buf: FfiBuf): ByteArray {
    if (buf.ptr == 0L || buf.len == 0) return ByteArray(0)
    return Pointer(buf.ptr).getByteArray(0, buf.len)
}

// ── TorvoxBridge ──────────────────────────────────────────────────────

class TorvoxBridge(
    private val handle: Long,
) : AutoCloseable {
    private var closed = false

    private fun <T> callOk(
        fn: (TorvoxNative) -> Pointer?,
        decode: (WireReader) -> T,
    ): T {
        val lib = ensureLib()
        val p = fn(lib) ?: throw RuntimeException("Native call returned null")
        val buf = readFfiBuf(p)
        if (buf.ptr == 0L) throw RuntimeException("Empty response from native call")
        val r = WireReader(readWireBytes(buf))
        val tag = r.readByte()
        return if (tag == 0.toByte()) decode(r) else throw RuntimeException(r.readString())
    }

    private fun callUnit(fn: (TorvoxNative) -> Pointer?) {
        callOk(fn) { }
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
        windowPtr: Long,
        width: Int,
        height: Int,
    ) {
        val lib = ensureLib()
        val low = (windowPtr and 0xFFFFFFFFL).toInt()
        val high = ((windowPtr shr 32) and 0xFFFFFFFFL).toInt()
        val ret = lib.torvox_bridge_set_native_window(handle, low, high, width, height)
        if (ret != 0) throw RuntimeException("setNativeWindow failed with code $ret")
    }

    fun render(): Int = ensureLib().torvox_bridge_render(handle)

    fun renderSoftware(): Int = ensureLib().torvox_bridge_render_software(handle)

    fun resize(
        rows: UInt,
        cols: UInt,
    ) {
        val lib = ensureLib()
        val ret = lib.torvox_bridge_resize(handle, rows.toInt(), cols.toInt())
        if (ret != 0) throw RuntimeException("resize failed with code $ret")
    }

    fun releaseSurface() {
        val lib = ensureLib()
        lib.torvox_bridge_release_surface(handle)
    }

    fun scrollbackLen(): UInt = ensureLib().torvox_bridge_scrollback_len(handle).toUInt()

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
        val p = ensureLib().boltffi_torvox_bridge_get_theme_names(handle) ?: return emptyList()
        return WireReader(readWireBytes(readFfiBuf(p))).readList { it.readString() }
    }

    fun listFonts(): List<String> {
        val p = ensureLib().boltffi_torvox_bridge_list_fonts(handle) ?: return emptyList()
        return WireReader(readWireBytes(readFfiBuf(p))).readList { it.readString() }
    }

    fun saveSession(path: String) {
        val bytes = path.toByteArray(Charsets.UTF_8)
        ensureLib().torvox_bridge_save_session(handle, bytes, bytes.size)
    }

    fun restoreSession(path: String) {
        val bytes = path.toByteArray(Charsets.UTF_8)
        ensureLib().torvox_bridge_restore_session(handle, bytes, bytes.size)
    }

    fun hasSavedSession(path: String): Boolean {
        val bytes = path.toByteArray(Charsets.UTF_8)
        return ensureLib().torvox_bridge_has_saved_session(handle, bytes, bytes.size)
    }

    fun scrollbackLine(index: UInt): String? {
        val ptr = ensureLib().torvox_bridge_scrollback_line(handle, index.toInt())
        if (ptr == 0L) return null
        val s = Pointer(ptr).getString(0, "UTF-8")
        ensureLib().torvox_bridge_free_string(ptr)
        return s
    }

    fun getTerminalText(): String? {
        val ptr = ensureLib().torvox_bridge_get_terminal_text(handle)
        if (ptr == 0L) return null
        val s = Pointer(ptr).getString(0, "UTF-8")
        ensureLib().torvox_bridge_free_string(ptr)
        return s
    }

    fun searchInScrollback(query: String): Pair<Int, Int>? {
        val queryBytes = query.toByteArray(Charsets.UTF_8)
        val ptr = ensureLib().torvox_bridge_search_in_scrollback(handle, queryBytes, queryBytes.size)
        if (ptr == 0L) return null
        val s = Pointer(ptr).getString(0, "UTF-8")
        ensureLib().torvox_bridge_free_string(ptr)
        val parts = s.split(',')
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
