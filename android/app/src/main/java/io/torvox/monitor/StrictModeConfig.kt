package io.torvox.monitor

import android.os.StrictMode
import io.torvox.BuildConfig

object StrictModeConfig {
    fun install() {
        val threadPolicy =
            StrictMode.ThreadPolicy
                .Builder()
                .detectDiskReads()
                .detectDiskWrites()
                .detectNetwork()
                .detectCustomSlowCalls()
                .apply {
                    if (BuildConfig.DEBUG) {
                        penaltyLog()
                    } else {
                        penaltyLog()
                    }
                }.build()

        val vmPolicy =
            StrictMode.VmPolicy
                .Builder()
                .detectActivityLeaks()
                .detectLeakedClosableObjects()
                .detectLeakedRegistrationObjects()
                .detectFileUriExposure()
                .detectCleartextNetwork()
                .apply {
                    if (BuildConfig.DEBUG) {
                        penaltyLog()
                    } else {
                        penaltyLog()
                    }
                }.build()

        StrictMode.setThreadPolicy(threadPolicy)
        StrictMode.setVmPolicy(vmPolicy)
    }
}
