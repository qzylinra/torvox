package io.torvox.ui.theme

import androidx.compose.ui.graphics.Color

data class TerminalTheme(
    val name: String,
    val background: Color,
    val foreground: Color,
    val cursor: Color,
    val ansi: List<Color>,
) {
    init {
        require(ansi.size == 16) { "Theme must have exactly 16 ANSI colors" }
    }
}

object BuiltInThemes {
    val catppuccinMocha =
        TerminalTheme(
            name = "Catppuccin Mocha",
            background = Color(0xFF1E1E2E),
            foreground = Color(0xFFCDD6F4),
            cursor = Color(0xFFF5E0DC),
            ansi =
                listOf(
                    Color(0xFF181825),
                    Color(0xFFF38BA8),
                    Color(0xFFA6E3A1),
                    Color(0xFFF9E2AF),
                    Color(0xFF89B4FA),
                    Color(0xFFCBA6F7),
                    Color(0xFF94E2D5),
                    Color(0xFFCDD6F4),
                    Color(0xFF6C7086),
                    Color(0xFFF38BA8),
                    Color(0xFFA6E3A1),
                    Color(0xFFF9E2AF),
                    Color(0xFF89B4FA),
                    Color(0xFFCBA6F7),
                    Color(0xFF94E2D5),
                    Color(0xFFBAC2DE),
                ),
        )

    val dracula =
        TerminalTheme(
            name = "Dracula",
            background = Color(0xFF282A36),
            foreground = Color(0xFFF8F8F2),
            cursor = Color(0xFFF8F8F2),
            ansi =
                listOf(
                    Color(0xFF000000),
                    Color(0xFFFF5555),
                    Color(0xFF50FA7B),
                    Color(0xFFF1FA8C),
                    Color(0xFFBD93F9),
                    Color(0xFFFF79C6),
                    Color(0xFF8BE9FD),
                    Color(0xFFFFFFFF),
                    Color(0xFF44475A),
                    Color(0xFFFF5555),
                    Color(0xFF50FA7B),
                    Color(0xFFF1FA8C),
                    Color(0xFFBD93F9),
                    Color(0xFFFF79C6),
                    Color(0xFF8BE9FD),
                    Color(0xFFFFFFFF),
                ),
        )

    val solarizedDark =
        TerminalTheme(
            name = "Solarized Dark",
            background = Color(0xFF002B36),
            foreground = Color(0xFF839496),
            cursor = Color(0xFF839496),
            ansi =
                listOf(
                    Color(0xFF073642),
                    Color(0xFFDC322F),
                    Color(0xFF859900),
                    Color(0xFFB58900),
                    Color(0xFF268BD2),
                    Color(0xFFD33682),
                    Color(0xFF2AA198),
                    Color(0xFFEEE8D5),
                    Color(0xFF002B36),
                    Color(0xFFDC322F),
                    Color(0xFF859900),
                    Color(0xFFB58900),
                    Color(0xFF268BD2),
                    Color(0xFFD33682),
                    Color(0xFF2AA198),
                    Color(0xFFEEE8D5),
                ),
        )

    val nord =
        TerminalTheme(
            name = "Nord",
            background = Color(0xFF2E3440),
            foreground = Color(0xFFD8DEE9),
            cursor = Color(0xFFD8DEE9),
            ansi =
                listOf(
                    Color(0xFF3B4252),
                    Color(0xFFBF616A),
                    Color(0xFFA3BE8C),
                    Color(0xFFEBCB8B),
                    Color(0xFF81A1C1),
                    Color(0xFFB48EAD),
                    Color(0xFF88C0D0),
                    Color(0xFFE5E9F0),
                    Color(0xFF4C566A),
                    Color(0xFFBF616A),
                    Color(0xFFA3BE8C),
                    Color(0xFFEBCB8B),
                    Color(0xFF81A1C1),
                    Color(0xFFB48EAD),
                    Color(0xFF88C0D0),
                    Color(0xFFECEFF4),
                ),
        )

    val tokyoNight =
        TerminalTheme(
            name = "Tokyo Night",
            background = Color(0xFF1A1B26),
            foreground = Color(0xFFC0CAF5),
            cursor = Color(0xFFC0CAF5),
            ansi =
                listOf(
                    Color(0xFF181926),
                    Color(0xFFF7768E),
                    Color(0xFF98C379),
                    Color(0xFFE5C07B),
                    Color(0xFF82AAFF),
                    Color(0xFFC792EA),
                    Color(0xFF56C4C6),
                    Color(0xFFC0CAF5),
                    Color(0xFF45475A),
                    Color(0xFFF7768E),
                    Color(0xFF98C379),
                    Color(0xFFE5C07B),
                    Color(0xFF82AAFF),
                    Color(0xFFC792EA),
                    Color(0xFF56C4C6),
                    Color(0xFFC0CAF5),
                ),
        )

