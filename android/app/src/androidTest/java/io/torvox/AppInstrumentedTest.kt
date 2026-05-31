package io.torvox

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith

/**
 * Instrumented tests for the Android app.
 * These tests run on a real Android device/emulator.
 *
 * Note: boltffi JNA bridge types (TorvoxBridge, Shell, etc.) are in
 * io.torvox.bridge.TorvoxBridge.kt. To regenerate after Rust changes:
 * cargo build -p torvox-gui-android && boltffi pack android
 * These tests verify basic Android functionality without the Rust bridge.
 */
@RunWith(AndroidJUnit4::class)
class AppInstrumentedTest {
    @Test
    fun appContextIsCorrect() {
        val appContext = InstrumentationRegistry.getInstrumentation().targetContext
        assertEquals("io.torvox", appContext.packageName)
    }

    @Test
    fun appHasMainActivity() {
        val appContext = InstrumentationRegistry.getInstrumentation().targetContext
        val pm = appContext.packageManager
        val intent = pm.getLaunchIntentForPackage("io.torvox")
        assertNotNull("App has launch intent", intent)
    }

    @Test
    fun appHasCorrectMinSdk() {
        val appContext = InstrumentationRegistry.getInstrumentation().targetContext
        val appInfo =
            appContext.packageManager.getApplicationInfo(
                "io.torvox",
                0,
            )
        // minSdk 33 (Android 13)
        assertTrue("minSdk >= 33", appInfo.minSdkVersion >= 33)
    }
}
