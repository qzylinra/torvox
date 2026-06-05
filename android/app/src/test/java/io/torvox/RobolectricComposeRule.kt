package io.torvox

import androidx.activity.ComponentActivity
import androidx.compose.ui.test.junit4.AndroidComposeTestRule
import org.junit.rules.TestRule
import org.junit.runner.Description
import org.junit.runners.model.Statement
import org.robolectric.Robolectric
import org.robolectric.android.controller.ActivityController
import kotlin.coroutines.EmptyCoroutineContext

class RobolectricActivityRule<A : ComponentActivity>(
    private val activityClass: Class<A>,
) : TestRule {
    lateinit var activity: A
        private set

    lateinit var controller: ActivityController<A>
        private set

    override fun apply(
        base: Statement,
        description: Description,
    ): Statement =
        object : Statement() {
            override fun evaluate() {
                controller = Robolectric.buildActivity(activityClass)
                activity =
                    controller
                        .create()
                        .start()
                        .resume()
                        .visible()
                        .get()
                try {
                    base.evaluate()
                } finally {
                    controller.pause().stop().destroy()
                }
            }
        }
}

inline fun <reified A : ComponentActivity> createRobolectricComposeRule(): AndroidComposeTestRule<RobolectricActivityRule<A>, A> {
    val activityRule = RobolectricActivityRule(A::class.java)
    return AndroidComposeTestRule(
        activityRule,
        EmptyCoroutineContext,
    ) { it.activity }
}
