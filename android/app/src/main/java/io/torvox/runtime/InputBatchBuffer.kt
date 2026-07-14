package io.torvox.runtime

import android.view.Choreographer
import java.nio.ByteBuffer

class InputBatchBuffer(
    private val flushSink: (ByteArray) -> Unit,
    private val capacity: Int = BATCH_CAPACITY,
) {
    private var buffer: ByteBuffer = ByteBuffer.allocateDirect(capacity)
    private val frameCallback: Choreographer.FrameCallback =
        Choreographer.FrameCallback { _ -> flush() }
    private var scheduled = false

    fun write(data: ByteArray) {
        if (buffer.remaining() < data.size) {
            flushInternal()
        }
        buffer.put(data)
        scheduleFrame()
    }

    fun flush() {
        flushInternal()
    }

    private fun flushInternal() {
        buffer.flip()
        val bytes = ByteArray(buffer.remaining())
        buffer.get(bytes)
        buffer.clear()
        if (bytes.isNotEmpty()) {
            flushSink(bytes)
        }
        scheduled = false
    }

    private fun scheduleFrame() {
        if (!scheduled) {
            scheduled = true
            Choreographer.getInstance().postFrameCallback(frameCallback)
        }
    }

    fun reset() {
        buffer.clear()
        scheduled = false
    }

    companion object {
        private const val BATCH_CAPACITY = 8192
    }
}
