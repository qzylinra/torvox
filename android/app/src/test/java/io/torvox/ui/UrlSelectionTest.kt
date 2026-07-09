package io.torvox.ui

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Test

class UrlSelectionTest {
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
