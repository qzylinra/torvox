package io.torvox.monitor

import android.app.ActivityManager
import android.content.ComponentCallbacks2
import android.content.Context
import android.os.Debug
import android.util.Log
import io.torvox.runtime.LogUtil
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch

class MemoryMonitor(
    private val context: Context,
    private val scope: CoroutineScope,
    private val onCriticalMemory: (() -> Unit)? = null,
) {
    private val am = context.getSystemService(Context.ACTIVITY_SERVICE) as ActivityManager
    private val memInfo = ActivityManager.MemoryInfo()
    private var pollingJob: Job? = null

    fun startPolling(intervalMs: Long = POLL_INTERVAL_MS) {
        stopPolling()
        pollingJob =
            scope.launch(Dispatchers.Default) {
                while (isActive) {
                    checkMemory()
                    delay(intervalMs)
                }
            }
    }

    fun stopPolling() {
        pollingJob?.cancel()
        pollingJob = null
    }

    fun checkMemory() {
        am.getMemoryInfo(memInfo)
        val availMb = memInfo.availMem / BYTES_PER_MB
        val totalMb = memInfo.totalMem / BYTES_PER_MB
        val thresholdMb = memInfo.threshold / BYTES_PER_MB

        val pssKb = Debug.getPss()
        val nativeHeapMb = Debug.getNativeHeapAllocatedSize() / BYTES_PER_MB

        val availPercent = if (memInfo.totalMem > 0) ((memInfo.availMem * 100) / memInfo.totalMem).toInt() else 0

        if (memInfo.lowMemory) {
            Log.e(
                TAG,
                "LOW MEMORY: avail=$availMb MB / $totalMb MB ($availPercent%), PSS=${pssKb}KB, nativeHeap=$nativeHeapMb MB, threshold=$thresholdMb MB",
            )
            onCriticalMemory?.invoke()
        } else if (availMb < thresholdMb * LOW_MEMORY_FACTOR) {
            Log.w(TAG, "Memory pressure: avail=$availMb MB / $totalMb MB ($availPercent%), PSS=${pssKb}KB, threshold=$thresholdMb MB")
        } else {
            LogUtil.d(TAG, "Memory OK: avail=$availMb MB / $totalMb MB ($availPercent%)")
        }
    }

    @Suppress("DEPRECATION")
    fun onTrimMemory(level: Int) {
        when (level) {
            ComponentCallbacks2.TRIM_MEMORY_RUNNING_CRITICAL -> {
                Log.e(TAG, "TRIM_MEMORY_RUNNING_CRITICAL — system is killing processes")
                onCriticalMemory?.invoke()
            }

            ComponentCallbacks2.TRIM_MEMORY_RUNNING_LOW -> {
                Log.w(TAG, "TRIM_MEMORY_RUNNING_LOW")
            }

            ComponentCallbacks2.TRIM_MEMORY_RUNNING_MODERATE -> {
                Log.w(TAG, "TRIM_MEMORY_RUNNING_MODERATE")
            }

            ComponentCallbacks2.TRIM_MEMORY_UI_HIDDEN -> {
                LogUtil.d(TAG, "TRIM_MEMORY_UI_HIDDEN")
            }
        }
    }

    companion object {
        private const val TAG = "MemoryMonitor"
        private const val POLL_INTERVAL_MS = 30_000L
        private const val BYTES_PER_MB = 1024L * 1024L
        private const val LOW_MEMORY_FACTOR = 2.0f
    }
}
