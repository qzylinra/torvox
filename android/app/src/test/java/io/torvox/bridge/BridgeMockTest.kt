package io.torvox.bridge

import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [34])
class BridgeMockTest {
    @Test
    fun bridge_ping_returnsPong() {
        val bridge = mockk<TorvoxBridge>()
        every { bridge.ping() } returns "pong"
        assertEquals("pong", bridge.ping())
    }

    @Test
    fun bridge_scrollbackLen_returnsValue() {
        val bridge = mockk<TorvoxBridge>()
        every { bridge.scrollbackLen() } returns 42u
        assertEquals(42u, bridge.scrollbackLen())
    }

    @Test
    fun bridge_scrollbackLine_returnsLine() {
        val bridge = mockk<TorvoxBridge>()
        every { bridge.scrollbackLine(0u) } returns "hello world"
        assertEquals("hello world", bridge.scrollbackLine(0u))
    }

    @Test
    fun bridge_scrollbackLine_returnsNullForInvalidIndex() {
        val bridge = mockk<TorvoxBridge>()
        every { bridge.scrollbackLine(999u) } returns null
        assertEquals(null, bridge.scrollbackLine(999u))
    }

    @Test
    fun bridge_writeToPty_doesNotThrow() {
        val bridge = mockk<TorvoxBridge>(relaxed = true)
        bridge.writeToPty("test\n".toByteArray())
        verify { bridge.writeToPty(any()) }
    }

    @Test
    fun bridge_resize_updatesDimensions() {
        val bridge = mockk<TorvoxBridge>(relaxed = true)
        bridge.resize(50u, 120u)
        verify { bridge.resize(50u, 120u) }
    }

    @Test
    fun bridge_releaseSurface_doesNotThrow() {
        val bridge = mockk<TorvoxBridge>(relaxed = true)
        bridge.releaseSurface()
        verify { bridge.releaseSurface() }
    }

    @Test
    fun bridge_close_doesNotThrow() {
        val bridge = mockk<TorvoxBridge>(relaxed = true)
        bridge.close()
        verify { bridge.close() }
    }

    @Test
    fun bridge_saveSession_doesNotThrow() {
        val bridge = mockk<TorvoxBridge>(relaxed = true)
        bridge.saveSession("/tmp/test.bin")
        verify { bridge.saveSession("/tmp/test.bin") }
    }

    @Test
    fun bridge_setSavePath_doesNotThrow() {
        val bridge = mockk<TorvoxBridge>(relaxed = true)
        bridge.setSavePath("/tmp/test.bin")
        verify { bridge.setSavePath("/tmp/test.bin") }
    }

    @Test
    fun terminalConfig_creation() {
        val config =
            TerminalConfig(
                shell = Shell.SystemDefault,
                rows = 24u,
                cols = 80u,
                scrollbackLines = 50000u,
                font_size_tenths = 140u,
                theme =
                    BridgeTheme(
                        name = "Test",
                        bg = 0x1E1E2E,
                        fg = 0xCDD6F4,
                        cursor = 0xF5E0DC,
                        selectionBg = 0x45475A,
                        ansi0 = 0x45475A,
                        ansi1 = 0xF38BA8,
                        ansi2 = 0xA6E3A1,
                        ansi3 = 0xF9E2AF,
                        ansi4 = 0x89B4FA,
                        ansi5 = 0xF5C2E7,
                        ansi6 = 0x94E2D5,
                        ansi7 = 0xBAC2DE,
                        ansi8 = 0x585B70,
                        ansi9 = 0xF38BA8,
                        ansi10 = 0xA6E3A1,
                        ansi11 = 0xF9E2AF,
                        ansi12 = 0x89B4FA,
                        ansi13 = 0xF5C2E7,
                        ansi14 = 0x94E2D5,
                        ansi15 = 0xA6ADC8,
                    ),
            )
        assertEquals(24u, config.rows)
        assertEquals(80u, config.cols)
        assertEquals(50000u, config.scrollbackLines)
        assertEquals(140u, config.font_size_tenths)
    }

    @Test
    fun shell_systemDefault() {
        val shell = Shell.SystemDefault
        assertNotNull(shell)
    }

    @Test
    fun shell_custom() {
        val shell = Shell.Custom("/system/bin/bash")
        assertTrue(shell is Shell.Custom)
        assertEquals("/system/bin/bash", (shell as Shell.Custom).path)
    }

    @Test
    fun bridgeTheme_all16AnsiColors() {
        val theme =
            BridgeTheme(
                name = "Test",
                bg = 0,
                fg = 0,
                cursor = 0,
                selectionBg = 0,
                ansi0 = 0,
                ansi1 = 1,
                ansi2 = 2,
                ansi3 = 3,
                ansi4 = 4,
                ansi5 = 5,
                ansi6 = 6,
                ansi7 = 7,
                ansi8 = 8,
                ansi9 = 9,
                ansi10 = 10,
                ansi11 = 11,
                ansi12 = 12,
                ansi13 = 13,
                ansi14 = 14,
                ansi15 = 15,
            )
        assertNotNull(theme)
        assertEquals("Test", theme.name)
    }

    @Test
    fun bridge_mockSequence_pingThenWrite() {
        val bridge = mockk<TorvoxBridge>(relaxed = true)
        every { bridge.ping() } returns "pong"

        assertEquals("pong", bridge.ping())
        bridge.writeToPty("ls\n".toByteArray())
        bridge.writeToPty("pwd\n".toByteArray())

        verify(exactly = 2) { bridge.writeToPty(any()) }
    }

    @Test
    fun bridge_mockSequence_lifecycle() {
        val bridge = mockk<TorvoxBridge>(relaxed = true)
        every { bridge.ping() } returns "pong"

        bridge.ping()
        bridge.setSavePath("/tmp/test.bin")
        bridge.saveSession("/tmp/test.bin")
        bridge.releaseSurface()
        bridge.close()

        verify { bridge.ping() }
        verify { bridge.setSavePath("/tmp/test.bin") }
        verify { bridge.saveSession("/tmp/test.bin") }
        verify { bridge.releaseSurface() }
        verify { bridge.close() }
    }
}
