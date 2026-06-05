package io.torvox

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Tests that exercise the data model invariants in a JVM-only environment.
 * No Android Context or instrumentation required.
 */
class DataModelTest {
    @Test
    fun terminalState_defaultSessionIdIsZero() {
        assertEquals(0L, TerminalState().sessionId)
    }

    @Test
    fun terminalState_customSessionId() {
        val t = TerminalState(sessionId = 42L)
        assertEquals(42L, t.sessionId)
    }

    @Test
    fun terminalState_defaultTitle() {
        assertEquals("Torvox", TerminalState().title)
    }

    @Test
    fun terminalState_runningFlagToggle() {
        val t1 = TerminalState()
        assertFalse(t1.isRunning)
        val t2 = t1.copy(isRunning = true)
        assertTrue(t2.isRunning)
        assertFalse(t1.isRunning) // copy is immutable
    }

    @Test
    fun selectionState_anchorsIndependence() {
        val start = SelectionAnchor(0, 0)
        val end = SelectionAnchor(1, 1)
        val s = SelectionState(active = true, start = start, end = end)
        assertNotNull(s.start)
        assertNotNull(s.end)
        assertNotEquals(s.start, s.end)
    }

    @Test
    fun selectionState_emptyText() {
        assertEquals("", SelectionState().selectedText)
    }

    @Test
    fun selectionState_longText() {
        val text = "x".repeat(10_000)
        val s = SelectionState(selectedText = text)
        assertEquals(10_000, s.selectedText.length)
    }

    @Test
    fun selectionState_modeCombinations() {
        for (mode in SelectionMode.entries) {
            val s = SelectionState(mode = mode)
            assertEquals(mode, s.mode)
        }
    }

    @Test
    fun selectionAnchor_zeroValues() {
        val a = SelectionAnchor(row = 0, col = 0)
        assertEquals(0, a.row)
        assertEquals(0, a.col)
    }

    @Test
    fun selectionAnchor_negativeValues() {
        val a = SelectionAnchor(row = -1, col = -1)
        assertEquals(-1, a.row)
        assertEquals(-1, a.col)
    }

    @Test
    fun selectionAnchor_largeValues() {
        val a = SelectionAnchor(row = Int.MAX_VALUE, col = Int.MAX_VALUE)
        assertEquals(Int.MAX_VALUE, a.row)
        assertEquals(Int.MAX_VALUE, a.col)
    }

    @Test
    fun terminalState_componentCopy() {
        val t =
            TerminalState(
                sessionId = 1L,
                isRunning = true,
                title = "X",
                selection = SelectionState(active = true),
            )
        val t2 = t.copy(sessionId = 2L)
        assertEquals(2L, t2.sessionId)
        assertEquals("X", t2.title)
        assertTrue(t2.isRunning)
    }

    @Test
    fun selectionState_componentCopy() {
        val s =
            SelectionState(
                active = true,
                start = SelectionAnchor(0, 0),
                end = SelectionAnchor(1, 1),
                mode = SelectionMode.Line,
                selectedText = "hello",
            )
        val s2 = s.copy(mode = SelectionMode.Block)
        assertEquals(SelectionMode.Block, s2.mode)
        assertEquals("hello", s2.selectedText)
        assertTrue(s2.active)
    }

    @Test
    fun terminalState_equalityAcrossComponents() {
        val a = TerminalState(title = "A", sessionId = 1L)
        val b = TerminalState(title = "A", sessionId = 1L)
        assertEquals(a, b)
        val c = TerminalState(title = "A", sessionId = 2L)
        assertNotEquals(a, c)
    }

    @Test
    fun dataClassComponent1ThroughComponentN() {
        val t = TerminalState(sessionId = 7L)
        val components =
            listOf(
                t.component1(), // sessionId
                t.component2(), // isRunning
                t.component3(), // title
                t.component4(), // selection
                t.component5(), // pendingInput
                t.component6(), // modifierKeys
            )
        assertEquals(7L, components[0])
    }

    @Test
    fun selectionStateComponent1ThroughComponent5() {
        val s =
            SelectionState(
                active = true,
                start = SelectionAnchor(1, 2),
                end = SelectionAnchor(3, 4),
                mode = SelectionMode.Word,
                selectedText = "abc",
            )
        assertEquals(true, s.component1())
        assertEquals(SelectionAnchor(1, 2), s.component2())
        assertEquals(SelectionAnchor(3, 4), s.component3())
        assertEquals(SelectionMode.Word, s.component4())
        assertEquals("abc", s.component5())
    }
}
