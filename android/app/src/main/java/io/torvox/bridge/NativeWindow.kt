package io.torvox.bridge

import android.view.Surface

object NativeWindow {
    init {
        System.loadLibrary("torvox_android")
    }

    @JvmStatic
    external fun getNativeWindowPtr(surface: Surface): Long
}
