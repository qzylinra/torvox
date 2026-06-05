package io.torvox.ui

import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.v2.createAndroidComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import io.torvox.MainActivity
import org.junit.Rule
import org.junit.Test

class TerminalScreenTest {
    @get:Rule
    val composeTestRule = createAndroidComposeRule<MainActivity>()

    @Test
    fun topAppBar_renders_with_settings_button() {
        composeTestRule.onNodeWithTag("TerminalScreen").assertIsDisplayed()
        composeTestRule.onNodeWithTag("TerminalTitle").assertIsDisplayed()
        composeTestRule.onNodeWithTag("SettingsButton").assertIsDisplayed()
    }

    @Test
    fun settings_button_opens_settings_screen() {
        composeTestRule.onNodeWithTag("SettingsButton").performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithTag("SettingsScreen").assertIsDisplayed()
        composeTestRule.onNodeWithTag("SettingsBackButton").assertIsDisplayed()
    }

    @Test
    fun terminal_surface_fills_screen() {
        composeTestRule.onNodeWithTag("TerminalContent").assertIsDisplayed()
    }

    @Test
    fun terminal_screen_shows_modifier_bar() {
        composeTestRule.onNodeWithTag("ModifierBar").assertIsDisplayed()
    }

    @Test
    fun terminal_title_shows_default_text() {
        composeTestRule.onNodeWithText("Torvox").assertIsDisplayed()
    }
}
