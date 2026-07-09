package io.torvox.ui

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.graphics.Bitmap
import android.graphics.Color
import android.util.Log
import android.view.MotionEvent
import android.view.TextureView
import android.view.View
import android.view.ViewGroup
import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.test.ext.junit.runners.AndroidJUnit4
import io.torvox.MainActivity
import io.torvox.getBridge
import io.torvox.waitForSession
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import java.io.File
import kotlin.math.abs
import kotlin.math.max
import kotlin.math.min

@RunWith(AndroidJUnit4::class)
class SelectionVisualVerificationTest {
    @get:Rule val composeRule = createAndroidComposeRule<MainActivity>()

    private var tv: TextureView? = null

    data class Region(
        val x: Int,
        val y: Int,
        val w: Int,
        val h: Int,
    )

    private fun findTextureView(root: View): TextureView? {
        if (root is TextureView) return root
        if (root is ViewGroup) {
            for (i in 0 until root.childCount) {
                findTextureView(root.getChildAt(i))?.let { return it }
            }
        }
        return null
    }

    private fun longPressOn(
        x: Float,
        y: Float,
    ) {
        val dt = android.os.SystemClock.uptimeMillis()
        tv!!.dispatchTouchEvent(
            MotionEvent.obtain(dt, dt, MotionEvent.ACTION_DOWN, x, y, 0),
        )
        Thread.sleep(900)
        tv!!.dispatchTouchEvent(
            MotionEvent.obtain(dt, dt + 900, MotionEvent.ACTION_MOVE, x + 1f, y + 1f, 0),
        )
        Thread.sleep(50)
        tv!!.dispatchTouchEvent(
            MotionEvent.obtain(dt, dt + 950, MotionEvent.ACTION_UP, x + 1f, y + 1f, 0),
        )
    }

    /** Save raw pixels to external dir, then pull via adb */
    private fun saveFrame(name: String) {
        Thread.sleep(300)
        val holder = arrayOfNulls<Bitmap>(1)
        composeRule.activityRule.scenario.onActivity { activity ->
            val view = activity.window.decorView
            val bmp = Bitmap.createBitmap(view.width, view.height, Bitmap.Config.ARGB_8888)
            val canvas = android.graphics.Canvas(bmp)
            view.draw(canvas)
            holder[0] = bmp
        }
        val bitmap =
            holder[0] ?: run {
                Log.w("VerifTest", "Failed to capture $name")
                return
            }
        val extDir = composeRule.activity.getExternalFilesDir("Pictures")
        if (extDir != null) {
            extDir.mkdirs()
            val file = File(extDir, "verification-$name.png")
            file.outputStream().use { bitmap.compress(Bitmap.CompressFormat.PNG, 100, it) }
            Log.i("VerifTest", "Saved $name: ${file.absolutePath} (${file.length()}B)")
        }
    }

    @Test
    fun verifyWordSelectionVisual() {
        Log.i("VerifTest", "==== Starting visual verification ====")
        composeRule.waitForSession()
        tv = findTextureView(composeRule.activity.window.decorView)
        if (tv == null) throw AssertionError("TextureView not found")
        val bridge = composeRule.getBridge()
        if (bridge == null) throw AssertionError("Bridge not ready")

        val w = tv!!.width
        val h = tv!!.height
        val cellW = w / 80f
        val cellH = h / 24f

        bridge!!.writeToPty("echo 'selectable text here'\n".toByteArray())
        Thread.sleep(3000)

        // Long-press at cell ~12 on row ~0 to select "text"
        val lx = cellW * 12.5f
        val ly = cellH * 0.5f
        Log.i("VerifTest", "Long-press at ($lx, $ly)")

        saveFrame("01-baseline")

        longPressOn(lx, ly)
        Thread.sleep(1500)

        saveFrame("02-selection-word")

        // Pull frames and verify offline
        Log.i("VerifTest", "Screenshots saved. Verify by pulling from device:")
        Log.i("VerifTest", "  adb pull /storage/emulated/0/Android/data/com.termux/files/Pictures/")
    }

    @Test
    fun verifyBlankAreaPasteButton() {
        composeRule.waitForSession()
        tv = findTextureView(composeRule.activity.window.decorView)
        if (tv == null) throw AssertionError("TextureView not found")
        val bridge = composeRule.getBridge()
        if (bridge == null) throw AssertionError("Bridge not ready")

        val w = tv!!.width
        val h = tv!!.height

        bridge!!.writeToPty("echo 'terminal content'\n".toByteArray())
        Thread.sleep(3000)

        val cm = composeRule.activity.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
        cm.setPrimaryClip(ClipData.newPlainText("test", "paste text"))

        val lx = w * 0.5f
        val ly = h * 0.85f
        Log.i("VerifTest", "Blank long-press at ($lx, $ly)")

        saveFrame("03-blank-baseline")

        longPressOn(lx, ly)
        Thread.sleep(1500)

        saveFrame("04-blank-paste")
        Log.i("VerifTest", "Paste screenshots saved. Pull with adb for analysis.")
    }
}
