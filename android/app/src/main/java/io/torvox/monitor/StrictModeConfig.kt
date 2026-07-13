package io.torvox.monitor

import android.os.StrictMode

object StrictModeConfig {
    fun install() {
        StrictMode.setThreadPolicy(
            StrictMode.ThreadPolicy
                .Builder()
                .detectDiskReads()
                .detectDiskWrites()
                .detectNetwork()
                .detectCustomSlowCalls()
                .detectUnbufferedIo()
                .penaltyLog()
                .build(),
        )
        StrictMode.setVmPolicy(
            StrictMode.VmPolicy
                .Builder()
                .detectActivityLeaks()
                .detectLeakedClosableObjects()
                .detectLeakedRegistrationObjects()
                .detectLeakedSqlLiteObjects()
                .detectFileUriExposure()
                .detectCleartextNetwork()
                .detectContentUriWithoutPermission()
                .penaltyLog()
                .penaltyLog()
                .build(),
        )
    }
}
