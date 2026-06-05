package io.torvox

import io.torvox.ui.ModifierKey
import io.torvox.ui.defaultModifierKeys
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Logic and invariant tests that exercise the data model semantics
 * without Android dependencies.
 */
class LogicTest {
    @Test
    fun selectionState_canRepresentActiveSelection() {
        val s =
            SelectionState(
                active = true,
                start = SelectionAnchor(2, 5),
                end = SelectionAnchor(4, 10),
            )
        assertTrue(s.active)
        assertTrue(s.start != null && s.end != null)
    }

    @Test
    fun selectionState_canRepresentInactiveSelection() {
        val s = SelectionState()
        assertFalse(s.active)
    }

    @Test
    fun terminalState_canRepresentRunningSession() {
        val t = TerminalState(sessionId = 1L, isRunning = true, title = "shell")
        assertTrue(t.isRunning)
        assertEquals(1L, t.sessionId)
    }

    @Test
    fun terminalState_canRepresentIdleSession() {
        val t = TerminalState(sessionId = 0L, isRunning = false)
        assertFalse(t.isRunning)
    }

    @Test
    fun modifierKeys_defaultIsReasonable() {
        val keys: List<ModifierKey> = defaultModifierKeys
        // Reasonable default has at least 5 keys
        assertTrue("default should have many modifier keys", keys.size >= 5)
    }

    @Test
    fun modifierKeys_noDuplicates() {
        val keys = defaultModifierKeys
        assertEquals(keys.size, keys.toSet().size)
    }

    @Test
    fun selectionMode_eachDistinct() {
        val set = SelectionMode.entries.toSet()
        assertEquals(4, set.size)
    }

    @Test
    fun selectionState_canHaveNoAnchorsEvenIfActive() {
        // Edge case: active but no anchors (transient state)
        val s = SelectionState(active = true, start = null, end = null)
        assertTrue(s.active)
        assertEquals(null, s.start)
    }

    @Test
    fun selectionAnchor_rowColOrdering() {
        // Higher row = later in document
        val a = SelectionAnchor(1, 50)
        val b = SelectionAnchor(2, 0)
        assertTrue(a.row < b.row)
    }

    @Test
    fun dataClassPreservesAllFieldsOnCopy() {
        val original =
            TerminalState(
                sessionId = 99L,
                isRunning = true,
                title = "Original",
            )
        val copied = original.copy()
        assertEquals(original, copied)
    }

    @Test
    fun selectionState_differentModes_inequality() {
        val a = SelectionState(mode = SelectionMode.Char)
        val b = SelectionState(mode = SelectionMode.Word)
        assertFalse(a == b)
    }

    @Test
    fun selectionState_differentActive_inequality() {
        val a = SelectionState(active = false)
        val b = SelectionState(active = true)
        assertFalse(a == b)
    }

    @Test
    fun selectionState_differentAnchors_inequality() {
        val a = SelectionState(start = SelectionAnchor(0, 0))
        val b = SelectionState(start = SelectionAnchor(1, 0))
        assertFalse(a == b)
    }

    @Test
    fun selectionState_differentText_inequality() {
        val a = SelectionState(selectedText = "hello")
        val b = SelectionState(selectedText = "world")
        assertFalse(a == b)
    }
}
