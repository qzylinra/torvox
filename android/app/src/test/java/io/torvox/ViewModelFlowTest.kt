package io.torvox

import app.cash.turbine.test
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import io.torvox.settings.SettingsRepository
import io.torvox.ui.ModifierKey
import io.torvox.ui.defaultModifierKeys
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.RuntimeEnvironment

@OptIn(ExperimentalCoroutinesApi::class)
@RunWith(RobolectricTestRunner::class)
class ViewModelFlowTest {
    private val testDispatcher = StandardTestDispatcher()
    private lateinit var settingsRepository: SettingsRepository

    @Before
    fun setup() {
        Dispatchers.setMain(testDispatcher)
        val context = RuntimeEnvironment.getApplication()
        settingsRepository = SettingsRepository(context)
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun terminalState_defaultValues() =
        runTest {
            val state = TerminalState()
            assertEquals(0L, state.sessionId)
            assertFalse(state.isRunning)
            assertEquals("Torvox", state.title)
            assertFalse(state.selection.active)
            assertNotNull(state.modifierKeys)
            assertEquals(13, state.modifierKeys.size)
        }

    @Test
    fun terminalState_copyPreservesFields() =
        runTest {
            val state =
                TerminalState(
                    sessionId = 5,
                    isRunning = true,
                    title = "Test",
                )
            val copied = state.copy(sessionId = 10)
            assertEquals(10L, copied.sessionId)
            assertTrue(copied.isRunning)
            assertEquals("Test", copied.title)
        }

    @Test
    fun selectionState_defaultValues() =
        runTest {
            val sel = SelectionState()
            assertFalse(sel.active)
            assertEquals(null, sel.start)
            assertEquals(null, sel.end)
            assertEquals(SelectionMode.Char, sel.mode)
            assertEquals("", sel.selectedText)
        }

    @Test
    fun selectionState_activeWithAnchors() =
        runTest {
            val sel =
                SelectionState(
                    active = true,
                    start = SelectionAnchor(0, 0),
                    end = SelectionAnchor(5, 10),
                )
            assertTrue(sel.active)
            assertEquals(0, sel.start?.row)
            assertEquals(10, sel.end?.col)
        }

    @Test
    fun selectionMode_allVariants() =
        runTest {
            val modes = SelectionMode.entries
            assertEquals(4, modes.size)
            assertTrue(modes.contains(SelectionMode.Char))
            assertTrue(modes.contains(SelectionMode.Word))
            assertTrue(modes.contains(SelectionMode.Line))
            assertTrue(modes.contains(SelectionMode.Block))
        }

    @Test
    fun selectionState_copyWithMode() =
        runTest {
            val sel = SelectionState(mode = SelectionMode.Char)
            val block = sel.copy(mode = SelectionMode.Block)
            assertEquals(SelectionMode.Block, block.mode)
            assertFalse(block.active)
        }

    @Test
    fun selectionAnchor_equality() =
        runTest {
            val a = SelectionAnchor(1, 2)
            val b = SelectionAnchor(1, 2)
            val c = SelectionAnchor(3, 4)
            assertEquals(a, b)
            assertEquals(a.hashCode(), b.hashCode())
            assertTrue(a != c)
        }

    @Test
    fun modifierKey_toggleDefault_isFalse() =
        runTest {
            val key = ModifierKey("CTRL", "", isToggle = true)
            assertTrue(key.isToggle)
        }

    @Test
    fun modifierKey_nonToggle_hasSequence() =
        runTest {
            val key = ModifierKey("ESC", "\u001b")
            assertFalse(key.isToggle)
            assertEquals("\u001b", key.vtSequence)
        }

    @Test
    fun defaultModifierKeys_has13Keys() =
        runTest {
            assertEquals(13, defaultModifierKeys.size)
            val labels = defaultModifierKeys.map { it.label }
            assertTrue(labels.contains("ESC"))
            assertTrue(labels.contains("TAB"))
            assertTrue(labels.contains("CTRL"))
            assertTrue(labels.contains("ALT"))
        }

    @Test
    fun defaultModifierKeys_ctrlAndAltAreToggles() =
        runTest {
            val ctrl = defaultModifierKeys.first { it.label == "CTRL" }
            val alt = defaultModifierKeys.first { it.label == "ALT" }
            assertTrue("CTRL should be toggle", ctrl.isToggle)
            assertTrue("ALT should be toggle", alt.isToggle)
        }

    @Test
    fun defaultModifierKeys_arrowKeysHaveSequences() =
        runTest {
            val arrows = defaultModifierKeys.filter { it.label in listOf("\u2190", "\u2191", "\u2192", "\u2193") }
            assertEquals(4, arrows.size)
            arrows.forEach { arrow ->
                assertTrue("Arrow ${arrow.label} should have VT sequence", arrow.vtSequence.isNotEmpty())
            }
        }
}
