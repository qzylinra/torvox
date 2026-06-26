package io.torvox.ui

import java.net.URLDecoder
import java.util.regex.Pattern

object UrlDetector {
    private val URL_PATTERN: Pattern =
        Pattern.compile(
            "(https?://[\\w\\-]+(\\.[\\w\\-]+)+([\\w\\-.,@?^=%&:/~+#]*[\\w\\-@^=%&/~+#])?)",
        )

    fun findUrls(text: String): List<String> {
        val matcher = URL_PATTERN.matcher(text)
        val urls = mutableListOf<String>()
        while (matcher.find()) {
            var url = matcher.group(1) ?: continue
            url = trimTrailingPunctuation(url)
            url = decodeIfNeeded(url)
            if (url.isNotBlank()) {
                urls.add(url)
            }
        }
        return urls
    }

    private fun trimTrailingPunctuation(url: String): String {
        var end = url.length
        while (end > 0) {
            val ch = url[end - 1]
            if (ch !in ".,;:!") break
            end--
        }
        return url.substring(0, end)
    }

    private fun decodeIfNeeded(url: String): String {
        if (!url.contains('%')) return url
        return try {
            URLDecoder.decode(url, "UTF-8")
        } catch (_: Exception) {
            url
        }
    }
}
