package io.torvox.ui

import android.content.Context
import android.util.AttributeSet
import android.view.GestureDetector
import android.view.MotionEvent
import android.view.ScaleGestureDetector
import android.view.SurfaceHolder
import android.view.SurfaceView

/**
 * SurfaceView with scrollback gesture support.
 * Handles fling/scroll gestures for terminal scrollback navigation.
 */
class TerminalSurface
    @JvmOverloads
    constructor(
        context: Context,
        attrs: AttributeSet? = null,
        defStyleAttr: Int = 0,
    ) : SurfaceView(context, attrs, defStyleAttr),
        SurfaceHolder.Callback {
        private var bridge: Any? = null
        private var rows: Int = 24
        private var cols: Int = 80
        private var isScrolling: Boolean = false
        private var scrollOffset: Int = 0
        private var maxScrollOffset: Int = 0

        var onScrollChanged: ((offset: Int) -> Unit)? = null
        var onScrollingStateChanged: ((isScrolling: Boolean) -> Unit)? = null

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
                    return false
                }
            }

        private val gestureDetector = GestureDetector(context, gestureListener)
        private val scaleGestureDetector =
            ScaleGestureDetector(
                context,
                object : ScaleGestureDetector.SimpleOnScaleGestureListener() {
                    override fun onScale(detector: ScaleGestureDetector): Boolean = false
                },
            )

        init {
            holder.addCallback(this)
            isFocusable = true
            isFocusableInTouchMode = true
        }

        fun initialize(bridgeInstance: Any) {
            this.bridge = bridgeInstance
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

        override fun onTouchEvent(event: MotionEvent): Boolean {
            var handled = scaleGestureDetector.onTouchEvent(event)
            handled = gestureDetector.onTouchEvent(event) || handled
            return handled || super.onTouchEvent(event)
        }

        override fun surfaceCreated(holder: SurfaceHolder) {}

        override fun surfaceChanged(
            holder: SurfaceHolder,
            format: Int,
            width: Int,
            height: Int,
        ) {}

        override fun surfaceDestroyed(holder: SurfaceHolder) {
            isScrolling = false
            scrollOffset = 0
        }
    }
