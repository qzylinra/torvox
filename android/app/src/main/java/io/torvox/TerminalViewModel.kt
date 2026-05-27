package io.torvox

import androidx.lifecycle.ViewModel
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import javax.inject.Inject

data class TerminalState(
    val sessionId: Long = 0L,
    val isRunning: Boolean = false,
    val title: String = "Torvox",
)

@HiltViewModel
class TerminalViewModel
    @Inject
    constructor() : ViewModel() {
        private val _state = MutableStateFlow(TerminalState())
        val state: StateFlow<TerminalState> = _state.asStateFlow()
    }
