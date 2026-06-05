package io.torvox.ui

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.systemBarsPadding
import androidx.compose.material3.DrawerValue
import androidx.compose.material3.ModalDrawerSheet
import androidx.compose.material3.ModalNavigationDrawer
import androidx.compose.material3.Surface
import androidx.compose.material3.rememberDrawerState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.viewinterop.AndroidView
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import io.torvox.TerminalViewModel
import kotlinx.coroutines.launch

@OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)
@Composable
fun TerminalScreen(
    modifier: Modifier = Modifier,
    viewModel: TerminalViewModel = hiltViewModel(),
    onSettings: () -> Unit = {},
) {
    val state by viewModel.state.collectAsState()
    val drawerState = rememberDrawerState(DrawerValue.Closed)
    val scope = rememberCoroutineScope()

    val lifecycleOwner = androidx.compose.ui.platform.LocalLifecycleOwner.current
    DisposableEffect(lifecycleOwner) {
        val observer =
            LifecycleEventObserver { _, event ->
                if (event == Lifecycle.Event.ON_PAUSE) {
                    viewModel.runtime.saveSession()
                }
            }
        lifecycleOwner.lifecycle.addObserver(observer)
        onDispose { lifecycleOwner.lifecycle.removeObserver(observer) }
    }

    ModalNavigationDrawer(
        drawerState = drawerState,
        drawerContent = {
            ModalDrawerSheet(
                drawerContainerColor = Color(0xFF1A1A1A),
            ) {
                SessionDrawer(
                    viewModel = viewModel,
                    onSettings = {
                        scope.launch { drawerState.close() }
                        onSettings()
                    },
                    onClose = {
                        scope.launch { drawerState.close() }
                    },
                )
            }
        },
        modifier = modifier,
    ) {
        Surface(
            modifier = Modifier.fillMaxSize().testTag("TerminalScreen"),
            color = Color(0xFF1E1E2E),
        ) {
            Column(modifier = Modifier.fillMaxSize().systemBarsPadding()) {
                Box(
                    modifier =
                        Modifier
                            .weight(1f)
                            .fillMaxWidth()
                            .testTag("TerminalContent"),
                ) {
                    AndroidView(
                        factory = { context ->
                            io.torvox.ui.TerminalSurface(context).apply {
                                initialize(viewModel)
                                val cfg = viewModel.runtime.state.value
                                setDimensions(cfg.rows, cfg.cols)
                                val bridge = viewModel.runtime.bridge()
                                val scrollbackLimit =
                                    try {
                                        bridge?.scrollbackLen()?.toInt() ?: 50000
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
                                post {
                                    requestFocus()
                                    val imm =
                                        context.getSystemService(
                                            android.content.Context.INPUT_METHOD_SERVICE,
                                        ) as android.view.inputmethod.InputMethodManager
                                    imm.showSoftInput(this, android.view.inputmethod.InputMethodManager.SHOW_IMPLICIT)
                                }
                            }
                        },
                        modifier = Modifier.fillMaxSize(),
                    )
                }

                ModifierBar(
                    modifier = Modifier.testTag("ModifierBar"),
                    onKeySend = { data ->
                        viewModel.writeToPty(data.toByteArray())
                    },
                    onToggleChanged = { label, active ->
                        when (label) {
                            "CTRL" -> viewModel.setCtrlActive(active)
                            "ALT" -> viewModel.setAltActive(active)
                        }
                    },
                    onSessionDrawer = {
                        scope.launch { drawerState.open() }
                    },
                )
            }
        }
    }
}
