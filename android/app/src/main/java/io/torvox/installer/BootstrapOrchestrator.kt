package io.torvox.installer

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

class BootstrapOrchestrator(
    private val downloader: BootstrapDownloader,
    private val installer: BootstrapInstaller,
    private val secondStageRunner: SecondStageRunner,
) {
    enum class Status {
        NOT_INSTALLED,
        INSTALLED,
        INSTALLING,
        ERROR,
    }

    fun getInstallStatus(): Status = if (BootstrapInstaller.isInstalled()) {
        Status.INSTALLED
    } else {
        Status.NOT_INSTALLED
    }

    suspend fun ensureBootstrap(bootstrapUrl: String): Result<String> = withContext(Dispatchers.IO) {
        if (bootstrapUrl.isBlank()) {
            return@withContext Result.success("Bootstrap disabled (URL empty)")
        }
        if (BootstrapInstaller.isInstalled()) {
            return@withContext Result.success("Bootstrap already installed")
        }
        try {
            val arch = detectAbi()
            val zipFile =
                downloader.download(bootstrapUrl, arch).getOrElse { e ->
                    return@withContext Result.failure(Exception("Download failed: ${e.message}"))
                }
            installer.install(zipFile).getOrElse { e ->
                return@withContext Result.failure(Exception("Install failed: ${e.message}"))
            }
            val secondStageResult = secondStageRunner.run()
            val messages = mutableListOf("Bootstrap installed successfully")
            if (secondStageResult.errors.isNotEmpty()) {
                messages.add("${secondStageResult.errors.size} postinst scripts had errors")
            }
            zipFile.delete()
            Result.success(messages.joinToString("; "))
        } catch (exception: Exception) {
            Result.failure(Exception("Bootstrap orchestration failed: ${exception.message}"))
        }
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
}
