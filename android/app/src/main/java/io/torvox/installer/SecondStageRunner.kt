package io.torvox.installer

import android.system.Os
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.File

class SecondStageRunner {
    data class Result(
        val success: Boolean,
        val errors: List<String> = emptyList(),
    )

    suspend fun run(): Result = withContext(Dispatchers.IO) {
        val lockFile = File("${BootstrapInstaller.PREFIX_DIR}/bin/termux-bootstrap-second-stage.sh.lock")
        if (lockFile.exists()) {
            return@withContext Result(true)
        }
        try {
            lockFile.parentFile?.mkdirs()
            Os.symlink(lockFile.absolutePath, lockFile.absolutePath)
        } catch (exception: android.system.ErrnoException) {
            if (exception.errno == android.system.OsConstants.EEXIST) {
                return@withContext Result(true)
            }
            return@withContext Result(false, listOf("Lock file error: ${exception.message}"))
        }
        val dpkgVersion = detectDpkgVersion() ?: "unknown"
        val arch = detectAbi()
        val postinstDir = File("${BootstrapInstaller.PREFIX_DIR}/var/lib/dpkg/info")
        val errors = mutableListOf<String>()
        if (postinstDir.isDirectory) {
            postinstDir
                .listFiles()
                ?.filter { it.name.endsWith(".postinst") }
                ?.forEach { script ->
                    val packageName = script.name.removeSuffix(".postinst")
                    try {
                        Os.chmod(script.absolutePath, 0x1ED)
                        val env =
                            mapOf(
                                "DPKG_MAINTSCRIPT_PACKAGE" to packageName,
                                "DPKG_MAINTSCRIPT_PACKAGE_REFCOUNT" to "1",
                                "DPKG_MAINTSCRIPT_ARCH" to arch,
                                "DPKG_MAINTSCRIPT_NAME" to "postinst",
                                "DPKG_MAINTSCRIPT_DEBUG" to "0",
                                "DPKG_RUNNING_VERSION" to dpkgVersion,
                                "DPKG_FORCE" to "security-mac,downgrade",
                                "DPKG_ADMINDIR" to "${BootstrapInstaller.PREFIX_DIR}/var/lib/dpkg",
                                "DPKG_ROOT" to "",
                                "HOME" to BootstrapInstaller.HOME_DIR,
                                "PATH" to "${BootstrapInstaller.PREFIX_DIR}/bin:/system/bin:/system/xbin",
                                "PREFIX" to BootstrapInstaller.PREFIX_DIR,
                            )
                        val proc =
                            Runtime.getRuntime().exec(
                                arrayOf(script.absolutePath, "configure"),
                                env.map { "${it.key}=${it.value}" }.toTypedArray(),
                                File("/"),
                            )
                        val exitCode = proc.waitFor()
                        if (exitCode != 0) {
                            errors.add("$packageName postinst exited with code $exitCode")
                        }
                    } catch (exception: Exception) {
                        errors.add("$packageName postinst error: ${exception.message}")
                    }
                }
        }
        writeTermuxEnv()
        Result(true, errors)
    }

    private fun detectDpkgVersion(): String? = try {
        val proc = Runtime.getRuntime().exec(arrayOf("${BootstrapInstaller.PREFIX_DIR}/bin/dpkg", "--version"))
        val text = proc.inputStream.bufferedReader().readText()
        val match = Regex("""(\d+\.\d+\.\d+)""").find(text)
        match?.value
    } catch (_: Exception) {
        null
    }

    private fun detectAbi(): String = when (
        android.os.Build.SUPPORTED_ABIS
            .firstOrNull()
    ) {
        "arm64-v8a" -> "aarch64"
        "armeabi-v7a" -> "arm"
        "x86_64" -> "x86_64"
        "x86" -> "i686"
        else -> "aarch64"
    }

    private fun writeTermuxEnv() {
        val envFile = File("${BootstrapInstaller.PREFIX_DIR}/etc/termux/termux.env")
        envFile.parentFile?.mkdirs()
        envFile.writeText(
            """
HOME=${BootstrapInstaller.HOME_DIR}
PREFIX=${BootstrapInstaller.PREFIX_DIR}
PATH=${BootstrapInstaller.PREFIX_DIR}/bin:/system/bin:/system/xbin
TMPDIR=${BootstrapInstaller.PREFIX_DIR}/tmp
SHELL=${BootstrapInstaller.PREFIX_DIR}/bin/bash
LANG=en_US.UTF-8
TERM=xterm-256color
COLORTERM=truecolor
            """.trimIndent(),
        )
    }
}
