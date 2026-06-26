@file:Suppress("MatchingDeclarationName")

package io.torvox.ui

import android.os.Environment
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.FileCopy
import androidx.compose.material.icons.filled.Folder
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.File
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

data class FileManagerEntry(
    val name: String,
    val isDirectory: Boolean,
    val size: Long,
    val lastModified: Long,
)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun FileManagerScreen(
    initialPath: String = Environment.getExternalStorageDirectory().absolutePath,
    onFileSelected: (File) -> Unit = {},
    onClose: () -> Unit = {},
) {
    var currentPath by remember { mutableStateOf(initialPath) }
    var entries by remember { mutableStateOf<List<FileManagerEntry>>(emptyList()) }
    var showPreview by remember { mutableStateOf<File?>(null) }
    var previewContent by remember { mutableStateOf("") }
    val scope = rememberCoroutineScope()

    fun loadDirectory(path: String) {
        scope.launch {
            entries = listDirectory(path)
            currentPath = path
        }
    }

    fun loadPreview(file: File) {
        scope.launch {
            showPreview = file
            previewContent = readFilePreview(file)
        }
    }

    LaunchedEffect(currentPath) {
        loadDirectory(currentPath)
    }

    Scaffold(
        topBar = {
            FileManagerTopBar(
                currentPath = currentPath,
                onBack = {
                    val parent = File(currentPath).parentFile
                    if (parent != null) {
                        loadDirectory(parent.absolutePath)
                    } else {
                        onClose()
                    }
                },
                onClose = onClose,
            )
        },
    ) { padding ->
        FileManagerBody(
            padding = padding,
            showPreview = showPreview,
            previewContent = previewContent,
            entries = entries,
            onDismissPreview = {
                showPreview = null
                previewContent = ""
            },
            onEntryClick = { entry ->
                val child = File(currentPath, entry.name)
                if (entry.isDirectory) {
                    loadDirectory(child.absolutePath)
                } else {
                    loadPreview(child)
                    onFileSelected(child)
                }
            },
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun FileManagerTopBar(
    currentPath: String,
    onBack: () -> Unit,
    onClose: () -> Unit,
) {
    TopAppBar(
        title = {
            Column {
                Text("File Manager", style = MaterialTheme.typography.titleMedium)
                Text(
                    currentPath,
                    style = MaterialTheme.typography.bodySmall,
                    fontFamily = FontFamily.Monospace,
                    maxLines = 1,
                )
            }
        },
        navigationIcon = {
            IconButton(onClick = onBack) {
                Icon(Icons.AutoMirrored.Filled.ArrowBack, "Back")
            }
        },
        actions = {
            IconButton(onClick = onClose) {
                Icon(Icons.Default.Close, "Close")
            }
        },
    )
}

@Suppress("LongParameterList")
@Composable
private fun FileManagerBody(
    padding: androidx.compose.foundation.layout.PaddingValues,
    showPreview: File?,
    previewContent: String,
    entries: List<FileManagerEntry>,
    onDismissPreview: () -> Unit,
    onEntryClick: (FileManagerEntry) -> Unit,
) {
    Column(
        modifier =
        Modifier
            .fillMaxSize()
            .padding(padding),
    ) {
        if (showPreview != null) {
            FilePreview(
                fileName = showPreview.name,
                content = previewContent,
                onDismiss = onDismissPreview,
            )
        } else {
            FileManagerList(
                entries = entries,
                onEntryClick = onEntryClick,
            )
        }
    }
}

@Composable
private fun FilePreview(
    fileName: String,
    content: String,
    onDismiss: () -> Unit,
) {
    Column(modifier = Modifier.padding(8.dp)) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            modifier = Modifier.fillMaxWidth(),
        ) {
            Text(fileName, style = MaterialTheme.typography.titleSmall, modifier = Modifier.weight(1f))
            IconButton(onClick = onDismiss) {
                Icon(Icons.Default.Close, "Close preview")
            }
        }
        Text(
            content,
            style = MaterialTheme.typography.bodySmall,
            fontFamily = FontFamily.Monospace,
            modifier =
            Modifier
                .fillMaxWidth()
                .weight(1f)
                .padding(4.dp),
        )
    }
}

@Composable
private fun FileManagerList(
    entries: List<FileManagerEntry>,
    onEntryClick: (FileManagerEntry) -> Unit,
) {
    LazyColumn(modifier = Modifier.fillMaxSize()) {
        items(entries) { entry ->
            FileManagerEntryRow(entry = entry, onClick = { onEntryClick(entry) })
        }
        if (entries.isEmpty()) {
            item {
                Text("Empty directory", modifier = Modifier.padding(16.dp), style = MaterialTheme.typography.bodyMedium)
            }
        }
    }
}

@Composable
private fun FileManagerEntryRow(
    entry: FileManagerEntry,
    onClick: () -> Unit,
) {
    Row(
        modifier =
        Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick)
            .padding(horizontal = 16.dp, vertical = 8.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Icon(
            if (entry.isDirectory) Icons.Default.Folder else Icons.Default.FileCopy,
            contentDescription = null,
            modifier = Modifier.size(24.dp),
            tint =
            if (entry.isDirectory) {
                MaterialTheme.colorScheme.primary
            } else {
                MaterialTheme.colorScheme.onSurfaceVariant
            },
        )
        Spacer(modifier = Modifier.width(12.dp))
        Column(modifier = Modifier.weight(1f)) {
            Text(
                entry.name,
                style = MaterialTheme.typography.bodyMedium,
                maxLines = 1,
            )
            val dateStr =
                remember(entry.lastModified) {
                    SimpleDateFormat("yyyy-MM-dd HH:mm", java.util.Locale.US).format(Date(entry.lastModified))
                }
            val sizeStr = if (entry.isDirectory) "dir" else formatFileSize(entry.size)
            Text(
                "$dateStr  $sizeStr",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
}

internal suspend fun listDirectory(path: String): List<FileManagerEntry> = withContext(Dispatchers.IO) {
    File(path)
        .listFiles()
        ?.map { f ->
            FileManagerEntry(
                name = f.name,
                isDirectory = f.isDirectory,
                size = if (f.isFile) f.length() else 0,
                lastModified = f.lastModified(),
            )
        }?.sortedWith(compareByDescending<FileManagerEntry> { it.isDirectory }.thenBy { it.name })
        ?: emptyList()
}

internal suspend fun readFilePreview(file: File): String = withContext(Dispatchers.IO) {
    try {
        val bytes = file.readBytes()
        if (bytes.isNotEmpty() && bytes.contains(0x00.toByte())) {
            return@withContext "[Cannot read: binary file]"
        }
        bytes
            .toString(Charsets.UTF_8)
            .lines()
            .take(200)
            .joinToString("\n")
    } catch (exception: Exception) {
        "[Cannot read: ${exception.message}]"
    }
}

internal fun formatFileSize(bytes: Long): String = when {
    bytes < 1024 -> "$bytes B"
    bytes < 1024 * 1024 -> "${bytes / 1024} KB"
    bytes < 1024 * 1024 * 1024 -> "${bytes / (1024 * 1024)} MB"
    else -> "${bytes / (1024 * 1024 * 1024)} GB"
}
