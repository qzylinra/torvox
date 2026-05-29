package io.torvox

import io.torvox.bridge.Shell
import io.torvox.bridge.TerminalConfig
import io.torvox.bridge.TerminalError
import io.torvox.bridge.TorvoxBridge
import org.junit.Assert.*
import org.junit.Test

/**
 * Unit tests for TorvoxBridge and config types.
 * These run on the host JVM (no Android device needed).
 */
class TorvoxBridgeTest {
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
                shell = Shell.Custom(path = "/bin/bash"),
                rows = 40u,
                cols = 120u,
                scrollbackLines = 10000u,
            )
        val bridge = TorvoxBridge(config)
        val got = bridge.getConfig()
        assertEquals(config.shell, got.shell)
        assertEquals(config.rows, got.rows)
        assertEquals(config.cols, got.cols)
        assertEquals(config.scrollbackLines, got.scrollbackLines)
    }

    @Test
    fun shellDefaultIsSystem() {
        val s = Shell.Default
        assertTrue(s is Shell.SystemDefault)
    }

    @Test
    fun terminalConfigDefault() {
        val config = TerminalConfig.Default
        assertEquals(24u, config.rows)
        assertEquals(80u, config.cols)
        assertEquals(50000u, config.scrollbackLines)
    }

    @Test
    fun terminalConfigRoundtrip() {
        val config =
            TerminalConfig(
                shell = Shell.Custom(path = "/bin/zsh"),
                rows = 50u,
                cols = 160u,
                scrollbackLines = 100000u,
            )
        val bridge = TorvoxBridge(config)
        val got = bridge.getConfig()
        assertEquals(config, got)
    }

    @Test
    fun bridgeAttrsRoundtrip() {
        val attrs =
            io.torvox.bridge.BridgeAttrs(
                bold = true,
                dim = true,
                italic = false,
                underline = true,
                doubleUnderline = false,
                reverse = false,
                strikethrough = true,
                blink = false,
                hidden = false,
                overline = false,
            )
        assertTrue(attrs.bold)
        assertTrue(attrs.dim)
        assertFalse(attrs.italic)
        assertTrue(attrs.underline)
        assertTrue(attrs.strikethrough)
    }
}
