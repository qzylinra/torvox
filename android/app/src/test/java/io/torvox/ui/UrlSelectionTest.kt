package io.torvox.ui

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Test

class UrlSelectionTest {
    data class UrlSpan(
        val startRow: Int,
        val startCol: Int,
        val endRow: Int,
        val endCol: Int,
    )

    /**
     * Detects whether a URL-like word on one row continues on the next row.
     *
     * Rules:
     * - The anchor row must contain a URL-like word (scheme, .com, .git, .org, .io, www.)
     * - The next row must have a short (< 20 chars) single continuation word that
     *   looks like a URL fragment (starts with lowercase, /, ., -, +, ~, #, &, or %)
     * - Two-word continuations are NEVER accepted (they're always prose)
     * - Long continuations (> 20 chars) are NEVER accepted (they fill the line)
     * - Indentation is NOT required (continuations can start at column 0)
     */
    private fun expandAcrossUrlWrap(
        lines: List<String>,
        row: Int,
        wordStartCol: Int,
        wordEndCol: Int,
    ): UrlSpan? {
        if (row < 0 || row >= lines.size) return null
        val currentLine = lines[row]
        if (wordStartCol < 0 || wordEndCol > currentLine.length) return null
        if (wordStartCol >= wordEndCol) return null

        val word = currentLine.substring(wordStartCol, wordEndCol)

        // Must look URL-like
        if (!word.contains("://") && !word.startsWith("www.") &&
            !word.contains(".com") && !word.contains(".git") &&
            !word.contains(".org") && !word.contains(".io")
        ) {
            return null
        }

        val nextRow = row + 1
        if (nextRow >= lines.size) return null

        val nextLine = lines[nextRow]
        val trimmed = nextLine.trimStart()
        if (trimmed.isEmpty()) return null

        val indent = nextLine.length - nextLine.trimStart().length
        val words = trimmed.split("\\s+".toRegex())

        // Only accept single-word continuations
        if (words.size != 1) return null

        // Continuation must be short (< 20 chars)
        val continuationWord = words[0]
        if (continuationWord.length >= 20) return null

        // URL continuation must look like a genuine fragment
        if (!continuationWord[0].isLowerCase() && continuationWord[0] != '/' &&
            continuationWord[0] != '.' && continuationWord[0] != '-' &&
            continuationWord[0] != '+' && continuationWord[0] != '~' &&
            continuationWord[0] != '#' && continuationWord[0] != '&' &&
            continuationWord[0] != '%'
        ) {
            return null
        }

        return UrlSpan(
            startRow = row,
            startCol = wordStartCol,
            endRow = nextRow,
            endCol = indent + continuationWord.length,
        )
    }

    /**
     * Given a selection range, checks if the selection should be expanded
     * to include a URL continuation on an adjacent row.
     */
    private fun expandUrlSelection(
        lines: List<String>,
        startRow: Int,
        startCol: Int,
        endRow: Int,
        endCol: Int,
    ): UrlSpan? {
        val forward = expandAcrossUrlWrap(lines, endRow, 0, endCol)
        if (forward != null) return forward

        if (startRow > 0) {
            val prevLine = lines[startRow - 1]
            val prevTrimmed = prevLine.trimEnd()
            if (prevTrimmed.isNotEmpty()) {
                val lastSpace = prevTrimmed.lastIndexOf(' ')
                val lastWordStart = if (lastSpace < 0) 0 else lastSpace + 1
                val lastWordEnd = prevTrimmed.length
                if (lastWordEnd > lastWordStart) {
                    val backward = expandAcrossUrlWrap(lines, startRow - 1, lastWordStart, lastWordEnd)
                    if (backward != null) return backward
                }
            }
        }

        return null
    }

    // -- expandAcrossUrlWrap tests --

    @Test
    fun forwardSchemePrefixedContinuation() {
        val span =
            expandAcrossUrlWrap(
                lines = listOf("https://github.com/GlassOnTin/Haven/iss", "ues/89", "more prose"),
                row = 0,
                wordStartCol = 0,
                wordEndCol = 38,
            )
        assertNotNull(span)
        assertEquals(1, span!!.endRow)
        assertEquals(6, span.endCol)
    }

