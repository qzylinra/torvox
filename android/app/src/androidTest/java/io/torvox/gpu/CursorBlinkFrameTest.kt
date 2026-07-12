
package io.torvox.gpu

import android.content.Context
import androidx.compose.ui.test.junit4.v2.createAndroidComposeRule
import io.torvox.MainActivity
import io.torvox.PixelFrame
import io.torvox.decodeRgbaToPixels
import io.torvox.getBridge
import io.torvox.waitForSession
import org.junit.Assert
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import java.io.File

class CursorBlinkFrameTest {
    @get:Rule
    val composeTestRule = createAndroidComposeRule<MainActivity>()

    @Before
    fun setUp() {
        composeTestRule.waitForSession()
    }

    private fun captureFrame(
        context: Context,
        testDir: File,
        bridge: io.torvox.bridge.TorvoxBridge,
    ): PixelFrame {
        bridge.saveTestFrame(testDir.absolutePath)
        val files =
            testDir.listFiles { f ->
                f.name.endsWith(".rgba") && f.name.startsWith("frame_")
            } ?: error("No frame files written")
        val sorted = files.sortedBy { it.lastModified() }
        return decodeRgbaToPixels(sorted.last())
    }

    @Test
    fun cursorBlink_causesPixelChangesBetweenFrames() {
        val context: Context = composeTestRule.activity.applicationContext
        val bridge = composeTestRule.getBridge() ?: throw AssertionError("bridge null")

        val testDir = File(context.cacheDir, "blink_test_${System.nanoTime()}")
        testDir.mkdirs()

        // Wait for initial frame
        Thread.sleep(500)

        bridge.setCursorBlinkEnabled(true)
        bridge.setCursorBlinkSpeedMs(300)

        val frame1 = captureFrame(context, testDir, bridge)
        Thread.sleep(400)
        val frame2 = captureFrame(context, testDir, bridge)

        val changedPixels = frame1.pixels.zip(frame2.pixels).count { (a, b) -> a != b }
        Assert.assertTrue(
            "Blink enabled: expected pixel changes (found $changedPixels / ${frame1.pixels.size})",
            changedPixels > 0,
        )

        testDir.deleteRecursively()
    }

    @Test
    fun cursorBlink_disabled_noSpontaneousChanges() {
        val context: Context = composeTestRule.activity.applicationContext
        val bridge = composeTestRule.getBridge() ?: throw AssertionError("bridge null")

        val testDir = File(context.cacheDir, "blink_test_${System.nanoTime()}")
        testDir.mkdirs()

        Thread.sleep(500)
        bridge.setCursorBlinkEnabled(false)

        val frame1 = captureFrame(context, testDir, bridge)
        Thread.sleep(1200)
        val frame2 = captureFrame(context, testDir, bridge)

        val changedPixels = frame1.pixels.zip(frame2.pixels).count { (a, b) -> a != b }
        Assert.assertEquals(
            "Blink disabled: expected zero pixel changes (found $changedPixels / ${frame1.pixels.size})",
            0,
            changedPixels,
        )

        testDir.deleteRecursively()
    }

    @Test
    fun cursorBlink_beforeBLINK_PERIOD_noChange() {
        val context: Context = composeTestRule.activity.applicationContext
        val bridge = composeTestRule.getBridge() ?: throw AssertionError("bridge null")

        val testDir = File(context.cacheDir, "blink_test_${System.nanoTime()}")
        testDir.mkdirs()

        Thread.sleep(500)

        // Speed long (1s) — capture early to verify no change before period
        bridge.setCursorBlinkEnabled(true)
        bridge.setCursorBlinkSpeedMs(1000)

        bridge.saveTestFrame(testDir.absolutePath)
        Thread.sleep(500)
        bridge.saveTestFrame(testDir.absolutePath)

        // At 500ms (half the period), cursor should not have blinked yet
        val files =
            testDir.listFiles { f ->
                f.name.endsWith(".rgba") && f.name.startsWith("frame_")
            } ?: error("No frame files written")
        val sorted = files.sortedBy { it.lastModified() }
        val frame1 = decodeRgbaToPixels(sorted[sorted.size - 2])
        val frame2 = decodeRgbaToPixels(sorted.last())

        val changedPixels = frame1.pixels.zip(frame2.pixels).count { (a, b) -> a != b }
        Assert.assertEquals(
            "At 500ms (half of 1000ms period): expected zero pixel changes (found $changedPixels / ${frame1.pixels.size})",
            0,
            changedPixels,
        )

        // Now wait beyond period — cursor should have blinked
        Thread.sleep(700)
        bridge.saveTestFrame(testDir.absolutePath)
        val sorted2 =
            testDir
                .listFiles { f ->
                    f.name.endsWith(".rgba") && f.name.startsWith("frame_")
                }?.sortedBy { it.lastModified() } ?: error("No frame files")
        val frame3 = decodeRgbaToPixels(sorted2.last())

        val changedAfterPeriod = frame2.pixels.zip(frame3.pixels).count { (a, b) -> a != b }
        Assert.assertTrue(
            "After 1200ms (beyond 1000ms period): expected pixel changes (found $changedAfterPeriod / ${frame2.pixels.size})",
            changedAfterPeriod > 0,
        )

        testDir.deleteRecursively()
    }
}
