package io.torvox.bridge

import org.junit.Assert.assertEquals
import org.junit.Test

class BridgeDataTest {
    @Test
    fun argbFromBlack() {
        val a = 255
        val r = 0
        val g = 0
        val b = 0
        val packed = (a shl 24) or (r shl 16) or (g shl 8) or b
        assertEquals(0xFF000000.toInt(), packed)
    }

    @Test
    fun argbFromWhite() {
        val a = 255
        val r = 255
        val g = 255
        val b = 255
        val packed = (a shl 24) or (r shl 16) or (g shl 8) or b
        assertEquals(-1, packed)
    }

    @Test
    fun argbFromPrimaryColors() {
        val packedRed = (255 shl 24) or (255 shl 16)
        assertEquals(0xFFFF0000.toInt(), packedRed)

        val packedGreen = (255 shl 24) or (255 shl 8)
        assertEquals(0xFF00FF00.toInt(), packedGreen)

        val packedBlue = (255 shl 24) or 255
        assertEquals(0xFF0000FF.toInt(), packedBlue)
    }

    @Test
    fun argbComponentExtraction() {
        val color = 0xAABBCCDD.toInt()
        val alpha = (color ushr 24) and 0xFF
        val red = (color ushr 16) and 0xFF
        val green = (color ushr 8) and 0xFF
        val blue = color and 0xFF
        assertEquals(0xAA, alpha)
        assertEquals(0xBB, red)
        assertEquals(0xCC, green)
        assertEquals(0xDD, blue)
    }

    @Test
    fun argbFullyTransparent() {
        val a = 0
        val r = 128
        val g = 64
        val b = 32
        val packed = (a shl 24) or (r shl 16) or (g shl 8) or b
        val extractedA = (packed ushr 24) and 0xFF
        val extractedR = (packed ushr 16) and 0xFF
        val extractedG = (packed ushr 8) and 0xFF
        val extractedB = packed and 0xFF
        assertEquals(0, extractedA)
        assertEquals(128, extractedR)
        assertEquals(64, extractedG)
        assertEquals(32, extractedB)
    }

    @Test
    fun argbComponentPackingRoundTrip() {
        val components =
            listOf(
                intArrayOf(255, 255, 255, 255),
                intArrayOf(255, 0, 0, 0),
                intArrayOf(0, 255, 255, 255),
                intArrayOf(128, 64, 32, 16),
                intArrayOf(200, 100, 50, 25),
            )
        for ((a, r, g, b) in components) {
            val packed = (a shl 24) or (r shl 16) or (g shl 8) or b
            assertEquals(a, (packed ushr 24) and 0xFF)
            assertEquals(r, (packed ushr 16) and 0xFF)
            assertEquals(g, (packed ushr 8) and 0xFF)
            assertEquals(b, packed and 0xFF)
        }
    }

    @Test
    fun bridgeThemeColorValuesPreservedInWire() {
        val original =
            BridgeTheme(
                bg = 0xFF282A36.toInt(),
                fg = 0xFFF8F8F2.toInt(),
                cursor = 0xFFF8F8F2.toInt(),
                selectionBg = 0xFF44475A.toInt(),
            )
        val bytes = original.wireEncodeBytes()
        val decoded = BridgeTheme.wireDecode(WireReader(bytes))
        assertEquals(original.bg, decoded.bg)
        assertEquals(original.fg, decoded.fg)
        assertEquals(original.cursor, decoded.cursor)
        assertEquals(original.selectionBg, decoded.selectionBg)
    }

    @Test
    fun cursorPositionEncodingLow16Bits() {
        val startRow: UShort = 5u
        val startCol: UShort = 42u
        val lowPart = (startRow.toUInt()) or (startCol.toUInt() shl 16)
        assertEquals(5u, lowPart and 0xFFFFu)
        assertEquals(42u, (lowPart shr 16) and 0xFFFFu)
    }

    @Test
    fun cursorPositionEncodingHigh16Bits() {
        val endRow: UShort = 100u
        val endCol: UShort = 200u
        val highPart = (endRow.toULong()) or (endCol.toULong() shl 16)
        assertEquals(100uL, highPart and 0xFFFFuL)
        assertEquals(200uL, (highPart shr 16) and 0xFFFFuL)
    }

    @Test
    fun cursorPositionFull64BitEncoding() {
        val startRow = 5u
        val startCol = 42u
        val endRow = 100u
        val endCol = 200u
        val encoded =
            startRow.toLong() or
                (startCol.toLong() shl 16) or
                (endRow.toLong() shl 32) or
                (endCol.toLong() shl 48)
        val decodedStartRow = (encoded and 0xFFFFL).toUInt()
        val decodedStartCol = ((encoded shr 16) and 0xFFFFL).toUInt()
        val decodedEndRow = ((encoded shr 32) and 0xFFFFL).toUInt()
        val decodedEndCol = ((encoded shr 48) and 0xFFFFL).toUInt()
        assertEquals(5u, decodedStartRow)
        assertEquals(42u, decodedStartCol)
        assertEquals(100u, decodedEndRow)
        assertEquals(200u, decodedEndCol)
    }

    @Test
    fun cursorPositionMaxValues() {
        val startRow: UShort = 65535u
        val startCol: UShort = 65535u
        val endRow: UShort = 65535u
        val endCol: UShort = 65535u
        val encoded =
            startRow.toLong() or
                (startCol.toLong() shl 16) or
                (endRow.toLong() shl 32) or
                (endCol.toLong() shl 48)
        val decodedStartRow = (encoded and 0xFFFFL).toUShort()
        val decodedStartCol = ((encoded shr 16) and 0xFFFFL).toUShort()
        val decodedEndRow = ((encoded shr 32) and 0xFFFFL).toUShort()
        val decodedEndCol = ((encoded shr 48) and 0xFFFFL).toUShort()
        assertEquals(65535u, decodedStartRow.toUInt())
        assertEquals(65535u, decodedStartCol.toUInt())
        assertEquals(65535u, decodedEndRow.toUInt())
        assertEquals(65535u, decodedEndCol.toUInt())
    }
}
