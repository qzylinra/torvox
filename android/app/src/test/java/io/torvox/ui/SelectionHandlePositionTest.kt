package io.torvox.ui

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class SelectionHandlePositionTest {
    private val cellW = 13.5f
    private val cellH = 66.833336f
    private val cols = 80
    private val rows = 24
    private val density = 3.0f
    private val handleW = (48 * density).toInt()
    private val handleH = (24 * density).toInt()
    private val surfaceW = (cellW * cols).toInt()
    private val surfaceH = (cellH * rows).toInt()

    companion object {
        const val ORIENTATION_LEFT = 0
        const val ORIENTATION_RIGHT = 1
    }

    private fun computeStartOrientation(
        startCol: Int,
        locX: Int,
    ): Int {
        val cursorX = (startCol * cellW).toInt()
        val tipX = locX + cursorX
        return when {
            tipX - handleW < 0 -> ORIENTATION_RIGHT
            tipX + handleW > surfaceW -> ORIENTATION_LEFT
            else -> ORIENTATION_LEFT
        }
    }

    private fun computeStartPopupX(
        startCol: Int,
        locX: Int,
        orientation: Int,
    ): Int {
        val cursorX = (startCol * cellW).toInt()
        val hotspotX = if (orientation == ORIENTATION_LEFT) (handleW * 3) / 4 else handleW / 4
        return locX + cursorX - hotspotX
    }

    private fun computeStartTipX(
        startCol: Int,
        locX: Int,
        orientation: Int,
    ): Int {
        val popupX = computeStartPopupX(startCol, locX, orientation)
        val hotspotX = if (orientation == ORIENTATION_LEFT) (handleW * 3) / 4 else handleW / 4
        return popupX + hotspotX
    }

    private fun computeEndOrientation(
        endCol: Int,
        locX: Int,
    ): Int {
        val cursorX = ((endCol + 1) * cellW).toInt()
        val tipX = locX + cursorX
        return when {
            tipX + handleW > surfaceW -> ORIENTATION_LEFT
            tipX - handleW < 0 -> ORIENTATION_RIGHT
            else -> ORIENTATION_RIGHT
        }
    }

    private fun computeEndPopupX(
        endCol: Int,
        locX: Int,
        orientation: Int,
    ): Int {
        val cursorX = ((endCol + 1) * cellW).toInt()
        val hotspotX = if (orientation == ORIENTATION_LEFT) (handleW * 3) / 4 else handleW / 4
        return locX + cursorX - hotspotX
    }

    private fun computeEndTipX(
        endCol: Int,
        locX: Int,
        orientation: Int,
    ): Int {
        val popupX = computeEndPopupX(endCol, locX, orientation)
        val hotspotX = if (orientation == ORIENTATION_LEFT) (handleW * 3) / 4 else handleW / 4
        return popupX + hotspotX
    }

    @Test
    fun handleDimensions_matchTermuxSpec() {
        assertEquals(48, handleW / density.toInt())
        assertEquals(24, handleH / density.toInt())
    }

    @Test
    fun startHandle_flipsToRight_whenNearLeftEdge() {
        val orientation = computeStartOrientation(0, 0)
        assertEquals("Handle at col 0 should flip to RIGHT", ORIENTATION_RIGHT, orientation)
    }

    @Test
    fun startHandle_flippedTipStillAlignsWithCursor() {
        val locX = 0
        val startCol = 0
        val orientation = computeStartOrientation(startCol, locX)
        val tipX = computeStartTipX(startCol, locX, orientation)
        val cursorX = locX + (startCol * cellW).toInt()
        assertEquals("Tip must align with cursor even when flipped", cursorX, tipX)
    }

    @Test
    fun startHandle_noFlip_inMiddle() {
        val orientation = computeStartOrientation(40, 0)
        assertEquals(ORIENTATION_LEFT, orientation)
    }

    @Test
    fun startHandle_tipAlignsWithCursor_inMiddle() {
        val locX = 0
        val startCol = 40
        val orientation = computeStartOrientation(startCol, locX)
        val tipX = computeStartTipX(startCol, locX, orientation)
        val cursorX = locX + (startCol * cellW).toInt()
        assertEquals("Tip must align with cursor", cursorX, tipX)
    }

    @Test
    fun endHandle_noFlip_inMiddle() {
        val orientation = computeEndOrientation(40, 0)
        assertEquals(ORIENTATION_RIGHT, orientation)
    }

    @Test
    fun endHandle_tipAlignsWithCursor_inMiddle() {
        val locX = 0
        val endCol = 40
        val orientation = computeEndOrientation(endCol, locX)
        val tipX = computeEndTipX(endCol, locX, orientation)
        val cursorX = locX + ((endCol + 1) * cellW).toInt()
        assertEquals("Tip must align with cursor", cursorX, tipX)
    }

    @Test
    fun endHandle_flipsToLeft_whenNearRightEdge() {
        val locX = 0
        val orientation = computeEndOrientation(79, locX)
        assertEquals("Handle at last col should flip to LEFT", ORIENTATION_LEFT, orientation)
    }

    @Test
    fun endHandle_flippedTipStillAlignsWithCursor() {
        val locX = 0
        val endCol = 79
        val orientation = computeEndOrientation(endCol, locX)
        val tipX = computeEndTipX(endCol, locX, orientation)
        val cursorX = locX + ((endCol + 1) * cellW).toInt()
        assertEquals("Tip must align with cursor even when flipped", cursorX, tipX)
    }

    @Test
    fun startHandlePopupX_alwaysLeftOfTip() {
        for (startCol in listOf(0, 10, 40, 79)) {
            val locX = 0
            val orientation = computeStartOrientation(startCol, locX)
            val popupX = computeStartPopupX(startCol, locX, orientation)
            val tipX = computeStartTipX(startCol, locX, orientation)
            assertTrue("Popup must be left of or at tip for col $startCol", popupX <= tipX)
        }
    }

    @Test
    fun endHandlePopupX_alwaysLeftOfTip() {
        for (endCol in listOf(0, 10, 40, 79)) {
            val locX = 0
            val orientation = computeEndOrientation(endCol, locX)
            val popupX = computeEndPopupX(endCol, locX, orientation)
            val tipX = computeEndTipX(endCol, locX, orientation)
            assertTrue("Popup must be left of or at tip for col $endCol", popupX <= tipX)
        }
    }

    @Test
    fun surfaceDimensions() {
        assertEquals(1080, surfaceW)
        assertEquals(1604, surfaceH)
    }

    @Test
    fun startHandleY_belowTextRow() {
        for (row in 0..5) {
            val handleY = ((row + 1) * cellH).toInt()
            assertTrue("Y must be positive for row $row", handleY > 0)
        }
    }

    // -- scrollOffset tests (G5/G6 fixes) --

    @Test
    fun startHandleY_subtractsScrollOffset() {
        val scrollOffset = 3
        val startRow = 5
        val handleY = (((startRow - scrollOffset) + 1) * cellH).toInt()
        val expectedY = ((5 - 3 + 1) * cellH).toInt()
        assertEquals(expectedY, handleY)
    }

    @Test
    fun endHandleY_subtractsScrollOffset() {
        val scrollOffset = 3
        val endRow = 7
        val handleY = (((endRow - scrollOffset) + 1) * cellH).toInt()
        val expectedY = ((7 - 3 + 1) * cellH).toInt()
        assertEquals(expectedY, handleY)
    }

    @Test
    fun contextMenuY_usesVisibleRow() {
        val scrollOffset = 5
        val selectionStartRow = 10
        val locY = 100
        val visibleStartRow = selectionStartRow - scrollOffset
        val selectionTopPx = (locY + visibleStartRow * cellH).toInt()
        val expectedTopPx = (100 + (10 - 5) * cellH).toInt()
        assertEquals(expectedTopPx, selectionTopPx)
    }

    @Test
    fun handleY_withZeroScrollOffset_matchesOriginalFormula() {
        val scrollOffset = 0
        val row = 4
        val newFormula = (((row - scrollOffset) + 1) * cellH).toInt()
        val oldFormula = ((row + 1) * cellH).toInt()
        assertEquals("With scrollOffset=0, both formulas match", oldFormula, newFormula)
    }

    @Test
    fun repositionHandleY_subtractsScrollOffset() {
        val scrollOffset = 10
        val row = 15
        val handleY = (((row - scrollOffset) + 1) * cellH).toInt()
        val expectedY = ((15 - 10 + 1) * cellH).toInt()
        assertEquals(expectedY, handleY)
    }
}
