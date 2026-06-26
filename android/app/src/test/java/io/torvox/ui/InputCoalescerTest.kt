package io.torvox.ui

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

class InputCoalescerTest {
    private lateinit var coalescer: InputCoalescer

    @Before
    fun setUp() {
        coalescer = InputCoalescer(deduplicateWindowNanos = 50_000_000L)
    }

    @Test
    fun firstCommitIsNeverDuplicate() {
        assertTrue(coalescer.shouldCommit("a", 1000L))
    }

    @Test
    fun identicalTextWithinWindowIsDuplicate() {
        assertTrue(coalescer.shouldCommit("a", 1000L))
        assertFalse(coalescer.shouldCommit("a", 1000L + 10_000_000L))
    }

    @Test
    fun identicalTextOutsideWindowIsNotDuplicate() {
        assertTrue(coalescer.shouldCommit("a", 1000L))
        assertTrue(coalescer.shouldCommit("a", 1000L + 60_000_000L))
    }

    @Test
    fun differentTextWithinWindowIsNotDuplicate() {
        assertTrue(coalescer.shouldCommit("a", 1000L))
        assertTrue(coalescer.shouldCommit("b", 1000L + 10_000_000L))
    }

    @Test
    fun tripleFireSecondIsDuplicateThirdIsNot() {
        assertTrue(coalescer.shouldCommit("x", 1000L))
        assertFalse(coalescer.shouldCommit("x", 1000L + 5_000_000L))
        assertTrue(coalescer.shouldCommit("x", 1000L + 60_000_000L))
    }

    @Test
    fun composingTextTracking() {
        assertFalse(coalescer.isComposing())
        assertNull(coalescer.getComposingText())

        coalescer.updateComposingText("hel")
        assertTrue(coalescer.isComposing())
        assertEquals("hel", coalescer.getComposingText())

        coalescer.updateComposingText("hell")
        assertEquals("hell", coalescer.getComposingText())

        coalescer.clearComposing()
        assertFalse(coalescer.isComposing())
        assertNull(coalescer.getComposingText())
    }

    @Test
    fun composingEmptyStringIsNotComposing() {
        coalescer.updateComposingText("")
        assertFalse(coalescer.isComposing())
    }

    @Test
    fun resetClearsAllState() {
        coalescer.shouldCommit("a", 1000L)
        coalescer.updateComposingText("hello")

        coalescer.reset()

        assertFalse(coalescer.isComposing())
        assertNull(coalescer.getComposingText())
        assertTrue(coalescer.shouldCommit("a", 2000L))
    }

    @Test
    fun dedupHandlesEmptyString() {
        assertTrue(coalescer.shouldCommit("", 1000L))
        assertFalse(coalescer.shouldCommit("", 1000L + 10_000_000L))
    }

    @Test
    fun dedupHandlesMultiByteCharacters() {
        assertTrue(coalescer.shouldCommit("你好", 1000L))
        assertFalse(coalescer.shouldCommit("你好", 1000L + 5_000_000L))
        assertTrue(coalescer.shouldCommit("你", 1000L + 5_000_000L))
    }

    @Test
    fun dedupHandlesLongStrings() {
        val longText = "a".repeat(1000)
        assertTrue(coalescer.shouldCommit(longText, 1000L))
        assertFalse(coalescer.shouldCommit(longText, 1000L + 10_000_000L))
    }

    @Test
    fun boundaryExactWindowTime() {
        assertTrue(coalescer.shouldCommit("a", 1000L))
        assertTrue(coalescer.shouldCommit("a", 1000L + 50_000_000L))
    }

    @Test
    fun boundaryOneNanosBeforeWindow() {
        assertTrue(coalescer.shouldCommit("a", 1000L))
        assertFalse(coalescer.shouldCommit("a", 1000L + 49_999_999L))
    }
}
