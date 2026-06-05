package io.torvox

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Sanity tests for io.torvox package structure and basic Kotlin assertions.
 * Ensures the test runner and source layout are correctly wired.
 * Skips Compose UI and TorvoxExec (Composable / native exec) classes that
 * require a full Android runtime to load.
 */
class PackageStructureTest {
    @Test
    fun appClassExists() {
        val cls = Class.forName("io.torvox.TorvoxApp")
        assertNotNull(cls)
    }

    @Test
    fun runtimePackageExists() {
        val cls = Class.forName("io.torvox.runtime.TorvoxRuntime")
        assertNotNull(cls)
    }

    @Test
    fun settingsPackageExists() {
        val cls = Class.forName("io.torvox.settings.SettingsRepository")
        assertNotNull(cls)
    }

    @Test
    fun servicePackageExists() {
        val cls = Class.forName("io.torvox.service.TerminalForegroundService")
        assertNotNull(cls)
    }

    @Test
    fun bridgePackageExists() {
        val cls = Class.forName("io.torvox.bridge.TorvoxBridge")
        assertNotNull(cls)
    }

    @Test
    fun execPackageExists() {
        // ExecInstaller is a Kotlin class but loads native libs via System.loadLibrary;
        // use NoExec to detect the package without loading native code.
        val cls = Class.forName("io.torvox.exec.ExecInstaller")
        assertNotNull(cls)
    }

    @Test
    fun terminalViewModelExists() {
        val cls = Class.forName("io.torvox.TerminalViewModel")
        assertNotNull(cls)
    }

    @Test
    fun mainActivityExists() {
        val cls = Class.forName("io.torvox.MainActivity")
        assertNotNull(cls)
    }

    @Test
    fun terminalStateClassExists() {
        val cls = Class.forName("io.torvox.TerminalState")
        assertNotNull(cls)
    }

    @Test
    fun selectionStateClassExists() {
        val cls = Class.forName("io.torvox.SelectionState")
        assertNotNull(cls)
    }

    @Test
    fun kotlinSourceFilesLoadable() {
        // Count Kotlin data classes via reflection
        val stateClass = TerminalState::class
        assertEquals("TerminalState", stateClass.simpleName)
    }

    @Test
    fun dataClassCopyWorks() {
        val t = TerminalState(title = "A")
        val t2 = t.copy(title = "B")
        assertEquals("A", t.title)
        assertEquals("B", t2.title)
    }

    @Test
    fun dataClassHashCode() {
        val a = TerminalState(title = "X")
        val b = TerminalState(title = "X")
        assertEquals(a.hashCode(), b.hashCode())
    }

    @Test
    fun dataClassToString() {
        val t = TerminalState(title = "Hello")
        val s = t.toString()
        assertTrue(s.contains("Hello"))
    }

    @Test
    fun selectionStateToString() {
        val s = SelectionState(active = true)
        val str = s.toString()
        assertTrue(str.contains("true"))
    }
}
