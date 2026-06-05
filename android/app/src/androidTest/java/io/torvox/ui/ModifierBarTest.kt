package io.torvox.ui

import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.v2.createAndroidComposeRule
import androidx.compose.ui.test.onNodeWithTag
import io.torvox.MainActivity
import org.junit.Rule
import org.junit.Test

class ModifierBarTest {
    @get:Rule
    val composeTestRule = createAndroidComposeRule<MainActivity>()

    @Test
    fun modifier_bar_renders_all_keys() {
        composeTestRule.onNodeWithTag("ModifierBar").assertIsDisplayed()
        val expectedLabels =
            listOf("ESC", "TAB", "CTRL", "ALT", "\u2191", "\u2193", "\u2630", "~", "/", "-", "\u2190", "\u2192", "ENT")
        expectedLabels.forEach { label ->
            composeTestRule.onNodeWithTag("Key_$label").assertIsDisplayed()
        }
    }

    @Test
    fun modifier_bar_esc_key_exists() {
        composeTestRule.onNodeWithTag("Key_ESC").assertIsDisplayed()
    }

    @Test
    fun modifier_bar_ctrl_key_exists() {
        composeTestRule.onNodeWithTag("Key_CTRL").assertIsDisplayed()
    }

    @Test
    fun modifier_bar_alt_key_exists() {
        composeTestRule.onNodeWithTag("Key_ALT").assertIsDisplayed()
    }
}
