package io.torvox

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.hilt.navigation.compose.hiltViewModel
import dagger.hilt.android.AndroidEntryPoint
import io.torvox.ui.SettingsScreen
import io.torvox.ui.TerminalScreen

@AndroidEntryPoint
class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            TorvoxNavHost()
        }
    }
}

@Composable
private fun TorvoxNavHost() {
    val viewModel: TerminalViewModel = hiltViewModel()
    var showSettings by remember { mutableStateOf(false) }

    if (showSettings) {
        SettingsScreen(
            viewModel = viewModel,
            onBack = { showSettings = false },
        )
    } else {
        TerminalScreen(
            viewModel = viewModel,
            onSettings = { showSettings = true },
        )
    }
}
