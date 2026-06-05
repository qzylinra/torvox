package io.torvox.ui

import android.content.Context
import android.util.AttributeSet
import android.view.GestureDetector
import android.view.InputDevice
import android.view.KeyEvent
import android.view.MotionEvent
import android.view.SurfaceHolder
import android.view.SurfaceView
import android.view.View
import android.view.inputmethod.BaseInputConnection
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InputConnection
import android.view.inputmethod.InputMethodManager
import android.widget.Magnifier
import io.torvox.SelectionMode
import io.torvox.TerminalViewModel

class TerminalSurface
    @JvmOverloads
    constructor(
        context: Context,
        attrs: AttributeSet? = null,
        defStyleAttr: Int = 0,
    ) : SurfaceView(context, attrs, defStyleAttr),
        SurfaceHolder.Callback {
        private var viewModel: TerminalViewModel? = null
        private var rows: Int = 24
        private var cols: Int = 80
        private var isScrolling: Boolean = false
        private var scrollOffset: Int = 0
        private var maxScrollOffset: Int = 0

        private var magnifier: Magnifier? = null

        var onScrollChanged: ((offset: Int) -> Unit)? = null
        var onScrollingStateChanged: ((isScrolling: Boolean) -> Unit)? = null
        var onSwipeLeft: (() -> Unit)? = null
        var onSwipeRight: (() -> Unit)? = null

        private val gestureListener =
            object : GestureDetector.SimpleOnGestureListener() {
                override fun onDown(e: MotionEvent): Boolean = true

                override fun onScroll(
                    e1: MotionEvent?,
                    e2: MotionEvent,
                    distanceX: Float,
                    distanceY: Float,
                ): Boolean {
                    if (!isScrolling) {
                        isScrolling = true
                        onScrollingStateChanged?.invoke(true)
                    }
                    val scrollAmount = (distanceY / 16f).toInt().coerceAtLeast(1)
                    val newOffset = (scrollOffset + scrollAmount).coerceIn(0, maxScrollOffset)
                    if (newOffset != scrollOffset) {
                        scrollOffset = newOffset
                        onScrollChanged?.invoke(scrollOffset)
                    }
                    return true
                }

                override fun onFling(
                    e1: MotionEvent?,
                    e2: MotionEvent,
                    velocityX: Float,
                    velocityY: Float,
                ): Boolean {
                    val absX = kotlin.math.abs(velocityX)
                    val absY = kotlin.math.abs(velocityY)

                    if (absX > absY && absX > 500f) {
                        if (velocityX > 0) {
                            onSwipeRight?.invoke()
                        } else {
                            onSwipeLeft?.invoke()
                        }
                        return true
                    }

                    val flingAmount = (velocityY / 100f).toInt().coerceIn(-50, 50)
                    val newOffset = (scrollOffset + flingAmount).coerceIn(0, maxScrollOffset)
                    if (newOffset != scrollOffset) {
                        scrollOffset = newOffset
                        onScrollChanged?.invoke(scrollOffset)
                    }
                    postDelayed({
                        isScrolling = false
                        onScrollingStateChanged?.invoke(false)
                    }, 300)
                    return true
                }

                override fun onSingleTapUp(e: MotionEvent): Boolean {
                    if (isScrolling) {
                        isScrolling = false
                        scrollOffset = 0
                        onScrollChanged?.invoke(0)
                        onScrollingStateChanged?.invoke(false)
                        return true
                    }
                    viewModel?.clearSelection()
                    requestFocus()
                    post {
                        val imm = context.getSystemService(Context.INPUT_METHOD_SERVICE) as InputMethodManager
                        imm.showSoftInput(this@TerminalSurface, InputMethodManager.SHOW_IMPLICIT)
                    }
                    return true
                }

                override fun onLongPress(e: MotionEvent) {
                    val row = (e.y / 16f).toInt().coerceIn(0, rows - 1)
                    val col = (e.x / 8f).toInt().coerceIn(0, cols - 1)
                    viewModel?.startSelection(row, col)
                    magnifier = magnifier ?: Magnifier.Builder(this@TerminalSurface).build()
                    magnifier?.show(e.rawX, e.rawY)
                }
            }

        private val gestureDetector = GestureDetector(context, gestureListener)

        init {
            holder.addCallback(this)
            holder.setFormat(android.graphics.PixelFormat.OPAQUE)
            isFocusable = true
            isFocusableInTouchMode = true
            setZOrderOnTop(false)
        }

        override fun onCheckIsTextEditor(): Boolean = true

        override fun onAttachedToWindow() {
            super.onAttachedToWindow()
            requestFocus()
            post {
                val imm = context.getSystemService(Context.INPUT_METHOD_SERVICE) as InputMethodManager
                imm.showSoftInput(this, InputMethodManager.SHOW_FORCED)
            }
        }

        override fun onWindowFocusChanged(hasWindowFocus: Boolean) {
            super.onWindowFocusChanged(hasWindowFocus)
            if (hasWindowFocus) {
                post {
                    val imm = context.getSystemService(Context.INPUT_METHOD_SERVICE) as InputMethodManager
                    imm.showSoftInput(this, InputMethodManager.SHOW_FORCED)
                }
            }
        }

        fun initialize(viewModel: TerminalViewModel) {
            this.viewModel = viewModel
        }

        fun setDimensions(
            rows: Int,
            cols: Int,
        ) {
            this.rows = rows
            this.cols = cols
        }

        fun setMaxScrollback(maxLines: Int) {
            this.maxScrollOffset = maxLines
        }

        fun setScrollOffset(offset: Int) {
            this.scrollOffset = offset.coerceIn(0, maxScrollOffset)
            onScrollChanged?.invoke(this.scrollOffset)
        }

        fun getScrollOffset(): Int = scrollOffset

        fun isCurrentlyScrolling(): Boolean = isScrolling

        fun consumePendingInput(): ByteArray? = viewModel?.consumePendingInput()

        override fun onCreateInputConnection(outAttrs: EditorInfo): InputConnection {
            outAttrs.inputType =
                android.text.InputType.TYPE_CLASS_TEXT or
                android.text.InputType.TYPE_TEXT_FLAG_NO_SUGGESTIONS
            outAttrs.imeOptions =
                EditorInfo.IME_FLAG_NO_ENTER_ACTION or EditorInfo.IME_ACTION_NONE
            return object : BaseInputConnection(this, false) {
                override fun commitText(
                    text: CharSequence?,
                    newCursorPosition: Int,
                ): Boolean {
                    val data = text?.toString()?.toByteArray() ?: return false
                    viewModel?.writeToPty(data)
                    return true
                }

                override fun sendKeyEvent(event: KeyEvent): Boolean =
                    if (event.action == KeyEvent.ACTION_DOWN) {
                        handleKeyEvent(event)
                    } else {
                        true
                    }

                override fun deleteSurroundingText(
                    beforeLength: Int,
                    afterLength: Int,
                ): Boolean {
                    viewModel?.writeToPty(byteArrayOf(0x7F))
                    return true
                }
            }
        }

        private fun ctrlKeyCode(keyCode: Int): Byte? =
            when (keyCode) {
                KeyEvent.KEYCODE_A -> 0x01
                KeyEvent.KEYCODE_B -> 0x02
                KeyEvent.KEYCODE_C -> 0x03
                KeyEvent.KEYCODE_D -> 0x04
                KeyEvent.KEYCODE_E -> 0x05
                KeyEvent.KEYCODE_F -> 0x06
                KeyEvent.KEYCODE_G -> 0x07
                KeyEvent.KEYCODE_H -> 0x08
                KeyEvent.KEYCODE_I -> 0x09
                KeyEvent.KEYCODE_J -> 0x0A
                KeyEvent.KEYCODE_K -> 0x0B
                KeyEvent.KEYCODE_L -> 0x0C
                KeyEvent.KEYCODE_M -> 0x0D
                KeyEvent.KEYCODE_N -> 0x0E
                KeyEvent.KEYCODE_O -> 0x0F
                KeyEvent.KEYCODE_P -> 0x10
                KeyEvent.KEYCODE_Q -> 0x11
                KeyEvent.KEYCODE_R -> 0x12
                KeyEvent.KEYCODE_S -> 0x13
                KeyEvent.KEYCODE_T -> 0x14
                KeyEvent.KEYCODE_U -> 0x15
                KeyEvent.KEYCODE_V -> 0x16
                KeyEvent.KEYCODE_W -> 0x17
                KeyEvent.KEYCODE_X -> 0x18
                KeyEvent.KEYCODE_Y -> 0x19
                KeyEvent.KEYCODE_Z -> 0x1A
                else -> null
            }

        override fun onKeyDown(
            keyCode: Int,
            event: KeyEvent,
        ): Boolean {
            if (event.isCtrlPressed || viewModel?.state?.value?.ctrlActive == true) {
                val ctrlByte = ctrlKeyCode(keyCode)
                if (ctrlByte != null) {
                    viewModel?.writeToPty(byteArrayOf(ctrlByte))
                    return true
                }
            }
            val unicodeChar = event.unicodeChar
            if (unicodeChar > 0) {
                if (event.isAltPressed || viewModel?.state?.value?.altActive == true) {
                    viewModel?.writeToPty(byteArrayOf(0x1B, unicodeChar.toByte()))
                } else {
                    viewModel?.writeToPty(byteArrayOf(unicodeChar.toByte()))
                }
                return true
            }
            return super.onKeyDown(keyCode, event)
        }

        override fun onKeyUp(
            keyCode: Int,
            event: KeyEvent,
        ): Boolean = true

        private fun handleKeyEvent(event: KeyEvent): Boolean = onKeyDown(event.keyCode, event)

        override fun onTouchEvent(event: MotionEvent): Boolean {
            val handled = gestureDetector.onTouchEvent(event)
            if (event.actionMasked == MotionEvent.ACTION_MOVE && viewModel
                    ?.state
                    ?.value
                    ?.selection
                    ?.active == true
            ) {
                val row = (event.y / 16f).toInt().coerceIn(0, rows - 1)
                val col = (event.x / 8f).toInt().coerceIn(0, cols - 1)
                viewModel?.updateSelection(row, col)
                magnifier?.show(event.rawX, event.rawY)
            }
            if (event.actionMasked == MotionEvent.ACTION_UP || event.actionMasked == MotionEvent.ACTION_CANCEL) {
                viewModel?.endSelection()
                magnifier?.dismiss()
                magnifier = null
            }
            return handled || super.onTouchEvent(event)
        }

        override fun surfaceCreated(holder: SurfaceHolder) {
            android.util.Log.d("TerminalSurface", "surfaceCreated: w=$width h=$height")
            android.util.Log.d(
                "TerminalSurface",
                "surfaceCreated frame: w=${holder.surfaceFrame.width()} h=${holder.surfaceFrame.height()}",
            )
            // Surface dimensions not final here; actual init happens in surfaceChanged
            viewModel?.let { vm ->
                vm.currentSurface = holder.surface
            }
        }

        override fun surfaceChanged(
            holder: SurfaceHolder,
            format: Int,
            width: Int,
            height: Int,
        ) {
            android.util.Log.d("TerminalSurface", "surfaceChanged: format=$format w=$width h=$height")
            viewModel?.let { vm ->
                vm.currentSurface = holder.surface
                vm.surfaceWidth = width
                vm.surfaceHeight = height
                if (!vm.runtime.state.value.isRunning) {
                    vm.startRuntime(holder.surface, width, height)
                } else {
                    vm.runtime.resize(
                        (height / 17f).toInt().coerceIn(5, 200),
                        (width / 8f).toInt().coerceIn(20, 300),
                    )
                }
            }
        }

        override fun surfaceDestroyed(holder: SurfaceHolder) {
            viewModel?.runtime?.stop()
            isScrolling = false
            scrollOffset = 0
        }

        override fun onDetachedFromWindow() {
            super.onDetachedFromWindow()
            handler.removeCallbacksAndMessages(null)
        }
    }
