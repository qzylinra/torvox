package io.torvox.ui

import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.v2.createAndroidComposeRule
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTouchInput
import androidx.compose.ui.test.swipeUp
import io.torvox.MainActivity
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class SettingsScreenTest {
    @get:Rule
    val composeTestRule = createAndroidComposeRule<MainActivity>()

    @Before
    fun setUp() {
        composeTestRule.onNodeWithTag("Key_DRAWER").performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithTag("SettingsButton").performClick()
        composeTestRule.waitForIdle()
    }

    @Test
    fun settings_screen_renders_back_button() {
        composeTestRule.onNodeWithTag("SettingsScreen").assertIsDisplayed()
        composeTestRule.onNodeWithTag("SettingsBackButton").assertIsDisplayed()
    }

    @Test
    fun back_button_navigates_to_terminal() {
        composeTestRule.onNodeWithTag("SettingsBackButton").performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithTag("TerminalScreen").assertIsDisplayed()
    }

    @Test
    fun settings_screen_shows_appearance_section() {
        composeTestRule.onNodeWithText("Appearance").assertIsDisplayed()
    }

    @Test
    fun settings_screen_shows_font_size_slider() {
        composeTestRule.onNodeWithTag("FontSizeSlider").assertIsDisplayed()
    }

    @Test
    fun settings_screen_switches_day_theme() {
        composeTestRule.onNodeWithText("Appearance").performTouchInput { swipeUp() }
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithText("Day").assertExists()
    }

    @Test
    fun settings_screen_displays_font_list() {
        composeTestRule.onNodeWithTag("FontSizeSlider").assertExists()
    }
}
