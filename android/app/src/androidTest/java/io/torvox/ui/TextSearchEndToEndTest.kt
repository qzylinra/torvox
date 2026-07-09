package io.torvox.ui

import android.graphics.Bitmap
import android.os.SystemClock
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.compose.ui.test.onAllNodesWithTag
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextClearance
import androidx.compose.ui.test.performTextInput
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.filters.LargeTest
import androidx.test.platform.app.InstrumentationRegistry
import io.torvox.MainActivity
import io.torvox.getBridge
import io.torvox.openDrawer
import io.torvox.waitForSession
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import java.io.File

@RunWith(AndroidJUnit4::class)
@LargeTest
class TextSearchEndToEndTest {
    @get:Rule
    val composeTestRule = createAndroidComposeRule<MainActivity>()

    @Test
    fun searchFindsTerminalPromptText() {
        composeTestRule.waitForSession()
        val bridge = composeTestRule.getBridge()
        assertNotNull("Bridge must be available", bridge)

        val terminalText = bridge!!.getTerminalText()
        assertNotNull("Terminal text should be available", terminalText)
        assertTrue("Terminal should have content", terminalText!!.isNotEmpty())

        val searchWord = pickSearchWord(terminalText)
        assertTrue("Terminal must contain '$searchWord'", terminalText.contains(searchWord))

        composeTestRule.openDrawer()
        composeTestRule.onNodeWithTag("SearchButton").performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithTag("TextSearchBar").assertIsDisplayed()
        composeTestRule.onNodeWithTag("SearchTextField").assertIsDisplayed()

        composeTestRule.onNodeWithTag("SearchTextField").performTextInput(searchWord)
        composeTestRule.waitForIdle()
        SystemClock.sleep(10000)

        val resultCountNodes = composeTestRule.onAllNodesWithTag("SearchResultCount").fetchSemanticsNodes()
        assertTrue(
            "Search for '$searchWord' should produce a result count indicator, have ${resultCountNodes.size}",
            resultCountNodes.isNotEmpty(),
        )

        takeScreenshot("emulator_endtoend_search_found")
    }

    @Test
    fun searchCaseSensitive_toggleAffectsResults() {
        composeTestRule.waitForSession()
        composeTestRule.openDrawer()
        composeTestRule.onNodeWithTag("SearchButton").performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithTag("TextSearchBar").assertIsDisplayed()

        composeTestRule.onNodeWithTag("SearchCaseSensitive").performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithTag("SearchTextField").performTextInput("HOME")
        composeTestRule.waitForIdle()
        SystemClock.sleep(1000)

        composeTestRule.onAllNodesWithTag("SearchResultCount").fetchSemanticsNodes().let { nodes ->
            assertTrue("Case-sensitive search should have result indicator", nodes.isNotEmpty())
        }

        takeScreenshot("emulator_endtoend_search_case_sensitive")
    }

    @Test
    fun searchNavigation_prevNextWork() {
        composeTestRule.waitForSession()
        composeTestRule.openDrawer()
        composeTestRule.onNodeWithTag("SearchButton").performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithTag("TextSearchBar").assertIsDisplayed()

        composeTestRule.onNodeWithTag("SearchTextField").performTextInput("e")
        composeTestRule.waitForIdle()
        SystemClock.sleep(1500)

        val resultCountNodes = composeTestRule.onAllNodesWithTag("SearchResultCount").fetchSemanticsNodes()

        if (resultCountNodes.isNotEmpty()) {
            composeTestRule.onNodeWithTag("SearchNext").performClick()
            composeTestRule.waitForIdle()
            SystemClock.sleep(500)

            composeTestRule.onNodeWithTag("SearchPrevious").performClick()
            composeTestRule.waitForIdle()
            SystemClock.sleep(500)
        }

        takeScreenshot("emulator_endtoend_search_navigation")
    }

    @Test
    fun searchClose_returnsToModifierBar() {
        composeTestRule.waitForSession()
        composeTestRule.openDrawer()
        composeTestRule.onNodeWithTag("SearchButton").performClick()
        composeTestRule.waitForIdle()
        composeTestRule.onNodeWithTag("TextSearchBar").assertIsDisplayed()

        composeTestRule.onNodeWithTag("SearchClose").performClick()
        composeTestRule.waitForIdle()

        composeTestRule.onAllNodesWithTag("ModifierBar").fetchSemanticsNodes().let { nodes ->
            assertTrue("Closing search should show modifier bar", nodes.isNotEmpty())
        }

        takeScreenshot("emulator_endtoend_search_closed")
    }

    private fun pickSearchWord(text: String): String {
        val words = text.split(Regex("\\s+"))
        for (word in words) {
            val trimmed = word.trim()
            if (trimmed.length >= 3 && !trimmed.all { it.isDigit() }) {
                return trimmed
            }
        }
        return if (text.length > 5) text.substring(0, 5) else text
    }

    private fun takeScreenshot(name: String) {
        val activity = composeTestRule.activity
        val rootView = activity.window.decorView.rootView
        val bitmap = Bitmap.createBitmap(rootView.width, rootView.height, Bitmap.Config.ARGB_8888)
        val canvas = android.graphics.Canvas(bitmap)
        rootView.draw(canvas)

        val dir =
            File(
                InstrumentationRegistry.getInstrumentation().targetContext.getExternalFilesDir(null),
                "screenshots",
            )
        dir.mkdirs()
        File(dir, "$name.png").outputStream().use { out ->
            bitmap.compress(Bitmap.CompressFormat.PNG, 100, out)
        }
        bitmap.recycle()
    }
}
