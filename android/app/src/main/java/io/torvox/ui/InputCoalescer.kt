package io.torvox.ui

import java.util.concurrent.atomic.AtomicLong
import java.util.concurrent.atomic.AtomicReference

/**
 * Deduplicates and coalesces IME input events.
 *
 * Some Android IMEs (especially Gboard, Samsung) double-fire commitText for
 * the same character on fast typing. This class detects and suppresses
 * duplicate commits within a configurable time window.
 *
 * Inspired by Haven's InputCoalescer pattern.
 */
class InputCoalescer(
    private val deduplicateWindowNanos: Long = DEDUP_WINDOW_NS,
) {
    companion object {
        private const val DEDUP_WINDOW_NS = 50_000_000L // 50ms
    }

    private val lastCommittedText = AtomicReference<String?>(null)
    private val lastCommitTimeNanos = AtomicLong(0L)

    private val composingText = AtomicReference<String?>(null)

    fun shouldCommit(
        text: String,
        currentTimeNanos: Long = System.nanoTime(),
    ): Boolean {
        val prevText = lastCommittedText.getAndSet(text)
        val prevTime = lastCommitTimeNanos.getAndSet(currentTimeNanos)

        val isDuplicate =
            prevText == text &&
                (currentTimeNanos - prevTime) < deduplicateWindowNanos

        return !isDuplicate
    }

    fun updateComposingText(text: String?) {
        composingText.set(text)
    }

    fun getComposingText(): String? = composingText.get()

    fun clearComposing() {
        composingText.set(null)
    }

    fun isComposing(): Boolean = composingText.get()?.isNotEmpty() == true

    fun reset() {
        lastCommittedText.set(null)
        lastCommitTimeNanos.set(0L)
        composingText.set(null)
    }
}
