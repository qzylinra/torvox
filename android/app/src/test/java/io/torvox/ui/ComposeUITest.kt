package io.torvox.ui

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.AndroidComposeTestRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import io.torvox.RobolectricActivityRule
import io.torvox.TestActivity
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import kotlin.coroutines.EmptyCoroutineContext

@RunWith(RobolectricTestRunner::class)
@Config(sdk = [34], application = android.app.Application::class)
class ComposeUITest {
    @get:Rule
    val composeTestRule: AndroidComposeTestRule<RobolectricActivityRule<TestActivity>, TestActivity> =
        AndroidComposeTestRule(
            RobolectricActivityRule(TestActivity::class.java),
            EmptyCoroutineContext,
        ) { it.activity }

    @Test
    fun modifierBar_rendersAllDefaultKeys() {
        composeTestRule.setContent {
            MaterialTheme {
                ModifierBar(onKeySend = {})
            }
        }
        defaultModifierKeys.forEach { key ->
            composeTestRule.onNodeWithTag("Key_${key.label}").assertIsDisplayed()
        }
    }

    @Test
    fun modifierBar_toggleKey_ctrlToggle() {
        composeTestRule.setContent {
            MaterialTheme {
                ModifierBar(onKeySend = {})
            }
        }
        composeTestRule.onNodeWithTag("Key_CTRL").performClick()
        composeTestRule.onNodeWithTag("Key_CTRL").assertIsDisplayed()
    }

    @Test
    fun modifierBar_toggleKey_altToggle() {
        composeTestRule.setContent {
            MaterialTheme {
                ModifierBar(onKeySend = {})
            }
        }
        composeTestRule.onNodeWithTag("Key_ALT").performClick()
        composeTestRule.onNodeWithTag("Key_ALT").assertIsDisplayed()
    }

    @Test
    fun modifierBar_nonToggleKey_sendsEscape() {
        var sentData = ""
        composeTestRule.setContent {
            MaterialTheme {
                ModifierBar(onKeySend = { sentData = it })
            }
        }
        composeTestRule.onNodeWithTag("Key_ESC").performClick()
        assert(sentData == "\u001b") { "ESC should send \\u001b, got: $sentData" }
    }

    @Test
    fun modifierBar_nonToggleKey_sendsTab() {
        var sentData = ""
        composeTestRule.setContent {
            MaterialTheme {
                ModifierBar(onKeySend = { sentData = it })
            }
        }
        composeTestRule.onNodeWithTag("Key_TAB").performClick()
        assert(sentData == "\t") { "TAB should send \\t, got: $sentData" }
    }

    @Test
    fun modifierBar_arrowKey_sendsSequence() {
        var sentData = ""
        composeTestRule.setContent {
            MaterialTheme {
                ModifierBar(onKeySend = { sentData = it })
            }
        }
        composeTestRule.onNodeWithTag("Key_\u2191").performClick()
        assert(sentData == "\u001b[A") { "Up arrow should send \\u001b[A, got: $sentData" }
    }

    @Test
    fun modifierBar_allKeysPresent() {
        val expectedLabels =
            listOf("ESC", "TAB", "CTRL", "ALT", "\u2191", "\u2193", "\u2630", "~", "/", "-", "\u2190", "\u2192", "ENT")
        composeTestRule.setContent {
            MaterialTheme {
                ModifierBar(onKeySend = {})
            }
        }
        expectedLabels.forEach { label ->
            composeTestRule.onNodeWithTag("Key_$label").assertIsDisplayed()
        }
        assert(defaultModifierKeys.size == 13) { "Should have 13 default keys" }
    }

    @Test
    fun modifierBar_enterKey_sendsReturn() {
        var sentData = ""
        composeTestRule.setContent {
            MaterialTheme {
                ModifierBar(onKeySend = { sentData = it })
            }
        }
        composeTestRule.onNodeWithTag("Key_ENT").performClick()
        assert(sentData == "\r") { "ENT should send \\r, got: $sentData" }
    }

    @Test
    fun modifierBar_sessionButton_opensDrawer() {
        var drawerOpened = false
        composeTestRule.setContent {
            MaterialTheme {
                ModifierBar(
                    onKeySend = {},
                    onSessionDrawer = { drawerOpened = true },
                )
            }
        }
        composeTestRule.onNodeWithTag("Key_\u2630").performClick()
        assert(drawerOpened) { "Session button should open drawer" }
    }

    @Test
    fun modifierBar_renders() {
        composeTestRule.setContent {
            MaterialTheme {
                Surface(modifier = Modifier.fillMaxSize()) {
                    Column(Modifier.fillMaxSize()) {
                        ModifierBar(
                            modifier = Modifier.testTag("ModifierBar"),
                            onKeySend = {},
                        )
                    }
                }
            }
        }
        composeTestRule.onNodeWithTag("ModifierBar").assertIsDisplayed()
    }
}
