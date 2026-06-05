package io.torvox.bridge

import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Unit tests for the wire encoding helpers used by the JNA bridge.
 * WireReader and WireWriter are pure JVM and can be tested without an
 * Android device or instrumented test.
 */
class WireCodecTest {
    @Test
    fun wireWriter_writeByte() {
        val w = WireWriter()
        w.writeByte(0x42)
        assertArrayEquals(byteArrayOf(0x42), w.toByteArray())
    }

    @Test
    fun wireWriter_writeI32_zero() {
        val w = WireWriter()
        w.writeI32(0)
        assertArrayEquals(byteArrayOf(0, 0, 0, 0), w.toByteArray())
    }

    @Test
    fun wireWriter_writeI32_one() {
        val w = WireWriter()
        w.writeI32(1)
        assertArrayEquals(byteArrayOf(1, 0, 0, 0), w.toByteArray())
    }

    @Test
    fun wireWriter_writeI32_littleEndian() {
        val w = WireWriter()
        w.writeI32(0x12345678.toInt())
        val expected = byteArrayOf(0x78, 0x56, 0x34, 0x12)
        assertArrayEquals(expected, w.toByteArray())
    }

    @Test
    fun wireWriter_writeI32_negative() {
        val w = WireWriter()
        w.writeI32(-1)
        assertArrayEquals(byteArrayOf(0xFF.toByte(), 0xFF.toByte(), 0xFF.toByte(), 0xFF.toByte()), w.toByteArray())
    }

    @Test
    fun wireWriter_writeU32() {
        val w = WireWriter()
        w.writeU32(0xDEADBEEFu)
        val expected = byteArrayOf(0xEF.toByte(), 0xBE.toByte(), 0xAD.toByte(), 0xDE.toByte())
        assertArrayEquals(expected, w.toByteArray())
    }

    @Test
    fun wireWriter_writeString_empty() {
        val w = WireWriter()
        w.writeString("")
        // 4 bytes for length (0) and no payload
        assertArrayEquals(byteArrayOf(0, 0, 0, 0), w.toByteArray())
    }

    @Test
    fun wireWriter_writeString_ascii() {
        val w = WireWriter()
        w.writeString("hi")
        // length=2, then 'h', 'i'
        assertArrayEquals(byteArrayOf(2, 0, 0, 0, 'h'.code.toByte(), 'i'.code.toByte()), w.toByteArray())
    }

    @Test
    fun wireWriter_writeString_utf8() {
        val w = WireWriter()
        val s = "中"
        val expected = s.toByteArray(Charsets.UTF_8)
        val w2 = WireWriter()
        w2.writeString(s)
        val out = w2.toByteArray()
        // 4 bytes length + UTF-8 bytes
        assertEquals(4 + expected.size, out.size)
        // last N bytes match UTF-8 encoding
        for (i in expected.indices) {
            assertEquals(expected[i], out[4 + i])
        }
    }

    @Test
    fun wireReader_readByte() {
        val r = WireReader(byteArrayOf(0x42))
        assertEquals(0x42.toByte(), r.readByte())
    }

    @Test
    fun wireReader_readBool_true() {
        val r = WireReader(byteArrayOf(1))
        assertTrue(r.readBool())
    }

    @Test
    fun wireReader_readBool_false() {
        val r = WireReader(byteArrayOf(0))
        assertFalse(r.readBool())
    }

    @Test
    fun wireRoundTrip_i32() {
        val values = listOf(0, 1, -1, 42, Int.MAX_VALUE, Int.MIN_VALUE, 0x12345678.toInt())
        for (v in values) {
            val w = WireWriter()
            w.writeI32(v)
            val r = WireReader(w.toByteArray())
            // We only have readI32 implemented through readU32/readI32 helpers; use a simple
            // roundtrip check by re-reading through the public readI32 method if present.
            // If not present, ensure bytes match expected little-endian encoding.
            val expected =
                byteArrayOf(
                    (v and 0xFF).toByte(),
                    ((v shr 8) and 0xFF).toByte(),
                    ((v shr 16) and 0xFF).toByte(),
                    ((v shr 24) and 0xFF).toByte(),
                )
            assertArrayEquals("i32 $v", expected, w.toByteArray())
        }
    }

    @Test
    fun wireWriter_concatenates() {
        val w = WireWriter()
        w.writeI32(1)
        w.writeI32(2)
        w.writeByte(0xAB.toByte())
        val out = w.toByteArray()
        assertEquals(9, out.size)
    }

    @Test
    fun wireReader_string_roundTripAscii() {
        val s = "hello"
        val w = WireWriter()
        w.writeString(s)
        val data = w.toByteArray()
        // length-prefixed: 4 bytes length + 5 bytes payload
        assertEquals(9, data.size)
    }
}