    val gruvboxDark =
        TerminalTheme(
            name = "Gruvbox Dark",
            background = Color(0xFF1D2021),
            foreground = Color(0xFFEBDBB2),
            cursor = Color(0xFFEBDBB2),
            ansi =
                listOf(
                    Color(0xFF1D2021),
                    Color(0xFFCC241D),
                    Color(0xFF98971A),
                    Color(0xFFD79921),
                    Color(0xFF458588),
                    Color(0xFFB16286),
                    Color(0xFF689D6A),
                    Color(0xFFEBDBB2),
                    Color(0xFF504945),
                    Color(0xFFFB4934),
                    Color(0xFFB8BB26),
                    Color(0xFFFABD2F),
                    Color(0xFF83A598),
                    Color(0xFFD3869B),
                    Color(0xFF8EC07C),
                    Color(0xFFE5DEB3),
                ),
        )

    val oneDark =
        TerminalTheme(
            name = "One Dark",
            background = Color(0xFF282C34),
            foreground = Color(0xFFABB2BF),
            cursor = Color(0xFFABB2BF),
            ansi =
                listOf(
                    Color(0xFF1F2335),
                    Color(0xFFE06C75),
                    Color(0xFF98C379),
                    Color(0xFFE5C07B),
                    Color(0xFF61AFEF),
                    Color(0xFFC678DD),
                    Color(0xFF56B6C2),
                    Color(0xFFABB2BF),
                    Color(0xFF4C5264),
                    Color(0xFFE06C75),
                    Color(0xFF98C379),
                    Color(0xFFE5C07B),
                    Color(0xFF61AFEF),
                    Color(0xFFC678DD),
                    Color(0xFF56B6C2),
                    Color(0xFFD0D4E0),
                ),
        )

    val monokai =
        TerminalTheme(
            name = "Monokai",
            background = Color(0xFF272822),
            foreground = Color(0xFFF8F8F2),
            cursor = Color(0xFFF8F8F2),
            ansi =
                listOf(
                    Color(0xFF1B1C16),
                    Color(0xFFF92672),
                    Color(0xFFA6E22E),
                    Color(0xFFF4BF75),
                    Color(0xFF66D9EF),
                    Color(0xFFAE81FF),
                    Color(0xFFA6E22E),
                    Color(0xFFF8F8F2),
                    Color(0xFF666666),
                    Color(0xFFF92672),
                    Color(0xFFA6E22E),
                    Color(0xFFF4BF75),
                    Color(0xFF66D9EF),
                    Color(0xFFAE81FF),
                    Color(0xFFA6E22E),
                    Color(0xFFF8F8F2),
                ),
        )

    val githubDark =
        TerminalTheme(
            name = "GitHub Dark",
            background = Color(0xFF161B22),
            foreground = Color(0xFFC9D1D9),
            cursor = Color(0xFFC9D1D9),
            ansi =
                listOf(
                    Color(0xFF1B1F24),
                    Color(0xFFFF7B72),
                    Color(0xFF3FB950),
                    Color(0xFFE5C07B),
                    Color(0xFF58A6FF),
                    Color(0xFFD2A8FF),
                    Color(0xFF67E2ED),
                    Color(0xFFC9D1D9),
                    Color(0xFF6E7681),
                    Color(0xFFFF7B72),
                    Color(0xFF3FB950),
                    Color(0xFFE5C07B),
                    Color(0xFF58A6FF),
                    Color(0xFFD2A8FF),
                    Color(0xFF67E2ED),
                    Color(0xFFE6EDF3),
                ),
        )

    val rosePine =
        TerminalTheme(
            name = "Rosé Pine",
            background = Color(0xFF191724),
            foreground = Color(0xFFE0DEF4),
            cursor = Color(0xFFE0DEF4),
            ansi =
                listOf(
                    Color(0xFF1F1D2E),
                    Color(0xFFEB6F92),
                    Color(0xFF9CCFD8),
                    Color(0xFFF6C177),
                    Color(0xFF7F84CB),
                    Color(0xFFC4A7E7),
                    Color(0xFF9CCFD8),
                    Color(0xFFE0DEF4),
                    Color(0xFF6E6A86),
                    Color(0xFFEB6F92),
                    Color(0xFF9CCFD8),
                    Color(0xFFF6C177),
                    Color(0xFF7F84CB),
                    Color(0xFFC4A7E7),
                    Color(0xFF9CCFD8),
                    Color(0xFFE9E5ED),
                ),
        )

    val all: List<TerminalTheme> =
        listOf(
            catppuccinMocha,
            dracula,
            solarizedDark,
            nord,
            tokyoNight,
            gruvboxDark,
            oneDark,
            monokai,
            githubDark,
            rosePine,
        )

    fun byName(name: String): TerminalTheme = all.firstOrNull { it.name == name } ?: catppuccinMocha
}
