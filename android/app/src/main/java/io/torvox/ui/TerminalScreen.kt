package io.torvox.ui

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalLifecycleOwner
import androidx.compose.ui.viewinterop.AndroidView
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import io.torvox.TerminalViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun TerminalScreen(
    modifier: Modifier = Modifier,
    viewModel: TerminalViewModel = hiltViewModel(),
    onSettings: () -> Unit = {},
) {
    val state by viewModel.state.collectAsState()

    val lifecycleOwner = LocalLifecycleOwner.current
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

    Surface(
        modifier = modifier.fillMaxSize(),
        color = MaterialTheme.colorScheme.background,
    ) {
        Column(
            modifier =
                Modifier
                    .fillMaxSize()
                    .windowInsetsPadding(WindowInsets.systemBars),
        ) {
            TopAppBar(
                title = { Text(state.title) },
                actions = {
                    IconButton(onClick = onSettings) {
                        Icon(
                            Icons.Default.Settings,
                            contentDescription = "Settings",
                        )
                    }
                },
                colors =
                    TopAppBarDefaults.topAppBarColors(
                        containerColor = MaterialTheme.colorScheme.surface,
                    ),
            )

            Box(
                modifier =
                    Modifier
                        .weight(1f)
                        .fillMaxWidth(),
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
                        }
                    },
                    modifier = Modifier.fillMaxSize(),
                )
            }

            ModifierBar(
                onKeySend = { data ->
                    viewModel.writeToPty(data.toByteArray())
                },
                keys = state.modifierKeys,
            )
        }
    }
}
