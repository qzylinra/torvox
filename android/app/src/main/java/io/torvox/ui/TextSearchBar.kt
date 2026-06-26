package io.torvox.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.KeyboardArrowDown
import androidx.compose.material.icons.filled.KeyboardArrowUp
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import io.torvox.R

data class SearchResult(
    val lineIndex: Int,
    val startIndex: Int,
    val endIndex: Int,
)

fun findMatches(
    text: String,
    query: String,
    matchCase: Boolean = false,
): List<SearchResult> {
    if (query.isEmpty()) return emptyList()
    val lines = text.split("\n")
    val results = mutableListOf<SearchResult>()
    for ((lineIndex, line) in lines.withIndex()) {
        val searchLine = if (matchCase) line else line.lowercase()
        val searchQuery = if (matchCase) query else query.lowercase()
        var startIndex = 0
        while (true) {
            val foundIndex = searchLine.indexOf(searchQuery, startIndex)
            if (foundIndex == -1) break
            results.add(
                SearchResult(
                    lineIndex = lineIndex,
                    startIndex = foundIndex,
                    endIndex = foundIndex + query.length,
                ),
            )
            startIndex = foundIndex + 1
        }
    }
    return results
}

@Composable
fun TextSearchBar(
    query: String,
    onQueryChange: (String) -> Unit,
    resultCount: Int,
    currentResultIndex: Int,
    onPrevious: () -> Unit,
    onNext: () -> Unit,
    onClose: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val focusRequester = remember { FocusRequester() }
    val keyboardController = LocalSoftwareKeyboardController.current

    LaunchedEffect(Unit) {
        focusRequester.requestFocus()
    }

    Row(
        modifier =
        modifier
            .fillMaxWidth()
            .background(MaterialTheme.colorScheme.surfaceContainerHigh)
            .padding(horizontal = 8.dp, vertical = 4.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        OutlinedTextField(
            value = query,
            onValueChange = onQueryChange,
            modifier =
            Modifier
                .weight(1f)
                .focusRequester(focusRequester)
                .testTag("SearchTextField"),
            placeholder = {
                Text(
                    text = stringResource(R.string.search_placeholder),
                    color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.5f),
                    fontSize = 14.sp,
                )
            },
            singleLine = true,
            textStyle = MaterialTheme.typography.bodyMedium.copy(color = MaterialTheme.colorScheme.onSurface),
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Next),
            keyboardActions =
            KeyboardActions(
                onNext = { onNext() },
            ),
            colors =
            OutlinedTextFieldDefaults.colors(
                focusedBorderColor = MaterialTheme.colorScheme.primary,
                unfocusedBorderColor = MaterialTheme.colorScheme.outline,
                cursorColor = MaterialTheme.colorScheme.primary,
            ),
        )

        Spacer(modifier = Modifier.width(4.dp))

        if (query.isNotEmpty()) {
            Text(
                text =
                if (resultCount == 0) {
                    stringResource(R.string.search_no_results)
                } else {
                    stringResource(R.string.search_result_of, currentResultIndex + 1, resultCount)
                },
                fontSize = 12.sp,
                color =
                if (resultCount == 0) {
                    MaterialTheme.colorScheme.error
                } else {
                    MaterialTheme.colorScheme.onSurfaceVariant
                },
                modifier = Modifier.testTag("SearchResultCount"),
            )
        }

        Spacer(modifier = Modifier.width(4.dp))

        IconButton(
            onClick = onPrevious,
            enabled = resultCount > 0,
            modifier = Modifier.size(32.dp).testTag("SearchPrevious"),
        ) {
            Icon(
                imageVector = Icons.Filled.KeyboardArrowUp,
                contentDescription = stringResource(R.string.search_previous),
                tint =
                if (resultCount > 0) {
                    MaterialTheme.colorScheme.onSurface
                } else {
                    MaterialTheme.colorScheme.onSurface.copy(alpha = 0.3f)
                },
                modifier = Modifier.size(20.dp),
            )
        }

        IconButton(
            onClick = onNext,
            enabled = resultCount > 0,
            modifier = Modifier.size(32.dp).testTag("SearchNext"),
        ) {
            Icon(
                imageVector = Icons.Filled.KeyboardArrowDown,
                contentDescription = stringResource(R.string.search_next),
                tint =
                if (resultCount > 0) {
                    MaterialTheme.colorScheme.onSurface
                } else {
                    MaterialTheme.colorScheme.onSurface.copy(alpha = 0.3f)
                },
                modifier = Modifier.size(20.dp),
            )
        }

        IconButton(
            onClick = {
                keyboardController?.hide()
                onClose()
            },
            modifier = Modifier.size(32.dp).testTag("SearchClose"),
        ) {
            Icon(
                imageVector = Icons.Default.Close,
                contentDescription = stringResource(R.string.search_close),
                tint = MaterialTheme.colorScheme.onSurface,
                modifier = Modifier.size(20.dp),
            )
        }
    }
}
