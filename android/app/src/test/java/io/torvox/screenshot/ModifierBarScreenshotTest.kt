package io.torvox.screenshot

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.MaterialTheme
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.test.junit4.AndroidComposeTestRule
import androidx.compose.ui.test.onNodeWithTag
import com.github.takahirom.roborazzi.captureRoboImage
import io.torvox.RobolectricActivityRule
import io.torvox.TestActivity
import io.torvox.ui.ModifierBar
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode
import kotlin.coroutines.EmptyCoroutineContext

@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [33], application = android.app.Application::class)
class ModifierBarScreenshotTest {
    @get:Rule
    val composeTestRule: AndroidComposeTestRule<RobolectricActivityRule<TestActivity>, TestActivity> =
        AndroidComposeTestRule(
            RobolectricActivityRule(TestActivity::class.java),
            EmptyCoroutineContext,
        ) { it.activity }

    @Test
    fun modifierBar_defaultState() {
        composeTestRule.setContent {
            MaterialTheme {
                Box(
                    modifier =
                        Modifier
                            .fillMaxSize()
                            .background(Color(0xFF1E1E2E))
                            .testTag("screenshot_modifier_bar"),
                ) {
                    ModifierBar(
                        onKeySend = {},
                        modifier = Modifier.fillMaxWidth(),
                    )
                }
            }
        }
        composeTestRule
            .onNodeWithTag("screenshot_modifier_bar")
            .captureRoboImage()
    }
}
