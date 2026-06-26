// @Terminal UI Compose screen, IMPL_ANDR_KT_003, impl, [REQ_ANDR_002]
// @need-ids: REQ_ANDR_002

@file:Suppress("LocalContextGetResourceValueCall")

package io.torvox.ui

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.DrawerValue
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalDrawerSheet
import androidx.compose.material3.ModalNavigationDrawer
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.rememberDrawerState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import io.torvox.R
import io.torvox.TerminalViewModel
import io.torvox.ui.theme.BuiltInThemes
import kotlinx.coroutines.launch
import kotlin.math.max
import kotlin.math.min

@OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)
@Composable
fun TerminalScreen(
    modifier: Modifier = Modifier,
    viewModel: TerminalViewModel = hiltViewModel(),
    onSettings: () -> Unit = {},
    onFileManager: () -> Unit = {},
) {
    val state by viewModel.state.collectAsState()
    val viewModelThemeMode by viewModel.themeMode.collectAsState()
    val viewModelThemeName by viewModel.themeName.collectAsState()
    val viewModelDayThemeName by viewModel.dayThemeName.collectAsState()
    val viewModelNightThemeName by viewModel.nightThemeName.collectAsState()
    val useNerdFontGlyphs by viewModel.useNerdFontGlyphs.collectAsState()
    val runtimeState by viewModel.runtime.state.collectAsState()
    val isSettingsDark =
        when (viewModel.appThemeMode.collectAsState().value) {
            "night" -> true
            "day" -> false
            else -> androidx.compose.foundation.isSystemInDarkTheme()
        }
    val terminalBg =
        when (viewModelThemeMode) {
            "fixed" -> {
                BuiltInThemes.byName(viewModelThemeName).background
            }

            "day" -> {
                BuiltInThemes.byName(viewModelDayThemeName).background
            }

            "night" -> {
                BuiltInThemes.byName(viewModelNightThemeName).background
            }

            "follow_system" -> {
                if (isSettingsDark) {
                    BuiltInThemes.byName(viewModelNightThemeName).background
                } else {
                    BuiltInThemes.byName(viewModelDayThemeName).background
                }
            }

            else -> {
                if (isSettingsDark) {
                    BuiltInThemes.byName(viewModelNightThemeName).background
                } else {
                    BuiltInThemes.byName(viewModelDayThemeName).background
                }
            }
        }
    val drawerState = rememberDrawerState(DrawerValue.Closed)
    val scope = rememberCoroutineScope()
    val context = androidx.compose.ui.platform.LocalContext.current
    val hostView = androidx.compose.ui.platform.LocalView.current
    var showTextSearch by remember { mutableStateOf(false) }

    val lifecycleOwner = androidx.compose.ui.platform.LocalLifecycleOwner.current
    DisposableEffect(lifecycleOwner) {
        val observer =
            LifecycleEventObserver { _, event ->
                if (event == Lifecycle.Event.ON_PAUSE) {
                    scope.launch { viewModel.runtime.saveSession() }
                }
            }
        lifecycleOwner.lifecycle.addObserver(observer)
        onDispose { lifecycleOwner.lifecycle.removeObserver(observer) }
    }

    LaunchedEffect(state.sessions.size) {
        val count = state.sessions.size
        if (count > 0) {
            hostView.announceForAccessibility(
                context.getString(R.string.sessions_accessible, count),
            )
        }
    }

    LaunchedEffect(state.title) {
        val title = state.title
        if (title.isNotEmpty() && title != "Torvox" && context.getString(R.string.terminal) != title) {
            hostView.announceForAccessibility(
                context.getString(R.string.title_changed, title),
            )
        }
    }

    BackHandler(enabled = drawerState.isOpen) {
        scope.launch { drawerState.close() }
    }

    ModalNavigationDrawer(
        drawerState = drawerState,
        drawerContent = {
            ModalDrawerSheet(
                drawerContainerColor = MaterialTheme.colorScheme.surface,
            ) {
                SessionDrawer(
                    viewModel = viewModel,
                    onSettings = {
                        scope.launch { drawerState.close() }
                        onSettings()
                    },
                    onSearch = {
                        showTextSearch = true
                    },
                    onFileManager = {
                        onFileManager()
                    },
                    onClose = {
                        scope.launch { drawerState.close() }
                    },
                )
            }
        },
        modifier = modifier,
    ) {
        val snackbarHostState = remember { SnackbarHostState() }

        Column(
            modifier =
            Modifier
                .fillMaxSize()
                .testTag("TerminalScreen")
                .background(terminalBg)
                .statusBarsPadding()
                .navigationBarsPadding()
                .imePadding(),
        ) {
            val surfaceRef = remember { mutableStateOf<TerminalSurface?>(null) }
            LaunchedEffect(drawerState.isOpen) {
                surfaceRef.value?.drawerOpen = drawerState.isOpen
            }
            val sel = state.selection
            val selectionActive = sel.active && sel.start != null && sel.end != null

            // Text search state
            var searchQuery by remember { mutableStateOf("") }
            var searchResults by remember { mutableStateOf<List<SearchResult>>(emptyList()) }
            var currentResultIndex by remember { mutableStateOf(0) }

            if (showTextSearch) {
                TextSearchBar(
                    query = searchQuery,
                    onQueryChange = { query ->
                        searchQuery = query
                        val text = viewModel.runtime.bridge()?.getTerminalText() ?: ""
                        searchResults = findMatches(text, query)
                        currentResultIndex = 0
                    },
                    resultCount = searchResults.size,
                    currentResultIndex = currentResultIndex,
                    onPrevious = {
                        if (searchResults.isNotEmpty()) {
                            currentResultIndex =
                                if (currentResultIndex > 0) {
                                    currentResultIndex - 1
                                } else {
                                    searchResults.size - 1
                                }
                        }
                    },
                    onNext = {
                        if (searchResults.isNotEmpty()) {
                            currentResultIndex =
                                if (currentResultIndex < searchResults.size - 1) {
                                    currentResultIndex + 1
                                } else {
                                    0
                                }
                        }
                    },
                    onClose = {
                        showTextSearch = false
                        searchQuery = ""
                        searchResults = emptyList()
                    },
                )
            }

            val resolvedTerminalTheme =
                BuiltInThemes.byName(
                    when (viewModelThemeMode) {
                        "fixed" -> viewModelThemeName
                        "day" -> viewModelDayThemeName
                        "night" -> viewModelNightThemeName
                        "follow_system" -> if (isSettingsDark) viewModelNightThemeName else viewModelDayThemeName
                        else -> if (isSettingsDark) viewModelNightThemeName else viewModelDayThemeName
                    },
                )

            Box(
                modifier =
                Modifier
                    .weight(1f)
                    .fillMaxWidth()
                    .testTag("TerminalContent"),
            ) {
                AndroidView(
                    factory = { context ->
                        io.torvox.ui
                            .TerminalSurface(context)
                            .also { surface ->
                                surfaceRef.value = surface
                            }.apply {
                                initialize(viewModel)
                                val cfg = runtimeState
                                setDimensions(cfg.rows, cfg.cols)
                                val bridge = viewModel.runtime.bridge()
                                val scrollbackLimit =
                                    try {
                                        bridge?.scrollbackLength()?.toInt() ?: 50000
                                    } catch (_: Exception) {
                                        50000
                                    }
                                setMaxScrollback(scrollbackLimit)
                                onSwipeLeft = {
                                    viewModel.writeToPty("\u001b".toByteArray())
                                }
                                onSwipeRight = {
                                    viewModel.writeToPty("\t".toByteArray())
                                }
                                onCopyRequested = { text ->
                                    scope.launch {
                                        snackbarHostState.currentSnackbarData?.dismiss()
                                        snackbarHostState.showSnackbar(
                                            message = context.getString(R.string.copied_chars, text.length),
                                            duration = SnackbarDuration.Short,
                                        )
                                    }
                                }
                                onPasteRequested = {
                                    val count = viewModel.pasteFromClipboard()
                                    if (count > 0) {
                                        scope.launch {
                                            snackbarHostState.currentSnackbarData?.dismiss()
                                            snackbarHostState.showSnackbar(
                                                message = context.getString(R.string.pasted_chars, count),
                                                duration = SnackbarDuration.Short,
                                            )
                                        }
                                    }
                                }
                                onZoomChanged = { increase ->
                                    val current = viewModel.fontSize.value
                                    val step = if (increase) 2f else -2f
                                    val newSize = (current + step).coerceIn(8f, 48f)
                                    viewModel.setFontSize(newSize)
                                }
                                post {
                                    requestFocus()
                                }
                            }
                    },
                    update = { surface ->
                        val cfg = runtimeState
                        if (cfg.rows > 0 && cfg.cols > 0) {
                            surface.setDimensions(cfg.rows, cfg.cols)
                        }
                    },
                    modifier = Modifier.fillMaxSize(),
                )

                if (selectionActive && sel.start != null && sel.end != null) {
                    val loRow = min(sel.start!!.row, sel.end!!.row)
                    val hiRow = max(sel.start!!.row, sel.end!!.row)
                    val loCol: Int
                    val hiCol: Int
                    if (sel.start!!.row <= sel.end!!.row) {
                        loCol = sel.start!!.col
                        hiCol = sel.end!!.col
                    } else {
                        loCol = sel.end!!.col
                        hiCol = sel.start!!.col
                    }
                    val themeFg = resolvedTerminalTheme.foreground
                    val themeBg = resolvedTerminalTheme.background
                    val themeAccent = resolvedTerminalTheme.ansi.getOrElse(5) { resolvedTerminalTheme.foreground }

                    fun colorToArgb(c: androidx.compose.ui.graphics.Color): Int = android.graphics.Color.argb(
                        (c.alpha * 255).toInt(),
                        (c.red * 255).toInt(),
                        (c.green * 255).toInt(),
                        (c.blue * 255).toInt(),
                    )
                    val themeFgArgb = colorToArgb(themeFg)
                    val themeBgArgb = colorToArgb(themeBg)
                    val themeAccentArgb = colorToArgb(themeAccent)
                    val toolbarBgArgb =
                        android.graphics.Color.argb(
                            230,
                            android.graphics.Color.red(themeBgArgb),
                            android.graphics.Color.green(themeBgArgb),
                            android.graphics.Color.blue(themeBgArgb),
                        )
                    LaunchedEffect(loRow, loCol, hiRow, hiCol, themeAccentArgb) {
                        surfaceRef.value?.showSelectionHandles(loRow, loCol, hiRow, hiCol, themeAccentArgb)
                        surfaceRef.value?.showSelectionToolbar(
                            loRow,
                            hiRow,
                            loCol,
                            hiCol,
                            onCopy = {
                                viewModel.copySelectionToClipboard()
                                viewModel.clearSelection()
                            },
                            onSelectAll = {
                                viewModel.selectAll()
                            },
                            onOpenUrl = { url ->
                                viewModel.openUrl(url)
                                viewModel.clearSelection()
                            },
                            selectedText = state.selection.selectedText,
                            themeFgColor = themeFgArgb,
                            themeBgColor = toolbarBgArgb,
                            onMoveAnchor = { moveEnd, direction ->
                                viewModel.moveSelectionAnchor(moveEnd, direction)
                            },
                        )
                    }
                } else {
                    LaunchedEffect(selectionActive) {
                        surfaceRef.value?.hideSelectionHandles()
                    }
                }

                val pasteRequest = state.pastePopupRequest
                if (pasteRequest != null) {
                    val requestRow = pasteRequest.row
                    val requestCol = pasteRequest.col
                    LaunchedEffect(requestRow, requestCol) {
                        surfaceRef.value?.showPastePopup(requestRow, requestCol)
                    }
                }
            }

            ModifierBar(
                modifier = Modifier.testTag("ModifierBar"),
                onKeyClick = { data ->
                    viewModel.writeToPty(data.toByteArray())
                },
                onDrawerClick = {
                    scope.launch { drawerState.open() }
                },
                onScrollClick = {
                    viewModel.toggleScrollMode()
                },
                ctrlState = state.ctrlState,
                altState = state.altState,
                onToggleCtrl = {
                    viewModel.cycleCtrlState()
                },
                onToggleAlt = {
                    viewModel.cycleAltState()
                },
                textColor = resolvedTerminalTheme.foreground,
                backgroundColor = resolvedTerminalTheme.background,
                useNerdFontGlyphs = useNerdFontGlyphs,
                toolbarLayout = rememberToolbarLayout(),
            )
        }
    }
}

@Composable
private fun ToolbarButton(
    text: String,
    onClick: () -> Unit,
) {
    TextButton(
        onClick = onClick,
        shape = RoundedCornerShape(4.dp),
        contentPadding =
        androidx.compose.foundation.layout
            .PaddingValues(horizontal = 12.dp, vertical = 6.dp),
    ) {
        Text(
            text = text,
            color = Color(0xFFDDDDDD),
            fontSize = 13.sp,
        )
    }
}
