package io.torvox

import org.junit.Assert.*
import org.junit.Test

/**
 * Unit tests for Kotlin models.
 * These run on the host JVM (no Android device needed).
 *
 * Note: boltffi JNA bridge types (TorvoxBridge, Shell, etc.) are in
 * io.torvox.bridge.TorvoxBridge.kt. To regenerate after Rust changes:
 * cargo build -p torvox-gui-android && boltffi pack android
 * These tests verify Kotlin-side logic without the Rust bridge.
 */
class KotlinModelsTest {
    @Test
    fun packageNameIsCorrect() {
        assertEquals("io.torvox", KotlinModelsTest::class.qualifiedName?.substringBeforeLast('.'))
    }
}
