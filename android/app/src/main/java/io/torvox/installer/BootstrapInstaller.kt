package io.torvox.installer

import android.system.Os
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.File
import java.io.FileInputStream
import java.util.zip.ZipInputStream

class BootstrapInstaller {
    companion object {
        const val TERMUX_FILES_DIR = "/data/data/com.termux/files"
        const val PREFIX_DIR = "$TERMUX_FILES_DIR/usr"
        const val STAGING_DIR = "$TERMUX_FILES_DIR/usr-staging"
        const val HOME_DIR = "$TERMUX_FILES_DIR/home"
        val EXEC_PREFIXES = listOf("bin/", "libexec/", "lib/apt/apt-helper", "lib/apt/methods/")

        fun needsInstall(): Boolean = !File("$PREFIX_DIR/bin/bash").exists()

        fun isInstalled(): Boolean = File("$PREFIX_DIR/bin/bash").exists()
    }

    suspend fun install(zipFile: File): Result<Unit> = withContext(Dispatchers.IO) {
        try {
            cleanupOld()
            createDirectories()
            val symlinks = extractZip(zipFile)
            if (symlinks.isEmpty()) {
                return@withContext Result.failure(Exception("No SYMLINKS.txt found in bootstrap ZIP"))
            }
            createSymlinks(symlinks)
            atomicRename()
            ensureHomeAndTmp()
            Result.success(Unit)
        } catch (exception: Exception) {
            Result.failure(exception)
        }
    }

    private fun cleanupOld() {
        delete(File(STAGING_DIR))
        if (File(PREFIX_DIR).exists()) {
            delete(File(PREFIX_DIR))
        }
    }

    private fun createDirectories() {
        File(STAGING_DIR).mkdirs()
        File(PREFIX_DIR).mkdirs()
    }

    private fun extractZip(zipFile: File): List<Pair<String, String>> {
        val symlinks = mutableListOf<Pair<String, String>>()
        FileInputStream(zipFile).use { fis ->
            ZipInputStream(fis).use { zis ->
                processZipEntries(zis, symlinks)
            }
        }
        return symlinks
    }

    private fun processZipEntries(
        zis: ZipInputStream,
        symlinks: MutableList<Pair<String, String>>,
    ) {
        var entry = zis.nextEntry
        while (entry != null) {
            val name = entry.name
            if (name == "SYMLINKS.txt") {
                symlinks.addAll(parseSymlinks(zis.readBytes().decodeToString()))
            } else if (entry.isDirectory) {
                File(STAGING_DIR, name).mkdirs()
            } else {
                val targetFile = File(STAGING_DIR, name)
                targetFile.parentFile?.mkdirs()
                targetFile.outputStream().use { out -> zis.copyTo(out, 8096) }
                if (isExecutable(name)) {
                    Os.chmod(targetFile.absolutePath, 0x1ED) // 0700
                }
            }
            entry = zis.nextEntry
        }
    }

    private fun parseSymlinks(content: String): List<Pair<String, String>> = content.lines().filter { it.isNotBlank() }.mapNotNull { line ->
        val parts = line.split("\u2190")
        if (parts.size == 2) parts[0].trim() to parts[1].trim() else null
    }

    private fun isExecutable(name: String): Boolean = EXEC_PREFIXES.any { name.startsWith(it) } ||
        name.startsWith("lib/apt/methods/")

    private fun createSymlinks(symlinks: List<Pair<String, String>>) {
        for ((target, linkPath) in symlinks) {
            val linkFile = File(STAGING_DIR, linkPath)
            linkFile.parentFile?.mkdirs()
            Os.symlink(target, linkFile.absolutePath)
        }
    }

    private fun atomicRename() {
        val staging = File(STAGING_DIR)
        val prefix = File(PREFIX_DIR)
        if (prefix.exists()) {
            delete(prefix)
        }
        if (!staging.renameTo(prefix)) {
            throw Exception("Atomic rename failed: $STAGING_DIR -> $PREFIX_DIR")
        }
    }

    private fun ensureHomeAndTmp() {
        File(HOME_DIR).mkdirs()
        File("$PREFIX_DIR/tmp").mkdirs()
    }

    private fun delete(file: File) {
        if (file.isDirectory) {
            file.listFiles()?.forEach { delete(it) }
        }
        file.delete()
    }
}
