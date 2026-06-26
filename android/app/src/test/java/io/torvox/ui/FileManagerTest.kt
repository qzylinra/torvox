package io.torvox.ui

import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import java.io.File

class FileManagerTest {
    @get:Rule
    val temporaryFolder = TemporaryFolder()

    @Test
    fun formatFileSizeBytes() {
        assertEquals("0 B", formatFileSize(0))
        assertEquals("512 B", formatFileSize(512))
        assertEquals("1023 B", formatFileSize(1023))
    }

    @Test
    fun formatFileSizeKilobytes() {
        assertEquals("1 KB", formatFileSize(1024))
        assertEquals("10 KB", formatFileSize(10 * 1024))
        assertEquals("1023 KB", formatFileSize(1023 * 1024))
    }

    @Test
    fun formatFileSizeMegabytes() {
        assertEquals("1 MB", formatFileSize(1024L * 1024))
        assertEquals("100 MB", formatFileSize(100L * 1024 * 1024))
    }

    @Test
    fun formatFileSizeGigabytes() {
        assertEquals("1 GB", formatFileSize(1024L * 1024 * 1024))
        assertEquals("10 GB", formatFileSize(10L * 1024 * 1024 * 1024))
    }

    @Test
    fun listDirectoryReturnsFilesAndDirectories() {
        val root = temporaryFolder.root
        File(root, "file.txt").writeText("hello")
        File(root, "subdir").mkdirs()

        val entries = runBlocking { listDirectory(root.absolutePath) }
        assertEquals(2, entries.size)
        val directory = entries.first { it.isDirectory }
        val file = entries.first { !it.isDirectory }
        assertEquals("subdir", directory.name)
        assertEquals("file.txt", file.name)
    }

    @Test
    fun listDirectorySortedDirectoriesFirst() {
        val root = temporaryFolder.root
        File(root, "aaa.txt").writeText("a")
        File(root, "zzz_dir").mkdirs()
        File(root, "bbb.txt").writeText("b")

        val entries = runBlocking { listDirectory(root.absolutePath) }
        assertEquals(3, entries.size)
        assertTrue(entries[0].isDirectory)
        assertTrue(entries[1].name < entries[2].name)
    }

    @Test
    fun listDirectoryEmptyFolder() {
        val root = temporaryFolder.root
        val entries = runBlocking { listDirectory(root.absolutePath) }
        assertTrue(entries.isEmpty())
    }

    @Test
    fun listDirectoryFileHasCorrectSize() {
        val root = temporaryFolder.root
        val file = File(root, "data.bin")
        file.writeBytes(ByteArray(2048) { it.toByte() })

        val entries = runBlocking { listDirectory(root.absolutePath) }
        val entry = entries.first { it.name == "data.bin" }
        assertEquals(2048L, entry.size)
    }

    @Test
    fun readFilePreviewPlainText() {
        val file = temporaryFolder.newFile("readme.txt")
        file.writeText("line1\nline2\nline3")

        val content = runBlocking { readFilePreview(file) }
        assertEquals("line1\nline2\nline3", content)
    }

    @Test
    fun readFilePreviewLargeFileTruncated() {
        val file = temporaryFolder.newFile("large.txt")
        val lines = (1..300).map { "line $it" }
        file.writeText(lines.joinToString("\n"))

        val content = runBlocking { readFilePreview(file) }
        val resultLines = content.split("\n")
        assertEquals(200, resultLines.size)
        assertEquals("line 1", resultLines[0])
        assertEquals("line 200", resultLines[199])
    }

    @Test
    fun readFilePreviewBinaryFileShowsError() {
        val file = temporaryFolder.newFile("binary.bin")
        file.writeBytes(byteArrayOf(0xFF.toByte(), 0xFE.toByte(), 0x00, 0x01))

        val content = runBlocking { readFilePreview(file) }
        assertTrue(content.startsWith("[Cannot read:"))
    }

    @Test
    fun readFilePreviewNonExistentFileShowsError() {
        val file = File(temporaryFolder.root, "does_not_exist.txt")

        val content = runBlocking { readFilePreview(file) }
        assertTrue(content.startsWith("[Cannot read:"))
    }
}