    @Test
    fun backwardFromContinuationRow() {
        val span =
            expandUrlSelection(
                lines = listOf("https://github.com/GlassOnTin/Haven/iss", "ues/89"),
                startRow = 1,
                startCol = 0,
                endRow = 1,
                endCol = 6,
            )
        assertNotNull(span)
        assertEquals(0, span!!.startRow)
        assertEquals(0, span.startCol)
        assertEquals(1, span.endRow)
        assertEquals(6, span.endCol)
    }

    @Test
    fun noWrapForAdjacentProse() {
        val span =
            expandAcrossUrlWrap(
                lines = listOf("abcdefghijklmnopqrstuvwxyzabcdefghijklmn", "opqrstuvwxyz"),
                row = 0,
                wordStartCol = 0,
                wordEndCol = 39,
            )
        assertNull(span)
    }

    @Test
    fun noWrapForMidRowWord() {
        val span =
            expandAcrossUrlWrap(
                lines = listOf("  https://example.com  ", "  next line text  "),
                row = 0,
                wordStartCol = 2,
                wordEndCol = 21,
            )
        assertNull(span)
    }

    @Test
    fun noWrapForMultiWordIndentedProse() {
        val span =
            expandAcrossUrlWrap(
                lines = listOf("https://example.com/some/pa", "  indented continuation"),
                row = 0,
                wordStartCol = 0,
                wordEndCol = 26,
            )
        assertNull(span)
    }

    @Test
    fun hangingIndentSingleCharTail() {
        val span =
            expandAcrossUrlWrap(
                lines = listOf("https://github.com/GlassOnTin/iSpindlePlotter.gi", "     t"),
                row = 0,
                wordStartCol = 0,
                wordEndCol = 47,
            )
        assertNotNull(span)
        assertEquals(1, span!!.endRow)
        assertEquals(6, span.endCol)
    }

    @Test
    fun hangingIndentMultiCharTail() {
        val span =
            expandAcrossUrlWrap(
                lines = listOf("https://github.com/GlassOnTin/Haven", "   /issues"),
                row = 0,
                wordStartCol = 0,
                wordEndCol = 34,
            )
        assertNotNull(span)
        assertEquals(1, span!!.endRow)
        assertEquals(10, span.endCol)
    }

