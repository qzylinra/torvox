package io.torvox

import io.torvox.SelectionAnchor
import io.torvox.SelectionMode
import io.torvox.SelectionState
import io.torvox.TerminalState
import io.torvox.ui.ModifierKey
import io.torvox.ui.defaultModifierKeys
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Unit tests for Kotlin data models (SelectionState, TerminalState, etc.).
 * These run on the host JVM and do not require the Android bridge or device.
 */
class KotlinModelsTest {
    @Test
    fun packageNameIsCorrect() {
        assertEquals(
            "io.torvox",
            KotlinModelsTest::class.qualifiedName?.substringBeforeLast('.'),
        )
    }

    @Test
    fun selectionMode_hasAllVariants() {
        val modes = SelectionMode.entries
        assertEquals(4, modes.size)
        assertTrue(SelectionMode.Char in modes)
        assertTrue(SelectionMode.Word in modes)
        assertTrue(SelectionMode.Line in modes)
        assertTrue(SelectionMode.Block in modes)
    }

    @Test
    fun selectionAnchor_equality() {
        val a = SelectionAnchor(row = 1, col = 2)
        val b = SelectionAnchor(row = 1, col = 2)
        val c = SelectionAnchor(row = 1, col = 3)
        assertEquals(a, b)
        assertEquals(a.hashCode(), b.hashCode())
        assertNotEquals(a, c)
    }

    @Test
    fun selectionState_default() {
        val s = SelectionState()
        assertFalse(s.active)
        assertNull(s.start)
        assertNull(s.end)
        assertEquals(SelectionMode.Char, s.mode)
        assertEquals("", s.selectedText)
    }

    @Test
    fun selectionState_active() {
        val s = SelectionState(active = true)
        assertTrue(s.active)
    }

    @Test
    fun selectionState_withAnchors() {
        val s =
            SelectionState(
                active = true,
                start = SelectionAnchor(0, 0),
                end = SelectionAnchor(5, 10),
            )
        assertEquals(0, s.start?.row)
        assertEquals(10, s.end?.col)
    }

    @Test
    fun terminalState_default() {
        val t = TerminalState()
        assertEquals(0L, t.sessionId)
        assertFalse(t.isRunning)
        assertEquals("Torvox", t.title)
        assertEquals(SelectionState(), t.selection)
    }

    @Test
    fun terminalState_running() {
        val t = TerminalState(isRunning = true)
        assertTrue(t.isRunning)
    }

    @Test
    fun terminalState_customTitle() {
        val t = TerminalState(title = "Custom")
        assertEquals("Custom", t.title)
    }

    @Test
    fun terminalState_pendingInput() {
        val data = byteArrayOf(0x41, 0x42)
        val t = TerminalState(pendingInput = data)
        assertNotNull(t.pendingInput)
        assertEquals(2, t.pendingInput?.size)
    }

    @Test
    fun selectionState_equality() {
        val a = SelectionState(active = true)
        val b = SelectionState(active = true)
        assertEquals(a, b)
        val c = SelectionState(active = true, mode = SelectionMode.Line)
        assertNotEquals(a, c)
    }

    @Test
    fun selectionState_copy() {
        val s = SelectionState(active = true, mode = SelectionMode.Word)
        val s2 = s.copy(mode = SelectionMode.Block)
        assertEquals(SelectionMode.Block, s2.mode)
        assertTrue(s2.active)
    }

    @Test
    fun terminalState_equality() {
        val a = TerminalState(title = "X")
        val b = TerminalState(title = "X")
        assertEquals(a, b)
    }

    @Test
    fun terminalState_modifierKeys() {
        val t = TerminalState()
        // defaultModifierKeys is non-empty list
        assertTrue(t.modifierKeys.isNotEmpty())
    }
}
