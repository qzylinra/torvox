package io.torvox.ui

import android.annotation.SuppressLint
import android.content.ClipboardManager
import android.content.Context
import android.graphics.SurfaceTexture
import android.graphics.drawable.Drawable
import android.util.AttributeSet
import android.util.Log
import android.view.GestureDetector
import android.view.InputDevice
import android.view.KeyEvent
import android.view.MotionEvent
import android.view.ScaleGestureDetector
import android.view.Surface
import android.view.TextureView
import android.view.View
import android.view.inputmethod.BaseInputConnection
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InputConnection
import android.view.inputmethod.InputMethodManager
import android.widget.Magnifier
import android.widget.PopupWindow
import io.torvox.R
import io.torvox.SelectionMode
import io.torvox.TerminalViewModel

class TerminalSurface
@JvmOverloads
constructor(
    context: Context,
    attrs: AttributeSet? = null,
    defStyleAttr: Int = 0,
) : TextureView(context, attrs, defStyleAttr),
    TextureView.SurfaceTextureListener {
    companion object {
        private const val TAG = "TerminalSurface"
        private const val ORIENTATION_LEFT = 0
        private const val ORIENTATION_RIGHT = 2
    }

    private var viewModel: TerminalViewModel? = null
    private var rows: Int = 24
    private var cols: Int = 80
    private var surfaceWidthPixels: Int = 0
    private var surfaceHeightPixels: Int = 0
    private var isScrolling: Boolean = false
    private var scrollOffset: Int = 0
    private var maxScrollOffset: Int = 0

    private var magnifier: Magnifier? = null

    var onScrollChanged: ((offset: Int) -> Unit)? = null
    var onScrollingStateChanged: ((isScrolling: Boolean) -> Unit)? = null
    var onSwipeLeft: (() -> Unit)? = null
    var onSwipeRight: (() -> Unit)? = null
    var onCopyRequested: ((text: String) -> Unit)? = null
    var onPasteRequested: (() -> Unit)? = null
    var onZoomChanged: ((increase: Boolean) -> Unit)? = null

    var drawerOpen: Boolean = false
        set(value) {
            field = value
            Log.d(TAG, "drawerOpen=$value")
            if (value) {
                hideSelectionHandles()
                hidePastePopup()
            }
        }

    private val drawerWidthPixels: Float by lazy { 280f * resources.displayMetrics.density }

    private var startHandlePopup: PopupWindow? = null
    private var endHandlePopup: PopupWindow? = null
    private var toolbarPopup: PopupWindow? = null
    private var cursorHandlePopup: PopupWindow? = null
    private var actionMode: android.view.ActionMode? = null
    private var pasteActionMode: android.view.ActionMode? = null
    private var selectionRect = android.graphics.Rect()
    private var selectionFgColor: Int = 0xFFFFFFFF.toInt()
    private var selectionBgColor: Int = 0xFF000000.toInt()
    private var pendingUrl: String? = null

    @Suppress("CyclomaticComplexMethod", "ComplexCondition")
    fun showSelectionHandles(
        startRow: Int,
        startCol: Int,
        endRow: Int,
        endCol: Int,
        themeFgColor: Int = 0xFF2196F3.toInt(),
    ) {
        hideSelectionHandles()
        if (startRow < 0 || startCol < 0 || endRow < 0 || endCol < 0) return

        val loc = IntArray(2)
        getLocationInWindow(loc)

        val leftDrawable =
            androidx.core.content.ContextCompat
                .getDrawable(context, R.drawable.text_select_handle_left_material)!!
                .mutate()
        val rightDrawable =
            androidx.core.content.ContextCompat
                .getDrawable(context, R.drawable.text_select_handle_right_material)!!
                .mutate()
        leftDrawable.setTint(themeFgColor)
        rightDrawable.setTint(themeFgColor)
        val handleW = leftDrawable.intrinsicWidth
        val handleH = leftDrawable.intrinsicHeight

        val startCursorX = (startCol * cellWidth).toInt()
        val startCursorY =
            ((startRow + 1) * cellHeight)
                .toInt()
                .coerceAtMost(surfaceHeightPixels - handleH)

        val startOrientation =
            when {
                startCursorX - handleW < 0 -> ORIENTATION_RIGHT
                startCursorX + handleW > surfaceWidthPixels -> ORIENTATION_LEFT
                else -> ORIENTATION_LEFT
            }
        val (startDrawable, startHotspotX) =
            if (startOrientation == ORIENTATION_LEFT) {
                leftDrawable to (handleW * 3) / 4
            } else {
                rightDrawable to handleW / 4
            }
        val startView = createHandleViewWithDrawable(startDrawable, HandleDrag.START)
        val startPopupX =
            (loc[0] + startCursorX - startHotspotX)
                .coerceIn(loc[0], (loc[0] + surfaceWidthPixels - handleW).coerceAtLeast(loc[0]))
        startHandlePopup =
            createHandlePopup(startView).apply {
                showAtLocation(this@TerminalSurface, 0, startPopupX, loc[1] + startCursorY)
            }

        val endCursorX = ((endCol + 1) * cellWidth).toInt()
        val endCursorY =
            ((endRow + 1) * cellHeight)
                .toInt()
                .coerceAtMost(surfaceHeightPixels - handleH)

        val endOrientation =
            when {
                endCursorX + handleW > surfaceWidthPixels -> ORIENTATION_LEFT
                endCursorX - handleW < 0 -> ORIENTATION_RIGHT
                else -> ORIENTATION_RIGHT
            }
        val (endDrawableChoice, endHotspotX) =
            if (endOrientation == ORIENTATION_LEFT) {
                leftDrawable to (handleW * 3) / 4
            } else {
                rightDrawable to handleW / 4
            }
        val endView = createHandleViewWithDrawable(endDrawableChoice, HandleDrag.END)
        val endPopupX =
            (loc[0] + endCursorX - endHotspotX)
                .coerceIn(loc[0], (loc[0] + surfaceWidthPixels - handleW).coerceAtLeast(loc[0]))
        endHandlePopup =
            createHandlePopup(endView).apply {
                showAtLocation(this@TerminalSurface, 0, endPopupX, loc[1] + endCursorY)
            }
    }

    @Suppress("LongParameterList")
    fun showSelectionToolbar(
        loRow: Int,
        hiRow: Int,
        loCol: Int,
        hiCol: Int,
        onCopy: () -> Unit,
        onSelectAll: () -> Unit,
        onOpenUrl: ((String) -> Unit)? = null,
        selectedText: String = "",
        themeFgColor: Int = 0xFFFFFFFF.toInt(),
        themeBgColor: Int = 0xFF000000.toInt(),
        onMoveAnchor: ((moveEnd: Boolean, direction: Int) -> Unit)? = null,
    ) {
        hideSelectionToolbar()
        selectionFgColor = themeFgColor
        selectionBgColor = themeBgColor

        val startPxX = (loCol * cellWidth).toInt()
        val startPxY = (loRow * cellHeight).toInt()
        val endPxX = ((hiCol + 1) * cellWidth).toInt()
        val endPxY = ((hiRow + 1) * cellHeight).toInt()

        val loc = IntArray(2)
        getLocationInWindow(loc)
        selectionRect =
            android.graphics.Rect(
                loc[0] + startPxX,
                loc[1] + startPxY,
                loc[0] + endPxX,
                loc[1] + endPxY,
            )

        val callback =
            object : android.view.ActionMode.Callback2() {
                override fun onCreateActionMode(
                    mode: android.view.ActionMode,
                    menu: android.view.Menu,
                ): Boolean {
                    menu.add(0, 1, 0, "Copy")
                    menu.add(0, 2, 1, "Select All")
                    menu.add(0, 5, 2, "◀ Anchor")
                    menu.add(0, 6, 3, "Anchor ▶")
                    if (onOpenUrl != null && selectedText.isNotEmpty()) {
                        val urls = UrlDetector.findUrls(selectedText)
                        if (urls.isNotEmpty()) {
                            menu.add(0, 4, 4, "Open URL")
                            pendingUrl = urls.first()
                        }
                    }
                    return true
                }

                override fun onPrepareActionMode(
                    mode: android.view.ActionMode,
                    menu: android.view.Menu,
                ): Boolean = false

                override fun onActionItemClicked(
                    mode: android.view.ActionMode,
                    item: android.view.MenuItem,
                ): Boolean {
                    when (item.itemId) {
                        1 -> {
                            onCopy()
                            mode.finish()
                        }

                        2 -> {
                            onSelectAll()
                        }

                        4 -> {
                            pendingUrl?.let { onOpenUrl?.invoke(it) }
                            mode.finish()
                        }

                        5 -> {
                            onMoveAnchor?.invoke(false, -1)
                            mode.finish()
                            showSelectionToolbar(
                                loRow,
                                hiRow,
                                loCol,
                                hiCol,
                                onCopy,
                                onSelectAll,
                                onOpenUrl,
                                selectedText,
                                themeFgColor,
                                themeBgColor,
                                onMoveAnchor,
                            )
                        }

                        6 -> {
                            onMoveAnchor?.invoke(true, 1)
                            mode.finish()
                            showSelectionToolbar(
                                loRow,
                                hiRow,
                                loCol,
                                hiCol,
                                onCopy,
                                onSelectAll,
                                onOpenUrl,
                                selectedText,
                                themeFgColor,
                                themeBgColor,
                                onMoveAnchor,
                            )
                        }
                    }
                    return true
                }

                override fun onDestroyActionMode(mode: android.view.ActionMode) {
                    actionMode = null
                }

                override fun onGetContentRect(
                    mode: android.view.ActionMode,
                    view: View,
                    outRect: android.graphics.Rect,
                ) {
                    outRect.set(selectionRect)
                    val toolbarHeight = (48 * resources.displayMetrics.density).toInt()
                    outRect.top = (selectionRect.top - toolbarHeight).coerceAtLeast(0)
                    outRect.bottom = selectionRect.top
                }
            }

        actionMode = startActionMode(callback, android.view.ActionMode.TYPE_FLOATING)
    }

    fun hideSelectionToolbar() {
        actionMode?.finish()
        actionMode = null
        toolbarPopup?.dismiss()
        toolbarPopup = null
        pendingUrl = null
    }

    fun showPastePopup(
        row: Int,
        col: Int,
    ) {
        hidePastePopup()

        val startPxX = (col * cellWidth).toInt()
        val startPxY = (row * cellHeight).toInt()
        val endPxX = ((col + 1) * cellWidth).toInt()
        val endPxY = ((row + 1) * cellHeight).toInt()

        val loc = IntArray(2)
        getLocationInWindow(loc)
        selectionRect =
            android.graphics.Rect(
                loc[0] + startPxX,
                loc[1] + startPxY,
                loc[0] + endPxX,
                loc[1] + endPxY,
            )

        val callback =
            object : android.view.ActionMode.Callback2() {
                override fun onCreateActionMode(
                    mode: android.view.ActionMode,
                    menu: android.view.Menu,
                ): Boolean {
                    menu.add(0, 3, 0, "Paste")
                    return true
                }

                override fun onPrepareActionMode(
                    mode: android.view.ActionMode,
                    menu: android.view.Menu,
                ): Boolean = false

                override fun onActionItemClicked(
                    mode: android.view.ActionMode,
                    item: android.view.MenuItem,
                ): Boolean {
                    if (item.itemId == 3) {
                        mode.finish()
                        pasteFromClipboardDirect()
                    }
                    return true
                }

                override fun onDestroyActionMode(mode: android.view.ActionMode) {
                    pasteActionMode = null
                }

                override fun onGetContentRect(
                    mode: android.view.ActionMode,
                    view: View,
                    outRect: android.graphics.Rect,
                ) {
                    outRect.set(selectionRect)
                }
            }

        pasteActionMode = startActionMode(callback, android.view.ActionMode.TYPE_FLOATING)
    }

    fun hidePastePopup() {
        pasteActionMode?.finish()
        pasteActionMode = null
        hideCursorHandle()
    }

    fun showCursorHandle(
        row: Int,
        col: Int,
        themeFgColor: Int = 0xFF2196F3.toInt(),
    ) {
        hideCursorHandle()
        val loc = IntArray(2)
        getLocationInWindow(loc)

        val cursorX = (col * cellWidth).toInt()
        val cursorY =
            ((row + 1) * cellHeight)
                .toInt()
                .coerceAtMost(surfaceHeightPixels - 48)

        val cursorDrawable =
            androidx.core.content.ContextCompat
                .getDrawable(context, R.drawable.text_select_handle_left_material)!!
                .mutate()
        cursorDrawable.setTint(themeFgColor)
        val handleW = cursorDrawable.intrinsicWidth
        val handleH = cursorDrawable.intrinsicHeight

        val cursorView = createHandleViewWithDrawable(cursorDrawable, HandleDrag.NONE)
        val popupX =
            (loc[0] + cursorX - handleW / 2)
                .coerceIn(loc[0], (loc[0] + surfaceWidthPixels - handleW).coerceAtLeast(loc[0]))

        cursorHandlePopup =
            createHandlePopup(cursorView).apply {
                showAtLocation(this@TerminalSurface, 0, popupX, loc[1] + cursorY)
            }
    }

    fun hideCursorHandle() {
        cursorHandlePopup?.dismiss()
        cursorHandlePopup = null
    }

    private fun createHandleViewWithDrawable(
        drawable: android.graphics.drawable.Drawable,
        which: HandleDrag,
    ): View = object : View(context) {
        override fun onMeasure(
            widthMeasureSpec: Int,
            heightMeasureSpec: Int,
        ) {
            setMeasuredDimension(drawable.intrinsicWidth, drawable.intrinsicHeight)
        }

        override fun onDraw(canvas: android.graphics.Canvas) {
            val drawableWidth = drawable.intrinsicWidth
            val drawableHeight = drawable.intrinsicHeight
            drawable.setBounds(0, 0, drawableWidth, drawableHeight)
            drawable.draw(canvas)
        }

        @SuppressLint("ClickableViewAccessibility")
        override fun onTouchEvent(event: MotionEvent): Boolean {
            if (which == HandleDrag.NONE) return super.onTouchEvent(event)
            val loc = IntArray(2)
            getLocationOnScreen(loc)
            val localX = event.rawX - loc[0]
            val localY = event.rawY - loc[1]
            when (event.actionMasked) {
                MotionEvent.ACTION_DOWN -> {
                    handleDragState = which
                    val col = (localX / cellWidth).toInt().coerceIn(0, (cols - 1).coerceAtLeast(0))
                    val row = (localY / cellHeight).toInt().coerceIn(0, (rows - 1).coerceAtLeast(0))
                    if (which == HandleDrag.START) {
                        viewModel?.updateSelectionStart(row, col)
                    } else {
                        viewModel?.updateSelection(row, col)
                    }
                    return true
                }

                MotionEvent.ACTION_MOVE -> {
                    val col = (localX / cellWidth).toInt().coerceIn(0, (cols - 1).coerceAtLeast(0))
                    val row = (localY / cellHeight).toInt().coerceIn(0, (rows - 1).coerceAtLeast(0))
                    if (which == HandleDrag.START) {
                        viewModel?.updateSelectionStart(row, col)
                    } else {
                        viewModel?.updateSelection(row, col)
                    }
                    return true
                }

                MotionEvent.ACTION_UP, MotionEvent.ACTION_CANCEL -> {
                    viewModel?.endSelection()
                    handleDragState = HandleDrag.NONE
                    return true
                }
            }
            return super.onTouchEvent(event)
        }
    }

    private fun createHandlePopup(contentView: View): PopupWindow {
        val popup = PopupWindow(context, null, android.R.attr.textSelectHandleWindowStyle)
        popup.setSplitTouchEnabled(true)
        popup.setClippingEnabled(false)
        popup.setWidth(android.view.ViewGroup.LayoutParams.WRAP_CONTENT)
        popup.setHeight(android.view.ViewGroup.LayoutParams.WRAP_CONTENT)
        popup.setBackgroundDrawable(null)
        popup.setAnimationStyle(0)
        if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.M) {
            popup.setWindowLayoutType(android.view.WindowManager.LayoutParams.TYPE_APPLICATION_SUB_PANEL)
            popup.setEnterTransition(null)
            popup.setExitTransition(null)
        }
        popup.setContentView(contentView)
        return popup
    }

    fun hideSelectionHandles() {
        startHandlePopup?.dismiss()
        startHandlePopup = null
        endHandlePopup?.dismiss()
        endHandlePopup = null
        hideSelectionToolbar()
        hideCursorHandle()
    }

    val cellWidth: Float get() = if (cols > 0 && surfaceWidthPixels > 0) surfaceWidthPixels.toFloat() / cols else 8f
    val cellHeight: Float get() = if (rows > 0 && surfaceHeightPixels > 0) surfaceHeightPixels.toFloat() / rows else 16f

    @JvmField
    var isAfterLongPress = false

    var lastTapTime = 0L

    @JvmField
    var scaleFactor = 1.0f

    private enum class HandleDrag { NONE, START, END }

    private var handleDragState = HandleDrag.NONE

    val isSelectingText: Boolean
        get() =
            viewModel
                ?.state
                ?.value
                ?.selection
                ?.active == true

    private val gestureListener =
        object : GestureDetector.SimpleOnGestureListener() {
            override fun onDown(e: MotionEvent): Boolean = true

            override fun onShowPress(e: MotionEvent) {
                isAfterLongPress = false
            }

            override fun onScroll(
                e1: MotionEvent?,
                e2: MotionEvent,
                distanceX: Float,
                distanceY: Float,
            ): Boolean {
                if (isSelectingText) return false
                if (!isScrolling) {
                    isScrolling = true
                    onScrollingStateChanged?.invoke(true)
                }
                val scrollAmount = (distanceY / cellHeight).toInt().coerceAtLeast(1)
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
                if (isSelectingText) return false
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

            override fun onSingleTapUp(event: MotionEvent): Boolean {
                if (isAfterLongPress) {
                    isAfterLongPress = false
                    return true
                }
                if (isScrolling) {
                    isScrolling = false
                    scrollOffset = 0
                    onScrollChanged?.invoke(0)
                    onScrollingStateChanged?.invoke(false)
                    return true
                }
                if (isSelectingText) {
                    hidePastePopup()
                    viewModel?.clearSelection()
                    return true
                }
                hidePastePopup()
                viewModel?.clearSelection()
                keyboardRequested = true
                requestFocus()
                post {
                    val imm = context.getSystemService(Context.INPUT_METHOD_SERVICE) as InputMethodManager
                    imm.showSoftInput(this@TerminalSurface, InputMethodManager.SHOW_IMPLICIT)
                }
                return true
            }

            override fun onDoubleTap(event: MotionEvent): Boolean {
                if (isSelectingText) {
                    viewModel?.clearSelection()
                    return true
                }
                val now = System.currentTimeMillis()
                if (now - lastTapTime < 400) {
                    val col = (event.x / cellWidth).toInt().coerceIn(0, (cols - 1).coerceAtLeast(0))
                    val row = (event.y / cellHeight).toInt().coerceIn(0, (rows - 1).coerceAtLeast(0))
                    viewModel?.setSelectionMode(SelectionMode.Line)
                    viewModel?.startSelection(row, 0)
                    val bridge = viewModel?.runtime?.bridge()
                    val scrollbackLength = bridge?.scrollbackLength()?.toInt() ?: 0
                    val line = bridge?.scrollbackLine((scrollbackLength + row).toUInt()) ?: ""
                    viewModel?.updateSelection(row, line.length.coerceAtLeast(0))
                    viewModel?.endSelection()
                } else {
                    startSelectionAt(event, expandToWord = true)
                }
                lastTapTime = now
                return true
            }

            override fun onLongPress(event: MotionEvent) {
                if (scaleFactor < 0.9f || scaleFactor > 1.1f) return
                isAfterLongPress = true
                Log.d(TAG, "onLongPress: x=${event.x} y=${event.y} cellW=$cellWidth cellH=$cellHeight cols=$cols rows=$rows")
                val col = (event.x / cellWidth).toInt().coerceIn(0, (cols - 1).coerceAtLeast(0))
                val row = (event.y / cellHeight).toInt().coerceIn(0, (rows - 1).coerceAtLeast(0))
                val bridge = viewModel?.runtime?.bridge()
                val scrollbackLength = bridge?.scrollbackLength()?.toInt() ?: 0
                val line = bridge?.scrollbackLine((scrollbackLength + row).toUInt()) ?: ""
                Log.d(TAG, "onLongPress: col=$col row=$row lineLen=${line.length}")
                handleLongPress(event.x, event.y)
            }
        }

    private val gestureDetector = GestureDetector(context, gestureListener)

    private val scaleDetector =
        ScaleGestureDetector(
            context,
            object : ScaleGestureDetector.SimpleOnScaleGestureListener() {
                override fun onScaleBegin(detector: ScaleGestureDetector): Boolean {
                    if (isSelectingText) return false
                    return true
                }

                override fun onScale(detector: ScaleGestureDetector): Boolean {
                    if (isSelectingText) return false
                    scaleFactor *= detector.scaleFactor
                    if (scaleFactor < 0.9f || scaleFactor > 1.1f) {
                        val increase = scaleFactor > 1.0f
                        onZoomChanged?.invoke(increase)
                        scaleFactor = 1.0f
                    }
                    return true
                }
            },
        )

    fun handleLongPress(
        x: Float,
        y: Float,
    ) {
        if (scaleFactor < 0.9f || scaleFactor > 1.1f) return
        isAfterLongPress = true

        @Suppress("DEPRECATION")
        performHapticFeedback(android.view.HapticFeedbackConstants.LONG_PRESS)

        hideSelectionHandles()
        hidePastePopup()

        val col = (x / cellWidth).toInt().coerceIn(0, (cols - 1).coerceAtLeast(0))
        val row = (y / cellHeight).toInt().coerceIn(0, (rows - 1).coerceAtLeast(0))
        val bridge = viewModel?.runtime?.bridge()
        val scrollbackLength = bridge?.scrollbackLength()?.toInt() ?: 0
        val line = bridge?.scrollbackLine((scrollbackLength + row).toUInt()) ?: ""
        val isOnEmptyArea = col >= line.length || line.isEmpty()
        if (isOnEmptyArea) {
            viewModel?.clearSelection()
            hideSelectionHandles()
            hidePastePopup()
            val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
            val themeAccent = 0xFF2196F3.toInt()
            showCursorHandle(row, col, themeAccent)
            if (clipboard.hasPrimaryClip()) {
                showPastePopup(row, col)
            }
        } else {
            hidePastePopup()
            val (startCol, endCol) = expandSmartSelection(row, col, line)
            viewModel?.startSelection(row, startCol)
            viewModel?.updateSelection(row, endCol)
            viewModel?.endSelection()
            showSelectionHandles(row, startCol, row, endCol)
        }
    }

    private var cachedSurface: android.view.Surface? = null

    private var lastTouchX = 0f
    private var lastTouchY = 0f

    override fun performLongClick(): Boolean {
        val longPressX = if (lastTouchX > 0f) lastTouchX else width / 2f
        val longPressY = if (lastTouchY > 0f) lastTouchY else height * 0.85f
        Log.d(TAG, "performLongClick: x=$longPressX y=$longPressY lastTouch=($lastTouchX,$lastTouchY) size=(${width}x$height)")
        if (lastTouchX <= 0f && lastTouchY <= 0f) {
            viewModel?.selectAll()
            return true
        }
        handleLongPress(longPressX, longPressY)
        return true
    }

    init {
        surfaceTextureListener = this
        isFocusable = true
        isFocusableInTouchMode = true
        isLongClickable = true
        scaleDetector.isQuickScaleEnabled = false
        contentDescription = context.getString(R.string.terminal)
        importantForAccessibility = IMPORTANT_FOR_ACCESSIBILITY_YES
    }

    private var keyboardRequested = false

    override fun onCheckIsTextEditor(): Boolean = keyboardRequested

    override fun onWindowFocusChanged(hasFocus: Boolean) {
        super.onWindowFocusChanged(hasFocus)
        viewModel?.runtime?.focusChange(hasFocus)
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

    private var coalescer = InputCoalescer()

    override fun onCreateInputConnection(outAttrs: EditorInfo): InputConnection {
        val mode = viewModel?.state?.value?.keyboardMode ?: "secure"
        when (mode) {
            "raw" -> {
                outAttrs.inputType = android.text.InputType.TYPE_NULL
                outAttrs.imeOptions =
                    EditorInfo.IME_FLAG_NO_ENTER_ACTION or EditorInfo.IME_ACTION_NONE
            }

            "standard" -> {
                outAttrs.inputType =
                    android.text.InputType.TYPE_CLASS_TEXT or
                    android.text.InputType.TYPE_TEXT_FLAG_AUTO_CORRECT
                outAttrs.imeOptions =
                    EditorInfo.IME_FLAG_NO_ENTER_ACTION or EditorInfo.IME_ACTION_NONE
            }

            else -> {
                outAttrs.inputType =
                    android.text.InputType.TYPE_CLASS_TEXT or
                    android.text.InputType.TYPE_TEXT_FLAG_NO_SUGGESTIONS
                outAttrs.imeOptions =
                    EditorInfo.IME_FLAG_NO_ENTER_ACTION or
                    EditorInfo.IME_FLAG_NO_PERSONALIZED_LEARNING or
                    EditorInfo.IME_FLAG_NO_EXTRACT_UI
            }
        }
        return object : BaseInputConnection(this, false) {
            override fun commitText(
                text: CharSequence?,
                newCursorPosition: Int,
            ): Boolean {
                val committedText = text?.toString() ?: return false
                if (!coalescer.shouldCommit(committedText)) {
                    return true
                }
                val terminalViewModel = viewModel
                val state = terminalViewModel?.state?.value
                val ctrlActive =
                    state?.ctrlState?.let {
                        it == ModifierState.Locked || it == ModifierState.Once
                    } == true
                val altActive =
                    state?.altState?.let {
                        it == ModifierState.Locked || it == ModifierState.Once
                    } == true
                terminalViewModel?.writeToPty(
                    TerminalInputEncoder.encodeCommittedText(
                        text = committedText,
                        ctrlActive = ctrlActive,
                        altActive = altActive,
                        bracketedPaste = false,
                    ),
                )
                terminalViewModel?.consumeOneShotModifiers()
                return true
            }

            override fun sendKeyEvent(event: KeyEvent): Boolean = if (event.action == KeyEvent.ACTION_DOWN) {
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

    fun pasteFromClipboardDirect() {
        val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
        if (!clipboard.hasPrimaryClip()) return
        val clip = clipboard.primaryClip?.getItemAt(0)?.text ?: return
        val data = clip.toString().replace("\n", "\r").toByteArray()
        viewModel?.writeToPty(data)
    }

    fun expandSmartSelection(
        row: Int,
        col: Int,
        line: String,
    ): Pair<Int, Int> {
        if (col >= line.length) return Pair(col, col)

        val urlBounds = expandUrlSelection(line, col)
        if (urlBounds != null) return urlBounds

        return expandWordSelection(row, col)
    }

    private fun expandUrlSelection(
        line: String,
        col: Int,
    ): Pair<Int, Int>? {
        val urlPattern =
            Regex(
                "(https?://[^\\s<>\"'`\\\\|\\[\\]{}]+|www\\.[^\\s<>\"'`\\\\|\\[\\]{}]+)",
            )
        for (match in urlPattern.findAll(line)) {
            if (col in match.range) {
                return Pair(match.range.first, match.range.last)
            }
        }
        return null
    }

    @Suppress("CyclomaticComplexMethod", "NestedBlockDepth")
    fun expandWordSelection(
        row: Int,
        col: Int,
    ): Pair<Int, Int> {
        val bridge = viewModel?.runtime?.bridge() ?: return Pair(col, col)
        val scrollbackLength = bridge.scrollbackLength().toInt()
        val line = bridge.scrollbackLine((scrollbackLength + row).toUInt()) ?: return Pair(col, col)
        if (col >= line.length) return Pair(col, col)
        val ch = line[col]
        if (ch == ' ' || ch == '\t') {
            var lookLeft = col - 1
            while (lookLeft >= 0 && (line[lookLeft] == ' ' || line[lookLeft] == '\t')) lookLeft--
            var lookRight = col + 1
            while (lookRight < line.length && (line[lookRight] == ' ' || line[lookRight] == '\t')) lookRight++
            val target =
                when {
                    lookLeft >= 0 && lookRight < line.length -> {
                        minOf(col - lookLeft, lookRight - col).let {
                            if (col - lookLeft <= lookRight - col) lookLeft else lookRight
                        }
                    }

                    lookLeft >= 0 -> {
                        lookLeft
                    }

                    lookRight < line.length -> {
                        lookRight
                    }

                    else -> {
                        return Pair(col, col)
                    }
                }
            return expandWordSelection(row, target)
        }

        var startCol = col
        while (startCol > 0) {
            val character = line[startCol - 1]
            if (character == ' ' || character == '\t') break
            startCol--
        }
        var endCol = col
        while (endCol < line.length - 1) {
            val character = line[endCol + 1]
            if (character == ' ' || character == '\t') break
            endCol++
        }
        return Pair(startCol, endCol)
    }

    private fun startSelectionAt(
        event: MotionEvent,
        expandToWord: Boolean = false,
    ) {
        val col = (event.x / cellWidth).toInt().coerceIn(0, (cols - 1).coerceAtLeast(0))
        val row = (event.y / cellHeight).toInt().coerceIn(0, (rows - 1).coerceAtLeast(0))
        Log.d(TAG, "startSelectionAt: col=$col row=$row expand=$expandToWord vm=${viewModel != null}")

        if (expandToWord) {
            val (startCol, endCol) = expandWordSelection(row, col)
            Log.d(TAG, "startSelectionAt: wordStart=$startCol wordEnd=$endCol")
            viewModel?.startSelection(row, startCol)
            viewModel?.updateSelection(row, endCol)
            viewModel?.endSelection()
        } else {
            viewModel?.startSelection(row, col)
        }

        try {
            magnifier = magnifier ?: Magnifier.Builder(this@TerminalSurface).build()
            magnifier?.show(event.rawX, event.rawY)
        } catch (_: Exception) {
        }
    }

    override fun onKeyDown(
        keyCode: Int,
        event: KeyEvent,
    ): Boolean {
        val terminalViewModel = viewModel
        val state = terminalViewModel?.state?.value
        val ctrlActive =
            event.isCtrlPressed || state?.ctrlState?.let {
                it == ModifierState.Locked || it == ModifierState.Once
            } == true
        val altActive =
            event.isAltPressed || state?.altState?.let {
                it == ModifierState.Locked || it == ModifierState.Once
            } == true
        val encodedInput =
            TerminalInputEncoder.encodeKeyEvent(
                keyCode = keyCode,
                unicodeChar = event.unicodeChar,
                ctrlActive = ctrlActive,
                altActive = altActive,
            )
        if (encodedInput != null) {
            terminalViewModel?.writeToPty(encodedInput)
            terminalViewModel?.consumeOneShotModifiers()
            return true
        }
        return super.onKeyDown(keyCode, event)
    }

    override fun onKeyUp(
        keyCode: Int,
        event: KeyEvent,
    ): Boolean = true

    private fun handleKeyEvent(event: KeyEvent): Boolean = onKeyDown(event.keyCode, event)

    @Suppress("CyclomaticComplexMethod", "LongMethod", "NestedBlockDepth")
    override fun onTouchEvent(event: MotionEvent): Boolean {
        if (event.action == MotionEvent.ACTION_DOWN) {
            lastTouchX = event.x
            lastTouchY = event.y
        }
        if (drawerOpen && event.x < drawerWidthPixels) {
            Log.d(TAG, "onTouchEvent: passing through drawer touch at x=${event.x}")
            return false
        }

        val fromMouse = event.isFromSource(InputDevice.SOURCE_MOUSE)

        if (fromMouse) {
            when {
                event.isButtonPressed(MotionEvent.BUTTON_SECONDARY) -> {
                    if (event.action == MotionEvent.ACTION_DOWN) {
                        viewModel?.clearSelection()
                        startSelectionAt(event, expandToWord = true)
                    }
                    return true
                }

                event.isButtonPressed(MotionEvent.BUTTON_TERTIARY) -> {
                    if (event.action == MotionEvent.ACTION_DOWN) {
                        pasteFromClipboardDirect()
                    }
                    return true
                }
            }
        }

        scaleDetector.onTouchEvent(event)
        gestureDetector.onTouchEvent(event)

        when (event.actionMasked) {
            MotionEvent.ACTION_DOWN -> {
                if (isSelectingText) {
                    val sel = viewModel?.state?.value?.selection
                    if (sel != null && sel.start != null && sel.end != null) {
                        val touchRow = (event.y / cellHeight).toInt()
                        val startCol = minOf(sel.start.col, sel.end.col)
                        val endCol = maxOf(sel.start.col, sel.end.col)
                        val loRow = minOf(sel.start.row, sel.end.row)
                        val hiRow = maxOf(sel.start.row, sel.end.row)
                        val startPx = startCol * cellWidth
                        val endPx = (endCol + 1) * cellWidth
                        val inRowRange = touchRow in (loRow - 1)..(hiRow + 1)
                        val touchRadius = cellWidth * 1.5f
                        if (inRowRange && kotlin.math.abs(event.x - startPx) < touchRadius) {
                            handleDragState = HandleDrag.START
                            viewModel?.updateSelectionStart(touchRow.coerceIn(loRow, hiRow), startCol)
                            return true
                        } else if (inRowRange && kotlin.math.abs(event.x - endPx) < touchRadius) {
                            handleDragState = HandleDrag.END
                            viewModel?.updateSelection(touchRow.coerceIn(loRow, hiRow), endCol)
                            return true
                        } else {
                            viewModel?.clearSelection()
                        }
                    }
                }
            }

            MotionEvent.ACTION_MOVE -> {
                if (isSelectingText && handleDragState != HandleDrag.NONE) {
                    val col = (event.x / cellWidth).toInt().coerceIn(0, (cols - 1).coerceAtLeast(0))
                    val row = (event.y / cellHeight).toInt().coerceIn(0, (rows - 1).coerceAtLeast(0))
                    if (handleDragState == HandleDrag.START) {
                        viewModel?.updateSelectionStart(row, col)
                    } else if (handleDragState == HandleDrag.END) {
                        viewModel?.updateSelection(row, col)
                    }
                }
            }

            MotionEvent.ACTION_UP, MotionEvent.ACTION_CANCEL -> {
                if (isSelectingText && handleDragState != HandleDrag.NONE) {
                    viewModel?.endSelection()
                }
                handleDragState = HandleDrag.NONE
                try {
                    magnifier?.dismiss()
                } catch (_: Exception) {
                }
                magnifier = null
                scaleFactor = 1.0f
            }
        }
        return true
    }

    // ── SurfaceTextureListener ─────────────────────────────────────────

    override fun onSizeChanged(
        width: Int,
        height: Int,
        previousWidth: Int,
        previousHeight: Int,
    ) {
        super.onSizeChanged(width, height, previousWidth, previousHeight)
        Log.d(TAG, "onSizeChanged: $width x $height (was ${previousWidth}x$previousHeight)")
        if (width <= 0 || height <= 0) return
        if (width == previousWidth && height == previousHeight && previousWidth != 0) return

        if (surfaceTexture != null) {
            surfaceTexture!!.setDefaultBufferSize(width, height)
        }

        surfaceWidthPixels = width
        surfaceHeightPixels = height
        viewModel?.let { terminalViewModel ->
            terminalViewModel.surfaceWidth = width
            terminalViewModel.surfaceHeight = height
            val surface =
                cachedSurface ?: (
                    surfaceTexture?.let {
                        Surface(it).also {
                            cachedSurface = it
                            terminalViewModel.currentSurface = it
                        }
                    }
                    )
            if (surface != null) {
                terminalViewModel.currentSurface = surface
                val windowPointer = terminalViewModel.runtime.getNativeWindowPtr(surface)
                if (windowPointer != 0L) {
                    terminalViewModel.runtime.updateNativeWindow(windowPointer, width, height)
                } else {
                    Log.w(TAG, "onSizeChanged: windowPointer=0, skipping resize")
                }
            } else {
                Log.w(TAG, "onSizeChanged: no cachedSurface, skipping resize")
            }
        }
    }

    override fun onSurfaceTextureAvailable(
        surfaceTexture: SurfaceTexture,
        width: Int,
        height: Int,
    ) {
        Log.d(TAG, "onSurfaceTextureAvailable: $width x $height")
        val textureSurface = Surface(surfaceTexture).also { cachedSurface = it }
        surfaceWidthPixels = width
        surfaceHeightPixels = height
        viewModel?.let { terminalViewModel ->
            terminalViewModel.surfaceWidth = width
            terminalViewModel.surfaceHeight = height
            terminalViewModel.currentSurface = textureSurface
            val isRunning = terminalViewModel.runtime.state.value.isRunning
            if (!isRunning) {
                terminalViewModel.startRuntime(textureSurface, width, height)
            } else {
                val windowPointer = terminalViewModel.runtime.getNativeWindowPtr(textureSurface)
                terminalViewModel.runtime.updateNativeWindow(windowPointer, width, height)
            }
        }
    }

    override fun onSurfaceTextureSizeChanged(
        surfaceTexture: SurfaceTexture,
        width: Int,
        height: Int,
    ) {
        Log.d(TAG, "onSurfaceTextureSizeChanged: $width x $height")
        surfaceWidthPixels = width
        surfaceHeightPixels = height
        viewModel?.let { terminalViewModel ->
            terminalViewModel.surfaceWidth = width
            terminalViewModel.surfaceHeight = height
            try {
                val surface =
                    cachedSurface ?: Surface(surfaceTexture).also {
                        cachedSurface = it
                        terminalViewModel.currentSurface = it
                    }
                terminalViewModel.currentSurface = surface
                val windowPointer = terminalViewModel.runtime.getNativeWindowPtr(surface)
                if (windowPointer != 0L) {
                    terminalViewModel.runtime.updateNativeWindow(windowPointer, width, height)
                }
            } catch (exception: Exception) {
                Log.e(TAG, "onSurfaceTextureSizeChanged failed", exception)
            }
        }
    }

    override fun onSurfaceTextureDestroyed(surfaceTexture: SurfaceTexture): Boolean {
        Log.d(TAG, "onSurfaceTextureDestroyed")
        cachedSurface = null
        viewModel?.let { terminalViewModel ->
            terminalViewModel.currentSurface = null
            terminalViewModel.runtime.pauseRendering()
        }
        return true
    }

    override fun onSurfaceTextureUpdated(surfaceTexture: SurfaceTexture) {
    }
}
