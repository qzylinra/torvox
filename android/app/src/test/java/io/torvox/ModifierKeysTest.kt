package io.torvox

import io.torvox.ui.ModifierKey
import io.torvox.ui.defaultModifierKeys
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class ModifierKeysTest {
    @Test
    fun defaultModifierKeys_isNotEmpty() {
        assertTrue(defaultModifierKeys.isNotEmpty())
    }

    @Test
    fun defaultModifierKeys_containsCtrl() {
        assertTrue(defaultModifierKeys.any { it == ModifierKey("CTRL", "", isToggle = true) })
    }

    @Test
    fun defaultModifierKeys_containsEscape() {
        assertTrue(defaultModifierKeys.any { it.label == "ESC" })
    }

    @Test
    fun defaultModifierKeys_containsArrowKeys() {
        assertTrue(defaultModifierKeys.any { it.label == "\u2190" })
        assertTrue(defaultModifierKeys.any { it.label == "\u2191" })
        assertTrue(defaultModifierKeys.any { it.label == "\u2193" })
        assertTrue(defaultModifierKeys.any { it.label == "\u2192" })
    }

    @Test
    fun modifierKey_enumEntries() {
        val k = ModifierKey("X", "\u001b")
        assertEquals("X", k.label)
    }

    @Test
    fun modifierKey_equality() {
        val a = ModifierKey("X", "\u001b")
        val b = ModifierKey("X", "\u001b")
        val c = ModifierKey("Y", "\u001b")
        assertEquals(a, b)
        assertNotEquals(a, c)
    }

    @Test
    fun modifierKey_distinct() {
        val keys = defaultModifierKeys
        val distinct = keys.toSet()
        assertTrue("Duplicate modifier keys: $keys", keys.size == distinct.size)
    }
}
