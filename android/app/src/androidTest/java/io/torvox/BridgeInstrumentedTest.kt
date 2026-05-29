package io.torvox

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import io.torvox.bridge.Shell
import io.torvox.bridge.TerminalConfig
import io.torvox.bridge.TerminalError
import io.torvox.bridge.TorvoxBridge
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith

/**
 * Instrumented tests for the Rust ↔ Kotlin bridge via UniFFI.
 * These tests run on a real Android device/emulator.
 */
@RunWith(AndroidJUnit4::class)
class BridgeInstrumentedTest {
    @Test
    fun bridgePing() {
        val config =
            TerminalConfig(
                shell = Shell.SystemDefault,
                rows = 24u,
                cols = 80u,
                scrollbackLines = 50000u,
            )
        val bridge = TorvoxBridge(config)
        assertEquals("pong", bridge.ping())
    }

    @Test
    fun bridgeGetConfig() {
        val config =
            TerminalConfig(
                shell = Shell.Custom(path = "/system/bin/sh"),
                rows = 40u,
                cols = 120u,
                scrollbackLines = 10000u,
            )
        val bridge = TorvoxBridge(config)
        val got = bridge.getConfig()
        assertEquals(40u, got.rows)
        assertEquals(120u, got.cols)
    }

    @Test
    fun bridgeEchoCells() {
        val config =
            TerminalConfig(
                shell = Shell.SystemDefault,
                rows = 24u,
                cols = 80u,
                scrollbackLines = 50000u,
            )
        val bridge = TorvoxBridge(config)
        val cells =
            listOf(
                io.torvox.bridge.BridgeCell(
                    charCode = 'A'.code.toUInt(),
                    fg = 0xFFFFFFu,
                    bg = 0x000000u,
                    attrs =
                        io.torvox.bridge.BridgeAttrs(
                            bold = true,
                            dim = false,
                            italic = false,
                            underline = false,
                            doubleUnderline = false,
                            reverse = false,
                            strikethrough = false,
                            blink = false,
                            hidden = false,
                            overline = false,
                        ),
                ),
            )
        val result = bridge.echoCells(cells)
        assertEquals(1, result.size)
        assertEquals('A'.code.toUInt(), result[0].charCode)
    }

    @Test
    fun shellEnumRoundtrip() {
        val system = Shell.SystemDefault
        val custom = Shell.Custom(path = "/bin/zsh")
        assertTrue(system is Shell.SystemDefault)
        assertTrue(custom is Shell.Custom)
    }

    @Test
    fun appContextIsCorrect() {
        val appContext = InstrumentationRegistry.getInstrumentation().targetContext
        assertEquals("io.torvox", appContext.packageName)
    }
}
