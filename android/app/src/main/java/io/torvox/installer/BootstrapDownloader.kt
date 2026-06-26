package io.torvox.installer

import android.content.Context
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.isActive
import kotlinx.coroutines.withContext
import java.io.File
import java.io.FileOutputStream
import java.net.HttpURLConnection
import java.net.URL

class BootstrapDownloader(
    private val context: Context,
) {
    suspend fun download(
        url: String,
        arch: String,
    ): Result<File> = withContext(Dispatchers.IO) {
        try {
            val cachedDir = File(context.cacheDir, "bootstrap-$arch.zip")
            cachedDir.delete()
            val connection = URL(url).openConnection() as HttpURLConnection
            connection.connectTimeout = 30_000
            connection.readTimeout = 300_000
            connection.instanceFollowRedirects = true
            connection.connect()
            if (connection.responseCode !in 200..299) {
                return@withContext Result.failure(
                    Exception("HTTP ${connection.responseCode}: ${connection.responseMessage}"),
                )
            }
            connection.contentLength.let { length ->
                if (length > 0 && length < 1_048_576) {
                    return@withContext Result.failure(Exception("File too small: $length bytes"))
                }
            }
            connection.inputStream.use { input ->
                FileOutputStream(cachedDir).use { output ->
                    val buffer = ByteArray(8192)
                    var total = 0L
                    while (true) {
                        if (!isActive) {
                            cachedDir.delete()
                            return@withContext Result.failure(Exception("Download cancelled"))
                        }
                        val bytesRead = input.read(buffer)
                        if (bytesRead == -1) break
                        output.write(buffer, 0, bytesRead)
                        total += bytesRead
                    }
                    if (total < 1_048_576) {
                        cachedDir.delete()
                        return@withContext Result.failure(Exception("Download too small: $total bytes"))
                    }
                }
            }
            Result.success(cachedDir)
        } catch (exception: Exception) {
            Result.failure(exception)
        }
    }
}