    @Test
    fun noWrapForLongIndentedRun() {
        val span =
            expandAcrossUrlWrap(
                lines = listOf("https://example.com/aaaaaaaaaaaaaaaaaaaa", "   bbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
                row = 0,
                wordStartCol = 0,
                wordEndCol = 39,
            )
        assertNull(span)
    }

    @Test
    fun ftpScheme() {
        val span =
            expandAcrossUrlWrap(
                lines = listOf("ftp://mirror.example.com/linux/dists/stable/m", "ain"),
                row = 0,
                wordStartCol = 0,
                wordEndCol = 41,
            )
        assertNotNull(span)
        assertEquals(1, span!!.endRow)
        assertEquals(3, span.endCol)
    }

    @Test
    fun noWrapForGitSshScheme() {
        val span =
            expandAcrossUrlWrap(
                lines = listOf("git@github.com:user/repo.git", ""),
                row = 0,
                wordStartCol = 0,
                wordEndCol = 26,
            )
        assertNull(span)
    }

    @Test
    fun portNumber() {
        val span =
            expandAcrossUrlWrap(
                lines = listOf("http://localhost:8080/api/v2/resource/long/path/ex", "ample"),
                row = 0,
                wordStartCol = 0,
                wordEndCol = 43,
            )
        assertNotNull(span)
        assertEquals(1, span!!.endRow)
        assertEquals(5, span.endCol)
    }

    @Test
    fun queryParameters() {
        val span =
            expandAcrossUrlWrap(
                lines = listOf("https://example.com/search?q=terminal+emulator+android+lon", "g+query"),
                row = 0,
                wordStartCol = 0,
                wordEndCol = 46,
            )
        assertNotNull(span)
        assertEquals(1, span!!.endRow)
        assertEquals(7, span.endCol)
    }

    @Test
    fun fragment() {
        val span =
            expandAcrossUrlWrap(
                lines = listOf("https://example.com/page#section-heading-long-en", "ough"),
                row = 0,
                wordStartCol = 0,
                wordEndCol = 42,
            )
        assertNotNull(span)
        assertEquals(1, span!!.endRow)
        assertEquals(4, span.endCol)
    }

    @Test
    fun noWrapOnEmptyNextLine() {
        val span =
            expandAcrossUrlWrap(
                lines = listOf("https://example.com/lo", "", "next section"),
                row = 0,
                wordStartCol = 0,
                wordEndCol = 21,
            )
        assertNull(span)
    }

    @Test
    fun noWrapOnLastRow() {
        val span =
            expandAcrossUrlWrap(
                lines = listOf("https://example.com"),
                row = 0,
                wordStartCol = 0,
                wordEndCol = 19,
            )
        assertNull(span)
    }

    @Test
    fun noWrapForInvalidIndices() {
        val span =
            expandAcrossUrlWrap(
                lines = listOf("hello world"),
                row = -1,
                wordStartCol = 0,
                wordEndCol = 5,
            )
        assertNull(span)
    }

    @Test
    fun selectionExpandsAcrossWrappedUrl() {
        val expanded =
            expandUrlSelection(
                lines = listOf("https://github.com/GlassOnTin/Haven/iss", "ues/89"),
                startRow = 0,
                startCol = 0,
                endRow = 0,
                endCol = 38,
            )
        assertNotNull(expanded)
        assertEquals(0, expanded!!.startRow)
        assertEquals(1, expanded.endRow)
        assertEquals(6, expanded.endCol)
    }

    @Test
    fun noSelectionForNonWrappedUrl() {
        val expanded =
            expandUrlSelection(
                lines = listOf("short URL: https://example.com", "next line"),
                startRow = 0,
                startCol = 11,
                endRow = 0,
                endCol = 30,
            )
        assertNull(expanded)
    }

    @Test
    fun noSelectionForProseWrap() {
        val expanded =
            expandUrlSelection(
                lines = listOf("This is a normal prose line that wraps at the col", "umn boundary"),
                startRow = 0,
                startCol = 0,
                endRow = 0,
                endCol = 43,
            )
        assertNull(expanded)
    }

    @Test
    fun specialCharacters() {
        val span =
            expandAcrossUrlWrap(
                lines = listOf("https://example.com/path/with-dashes_underscores_and.do", "ts"),
                row = 0,
                wordStartCol = 0,
                wordEndCol = 49,
            )
        assertNotNull(span)
        assertEquals(1, span!!.endRow)
        assertEquals(2, span.endCol)
    }

    @Test
    fun percentEncoding() {
        val span =
            expandAcrossUrlWrap(
                lines = listOf("https://example.com/%E4%BD%A0%E5%A5%BD/%E4%B8%96%E7%95%8C/lo", "ng"),
                row = 0,
                wordStartCol = 0,
                wordEndCol = 44,
            )
        assertNotNull(span)
        assertEquals(1, span!!.endRow)
        assertEquals(2, span.endCol)
    }

    @Test
    fun httpsPortSubdir() {
        val span =
            expandAcrossUrlWrap(
                lines = listOf("https://localhost:8443/very/long/api/endpoint/that/wra", "ps"),
                row = 0,
                wordStartCol = 0,
                wordEndCol = 46,
            )
        assertNotNull(span)
        assertEquals(1, span!!.endRow)
        assertEquals(2, span.endCol)
    }

    @Test
    fun orgDomain() {
        val span =
            expandAcrossUrlWrap(
                lines = listOf("https://www.rust-lang.org/learn/get-started/instal", "led"),
                row = 0,
                wordStartCol = 0,
                wordEndCol = 46,
            )
        assertNotNull(span)
        assertEquals(1, span!!.endRow)
        assertEquals(3, span.endCol)
    }

    @Test
    fun ioDomain() {
        val span =
            expandAcrossUrlWrap(
                lines = listOf("https://example.io/very/long/path/that/wraps/arou", "nd"),
                row = 0,
                wordStartCol = 0,
                wordEndCol = 47,
            )
        assertNotNull(span)
        assertEquals(1, span!!.endRow)
        assertEquals(2, span.endCol)
    }

    @Test
    fun noWrapForTwoWordContinuation() {
        val span =
            expandAcrossUrlWrap(
                lines = listOf("https://example.com", "  check this out"),
                row = 0,
                wordStartCol = 0,
                wordEndCol = 19,
            )
        assertNull(span)
    }
}
