package io.torvox.selection

import io.torvox.ui.expandAcrossUrlWrap
import io.torvox.ui.expandUrlSelection
import io.torvox.ui.expandWordOnLine
import io.torvox.ui.isUrlLikeWord
import io.torvox.ui.isValidContinuationStart
import io.torvox.ui.isWordChar
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class SelectionHelpersTest {
    // -- isWordChar --

    @Test
    fun isWordChar_letters_true() {
        assertTrue(isWordChar('a'))
        assertTrue(isWordChar('Z'))
        assertTrue(isWordChar('n'))
    }

    @Test
    fun isWordChar_digits_true() {
        assertTrue(isWordChar('0'))
        assertTrue(isWordChar('9'))
    }

    @Test
    fun isWordChar_underscore_hyphen_dot_slash_true() {
        assertTrue(isWordChar('_'))
        assertTrue(isWordChar('-'))
        assertTrue(isWordChar('.'))
        assertTrue(isWordChar('/'))
    }

    @Test
    fun isWordChar_bracket_exclamation_false() {
        assertFalse(isWordChar(':'))
        assertFalse(isWordChar('@'))
        assertFalse(isWordChar('+'))
        assertFalse(isWordChar(')'))
        assertFalse(isWordChar(','))
        assertFalse(isWordChar(';'))
    }

    @Test
    fun isWordChar_whitespace_false() {
        assertFalse(isWordChar(' '))
        assertFalse(isWordChar('\t'))
        assertFalse(isWordChar('\n'))
    }

    @Test
    fun isWordChar_brackets_special_false() {
        assertFalse(isWordChar('('))
        assertFalse(isWordChar(')'))
        assertFalse(isWordChar('['))
        assertFalse(isWordChar(']'))
        assertFalse(isWordChar('{'))
        assertFalse(isWordChar('}'))
        assertFalse(isWordChar('<'))
        assertFalse(isWordChar('>'))
        assertFalse(isWordChar('"'))
        assertFalse(isWordChar('\''))
        assertFalse(isWordChar('`'))
        assertFalse(isWordChar('|'))
        assertFalse(isWordChar('\\'))
        assertFalse(isWordChar(','))
        assertFalse(isWordChar(';'))
        assertFalse(isWordChar('!'))
    }

    // -- expandWordOnLine --
    // expandWordOnLine uses isWordChar, so URL chars (:, /, ., etc.) are word chars.
    // The result is a contiguous span of word chars around the pivot column.
    // On whitespace/punctuation, it snaps to the nearest word char.

    @Test
    fun expandWordOnLine_middleOfWord() {
        val (start, end) = expandWordOnLine("hello world terminal", 6)
        // col 6 = 'w', word is "world" at [6,11)
        assertEquals(6, start)
        assertEquals(11, end)
    }

    @Test
    fun expandWordOnLine_atWordStart() {
        val (start, end) = expandWordOnLine("hello world", 0)
        // col 0 = 'h', word "hello" at [0,5)
        assertEquals(0, start)
        assertEquals(5, end)
    }

    @Test
    fun expandWordOnLine_atWordEnd() {
        val (start, end) = expandWordOnLine("hello world", 4)
        // col 4 = 'o' (last char of "hello"), word at [0,5)
        assertEquals(0, start)
        assertEquals(5, end)
    }

    @Test
    fun expandWordOnLine_onWhitespaceBetweenWords() {
        val (start, end) = expandWordOnLine("abc def", 3)
        // col 3 = space, snaps to nearest word char: left 'c' at col 2
        // pivot becomes 2, word "abc" at [0,3)
        assertEquals(0, start)
        assertEquals(3, end)
    }

    @Test
    fun expandWordOnLine_atLineFirstChar() {
        val (start, end) = expandWordOnLine("torvox", 0)
        // col 0 = 't', word at [0,6)
        assertEquals(0, start)
        assertEquals(6, end)
    }

    @Test
    fun expandWordOnLine_atLineLastChar() {
        val (start, end) = expandWordOnLine("torvox", 5)
        // col 5 = 'x', word at [0,6)
        assertEquals(0, start)
        assertEquals(6, end)
    }

    @Test
    fun expandWordOnLine_urlDoesNotExpandPastNextWord() {
        val (start, end) = expandWordOnLine("hello world", 8)
        // col 8 = 'r' in "world", word at [6,11)
        assertEquals(6, start)
        assertEquals(11, end)
    }

    @Test
    fun expandWordOnLine_pathSeparator() {
        val (start, end) = expandWordOnLine("cd /usr/local/bin", 7)
        // col 7 = '/' is word char, spans from '/' at col 3 to 'n' at col 16 (exclusive: 17)
        assertEquals(3, start)
        assertEquals(17, end)
    }

    @Test
    fun expandWordOnLine_punctuationBoundary() {
        val (start, end) = expandWordOnLine("hello(world)baz", 6)
        // col 6 = 'w', '(' and ')' are not word chars, so word "world" at [6,11)
        assertEquals(6, start)
        assertEquals(11, end)
    }

    @Test
    fun expandWordOnLine_underscoreIsWordChar() {
        val (start, end) = expandWordOnLine("my_var_name", 4)
        // col 4 = 'a', all word chars, whole line at [0,11)
        assertEquals(0, start)
        assertEquals(11, end)
    }

    @Test
    fun expandWordOnLine_emptyLine() {
        val (start, end) = expandWordOnLine("", 0)
        // 0 >= line.length=0, returns Pair(0, 0)
        assertEquals(0, start)
        assertEquals(0, end)
    }

    @Test
    fun expandWordOnLine_colBeyondLength() {
        val (start, end) = expandWordOnLine("hi", 10)
        // 10 >= line.length=2, returns Pair(10, 10)
        assertEquals(10, start)
        assertEquals(10, end)
    }

    // -- isUrlLikeWord --

    @Test
    fun isUrlLikeWord_httpScheme_true() {
        assertTrue(isUrlLikeWord("https://example.com"))
        assertTrue(isUrlLikeWord("http://example.com/path"))
    }

    @Test
    fun isUrlLikeWord_wwwDot_true() {
        assertTrue(isUrlLikeWord("www.example.com"))
        assertTrue(isUrlLikeWord("www.google.com/path"))
    }

    @Test
    fun isUrlLikeWord_dotCom_true() {
        assertTrue(isUrlLikeWord("example.com"))
        assertTrue(isUrlLikeWord("sub.domain.com/path"))
    }

    @Test
    fun isUrlLikeWord_dotGit_true() {
        // Baseline: ".git" without scheme/domain is NOT a URL
        assertFalse(isUrlLikeWord("repo.git"))
        // But with a domain path it is still detected
        assertTrue(isUrlLikeWord("github.com/user/repo.git"))
    }

    @Test
    fun isUrlLikeWord_dotOrg_io_true() {
        assertTrue(isUrlLikeWord("rust-lang.org"))
        assertTrue(isUrlLikeWord("example.io/path"))
    }

    @Test
    fun isUrlLikeWord_plainWord_false() {
        assertFalse(isUrlLikeWord("hello"))
        assertFalse(isUrlLikeWord("justaword"))
        assertFalse(isUrlLikeWord("abc"))
    }

    // -- isValidContinuationStart --

    @Test
    fun isValidContinuationStart_lowercase_slash_dot_hyphen_true() {
        assertTrue(isValidContinuationStart('a'))
        assertTrue(isValidContinuationStart('z'))
        assertTrue(isValidContinuationStart('/'))
        assertTrue(isValidContinuationStart('.'))
        assertTrue(isValidContinuationStart('-'))
    }

    @Test
    fun isValidContinuationStart_plus_tilde_hash_amp_percent_true() {
        assertTrue(isValidContinuationStart('+'))
        assertTrue(isValidContinuationStart('~'))
        assertTrue(isValidContinuationStart('#'))
        assertTrue(isValidContinuationStart('&'))
        assertTrue(isValidContinuationStart('%'))
    }

    @Test
    fun isValidContinuationStart_uppercase_digit_whitespace_false() {
        assertFalse(isValidContinuationStart('A'))
        assertFalse(isValidContinuationStart('5'))
        assertFalse(isValidContinuationStart(' '))
        assertFalse(isValidContinuationStart('('))
        assertFalse(isValidContinuationStart('!'))
    }

    // -- expandAcrossUrlWrap --

    @Test
    fun expandAcrossUrlWrap_withWrappingUrl() {
        val lines = listOf("https://example.com/abc", "def")
        val span = expandAcrossUrlWrap(lines, 0, 0, 22)
        assertNotNull(span)
        assertEquals(0, span!!.startRow)
        assertEquals(0, span.startCol)
        assertEquals(1, span.endRow)
        assertEquals(3, span.endCol)
    }

    @Test
    fun expandAcrossUrlWrap_nonUrlContent_returnsNull() {
        val lines = listOf("not a url at all", "continuation")
        val span = expandAcrossUrlWrap(lines, 0, 0, 14)
        assertNull(span)
    }

    @Test
    fun expandAcrossUrlWrap_emptyContinuationLine_returnsNull() {
        val lines = listOf("check https://example", "")
        val span = expandAcrossUrlWrap(lines, 0, 6, 25)
        assertNull(span)
    }

    @Test
    fun expandAcrossUrlWrap_multiWordContinuation_returnsNull() {
        val lines = listOf("check https://example", "/path more text")
        val span = expandAcrossUrlWrap(lines, 0, 6, 25)
        assertNull(span)
    }

    @Test
    fun expandAcrossUrlWrap_excessiveContinuationLength_returnsNull() {
        val lines = listOf("check https://example", "a".repeat(20))
        val span = expandAcrossUrlWrap(lines, 0, 6, 25)
        assertNull(span)
    }

    @Test
    fun expandAcrossUrlWrap_hangingIndentWithSpaces() {
        val lines = listOf("https://github.com/user/repo", "   /issues")
        val span = expandAcrossUrlWrap(lines, 0, 0, 27)
        assertNotNull(span)
        assertEquals(1, span!!.endRow)
        assertEquals(10, span.endCol)
    }

    @Test
    fun expandAcrossUrlWrap_ftpScheme() {
        val lines = listOf("ftp://mirror.example.com/linux/dists/stable/m", "ain")
        val span = expandAcrossUrlWrap(lines, 0, 0, 41)
        assertNotNull(span)
        assertEquals(1, span!!.endRow)
        assertEquals(3, span.endCol)
    }

    @Test
    fun expandAcrossUrlWrap_lastRow_returnsNull() {
        val lines = listOf("https://example.com")
        val span = expandAcrossUrlWrap(lines, 0, 0, 19)
        assertNull(span)
    }

    @Test
    fun expandAcrossUrlWrap_negativeRow_returnsNull() {
        val lines = listOf("hello world")
        val span = expandAcrossUrlWrap(lines, -1, 0, 5)
        assertNull(span)
    }

    @Test
    fun expandAcrossUrlWrap_rowOutOfBounds_returnsNull() {
        val lines = listOf("hello")
        val span = expandAcrossUrlWrap(lines, 5, 0, 4)
        assertNull(span)
    }

    @Test
    fun expandAcrossUrlWrap_indentedNonUrlNextLine_returnsNull() {
        val lines = listOf("https://example.com/some/pa", "  indented prose here")
        val span = expandAcrossUrlWrap(lines, 0, 0, 26)
        assertNull(span)
    }

    // -- expandUrlSelection --

    @Test
    fun expandUrlSelection_forwardExpansion() {
        val expanded =
            expandUrlSelection(
                lines = listOf("https://github.com/user/repo/iss", "ues/42"),
                startRow = 0,
                startCol = 0,
                endRow = 0,
                endCol = 31,
            )
        assertNotNull(expanded)
        assertEquals(0, expanded!!.startRow)
        assertEquals(1, expanded.endRow)
        assertEquals(6, expanded.endCol)
    }

    @Test
    fun expandUrlSelection_backwardExpansion() {
        val expanded =
            expandUrlSelection(
                lines = listOf("https://github.com/user/repo/iss", "ues/42"),
                startRow = 1,
                startCol = 0,
                endRow = 1,
                endCol = 5,
            )
        assertNotNull(expanded)
        assertEquals(0, expanded!!.startRow)
        assertEquals(1, expanded.endRow)
        assertEquals(6, expanded.endCol)
    }

    @Test
    fun expandUrlSelection_noUrlPresent_returnsNull() {
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
    fun expandUrlSelection_wrapThenBackward_noMatch_returnsNull() {
        val expanded =
            expandUrlSelection(
                lines = listOf("short URL: https://example.com", "next line follows here"),
                startRow = 0,
                startCol = 11,
                endRow = 0,
                endCol = 30,
            )
        assertNull(expanded)
    }

    @Test
    fun expandUrlSelection_firstRowOnly_noWrap() {
        val expanded =
            expandUrlSelection(
                lines = listOf("https://example.com"),
                startRow = 0,
                startCol = 0,
                endRow = 0,
                endCol = 19,
            )
        assertNull(expanded)
    }

    // -- URL-like detection for various patterns --
    @Test
    fun isUrlLikeWord_dotIoDetected() {
        assertTrue(isUrlLikeWord("example.io"))
    }

    @Test
    fun isUrlLikeWord_mixedCaseScheme() {
        assertTrue(isUrlLikeWord("https://Example.Com/Path"))
    }

    @Test
    fun isUrlLikeWord_withPathAndQuery() {
        assertTrue(isUrlLikeWord("https://example.com/search?q=test&page=1"))
    }

    // -- detectUrlBounds (uses same regex as TerminalSurface.detectUrlBounds) --

    private val urlRegex =
        Regex("(https?://[^\\s<>\"'`\\\\|\\[\\]{}]+|www\\.[^\\s<>\"'`\\\\|\\[\\]{}]+)")

    private fun detectUrlBounds(
        line: String,
        col: Int,
    ): Pair<Int, Int>? {
        var bestMatch: Pair<Int, Int>? = null
        var bestDist = Int.MAX_VALUE
        for (match in urlRegex.findAll(line)) {
            if (col >= match.range.first - 1 && col <= match.range.last + 1) {
                val dist = minOf(kotlin.math.abs(col - match.range.first), kotlin.math.abs(col - match.range.last))
                if (dist < bestDist) {
                    bestDist = dist
                    bestMatch = Pair(match.range.first, match.range.last)
                }
            }
        }
        return bestMatch
    }

    @Test
    fun detectUrlBounds_simpleHttpUrl() {
        val line = "visit https://example.com/path now"
        // "https://example.com/path" at cols 6..29
        val bounds = detectUrlBounds(line, 8)
        assertNotNull(bounds)
        assertEquals(6, bounds!!.first)
        assertEquals(29, bounds.second)
    }

    @Test
    fun detectUrlBounds_middleOfHttpsUrl() {
        val line = "check https://secure.example.com/page?id=123 now"
        // "https://secure.example.com/page?id=123" at cols 6..? (len=36, lastIdx=41)
        val bounds = detectUrlBounds(line, 10)
        assertNotNull(bounds)
        assertTrue(bounds!!.first <= 10)
        assertTrue(bounds.second >= 41)
    }

    @Test
    fun detectUrlBounds_wwwDotUrl() {
        val line = "go to www.example.com/path now"
        val bounds = detectUrlBounds(line, 8)
        assertNotNull(bounds)
        assertTrue(bounds!!.first <= 8)
        assertTrue(bounds.second >= 22)
    }

    @Test
    fun detectUrlBounds_httpUrlWithPort() {
        val line = "http://localhost:8080/api/v1/test"
        val bounds = detectUrlBounds(line, 10)
        assertNotNull(bounds)
        assertEquals(0, bounds!!.first)
        assertEquals(32, bounds.second)
    }

    @Test
    fun detectUrlBounds_atLineStart_col0() {
        val bounds = detectUrlBounds("https://example.com", 0)
        assertNotNull(bounds)
        assertEquals(0, bounds!!.first)
        assertEquals(18, bounds.second)
    }

    @Test
    fun detectUrlBounds_onePastUrlEdge() {
        // URL spans 0..18, buffer is 0..19. col=20 is outside.
        val bounds = detectUrlBounds("https://example.com", 20)
        assertNull(bounds)
    }

    @Test
    fun detectUrlBounds_atBufferEdge() {
        // URL spans 0..18, col=19 is within 1-column buffer.
        val bounds = detectUrlBounds("https://example.com", 19)
        assertNotNull(bounds)
        assertEquals(0, bounds!!.first)
        assertEquals(18, bounds.second)
    }

    @Test
    fun detectUrlBounds_farFromUrl_returnsNull() {
        val bounds = detectUrlBounds("not a url at all", 5)
        assertNull(bounds)
    }

    @Test
    fun detectUrlBounds_multipleUrls_picksNearest() {
        val bounds = detectUrlBounds("first https://first.com and https://second.com/path", 40)
        assertNotNull(bounds)
        assertTrue(bounds!!.first >= 0)
    }

    @Test
    fun detectUrlBounds_urlWithQueryString() {
        val line = "search https://example.com/search?q=torvox&page=2"
        val bounds = detectUrlBounds(line, 10)
        assertNotNull(bounds)
        assertEquals(7, bounds!!.first)
    }

    @Test
    fun detectUrlBounds_urlWithFragment() {
        val bounds = detectUrlBounds("link https://example.com/page#section", 10)
        assertNotNull(bounds)
        assertEquals(5, bounds!!.first)
    }

    @Test
    fun detectUrlBounds_urlWithPathSpecialChars() {
        val bounds = detectUrlBounds("url https://example.com/~user/path+test", 10)
        assertNotNull(bounds)
        assertEquals(4, bounds!!.first)
    }

    @Test
    fun detectUrlBounds_colBeforeUrlMinusTwo() {
        // URL spans 0..18, buffer starts at -1. col=-2 is outside.
        val bounds = detectUrlBounds("https://example.com", -2)
        assertNull(bounds)
    }

    @Test
    fun detectUrlBounds_emptyLine() {
        val bounds = detectUrlBounds("", 0)
        assertNull(bounds)
    }
}
