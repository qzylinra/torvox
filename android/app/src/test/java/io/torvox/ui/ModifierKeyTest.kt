package io.torvox.ui

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class ModifierKeyTest {
    @Test
    fun modifierKey_equality() {
        val a = ModifierKey("ESC", "\u001b", isToggle = false)
        val b = ModifierKey("ESC", "\u001b", isToggle = false)
        assertEquals(a, b)
        assertEquals(a.hashCode(), b.hashCode())
    }

    @Test
    fun modifierKey_inequality() {
        val a = ModifierKey("ESC", "\u001b", isToggle = false)
        val b = ModifierKey("ESC", "\u001b", isToggle = true)
        assertNotEquals(a, b)
    }

    @Test
    fun modifierKey_defaultIsNotToggle() {
        val a = ModifierKey("TAB", "\t")
        assertFalse(a.isToggle)
    }

    @Test
    fun modifierKey_explicitToggle() {
        val a = ModifierKey("CTRL", "", isToggle = true)
        assertTrue(a.isToggle)
    }

    @Test
    fun modifierKey_label() {
        val a = ModifierKey("ESC", "\u001b")
        assertEquals("ESC", a.label)
    }

    @Test
    fun modifierKey_vtSequence() {
        val a = ModifierKey("ESC", "\u001b")
        assertEquals("\u001b", a.vtSequence)
    }

    @Test
    fun defaultModifierKeys_containsEsc() {
        assertTrue(defaultModifierKeys.any { it.label == "ESC" })
    }

    @Test
    fun defaultModifierKeys_containsTab() {
        assertTrue(defaultModifierKeys.any { it.label == "TAB" })
    }

    @Test
    fun defaultModifierKeys_containsCtrl() {
        assertTrue(defaultModifierKeys.any { it.label == "CTRL" && it.isToggle })
    }

    @Test
    fun defaultModifierKeys_containsAlt() {
        assertTrue(defaultModifierKeys.any { it.label == "ALT" && it.isToggle })
    }

    @Test
    fun defaultModifierKeys_containsAllArrows() {
        val labels = defaultModifierKeys.map { it.label }
        assertTrue("\u2190" in labels)
        assertTrue("\u2191" in labels)
        assertTrue("\u2193" in labels)
        assertTrue("\u2192" in labels)
    }

    @Test
    fun defaultModifierKeys_containsSessionButton() {
        assertTrue(defaultModifierKeys.any { it.isSessionButton })
    }

    @Test
    fun defaultModifierKeys_vtSequencesNonEmpty() {
        for (k in defaultModifierKeys) {
            assertTrue(k.vtSequence.isNotEmpty() || k.isToggle || k.isSessionButton)
        }
    }

    @Test
    fun defaultModifierKeys_escVtSequenceIsEscape() {
        val esc = defaultModifierKeys.first { it.label == "ESC" }
        assertEquals("\u001b", esc.vtSequence)
    }

    @Test
    fun defaultModifierKeys_tabVtSequenceIsTab() {
        val tab = defaultModifierKeys.first { it.label == "TAB" }
        assertEquals("\t", tab.vtSequence)
    }

    @Test
    fun defaultModifierKeys_arrowVtSequences() {
        val left = defaultModifierKeys.first { it.label == "\u2190" }
        val up = defaultModifierKeys.first { it.label == "\u2191" }
        val down = defaultModifierKeys.first { it.label == "\u2193" }
        val right = defaultModifierKeys.first { it.label == "\u2192" }
        assertEquals("\u001b[D", left.vtSequence)
        assertEquals("\u001b[A", up.vtSequence)
        assertEquals("\u001b[B", down.vtSequence)
        assertEquals("\u001b[C", right.vtSequence)
    }

    @Test
    fun modifierKey_copy() {
        val a = ModifierKey("ESC", "\u001b")
        val b = a.copy(label = "ESCAPE")
        assertEquals("ESCAPE", b.label)
        assertEquals("\u001b", b.vtSequence)
    }

    @Test
    fun modifierKey_toString() {
        val a = ModifierKey("ESC", "\u001b")
        val s = a.toString()
        assertTrue(s.contains("ESC"))
    }

    @Test
    fun defaultModifierKeys_count() {
        assertEquals(13, defaultModifierKeys.size)
    }

    @Test
    fun defaultModifierKeys_areDistinct() {
        assertEquals(defaultModifierKeys.size, defaultModifierKeys.toSet().size)
    }
}
