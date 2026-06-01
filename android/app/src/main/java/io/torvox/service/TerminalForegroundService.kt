package io.torvox.service

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.os.IBinder
import android.os.PowerManager
import io.torvox.MainActivity

class TerminalForegroundService : Service() {
    companion object {
        private const val CHANNEL_ID = "torvox_terminal"
        private const val NOTIFICATION_ID = 1
        private const val WAKE_LOCK_TAG = "torvox:terminal_session"

        fun start(context: Context) {
            val intent = Intent(context, TerminalForegroundService::class.java)
            context.startForegroundService(intent)
        }

        fun stop(context: Context) {
            context.stopService(Intent(context, TerminalForegroundService::class.java))
        }
    }

    private var wakeLock: PowerManager.WakeLock? = null
    private var sessionCount: Int = 0

    override fun onCreate() {
        super.onCreate()
        val channel =
            NotificationChannel(
                CHANNEL_ID,
                "Terminal Session",
                NotificationManager.IMPORTANCE_LOW,
            ).apply {
                description = "Persistent notification for active terminal sessions"
                setShowBadge(false)
            }
        val nm = getSystemService(NotificationManager::class.java)
        nm.createNotificationChannel(channel)
    }

    override fun onStartCommand(
        intent: Intent?,
        flags: Int,
        startId: Int,
    ): Int {
        sessionCount = intent?.getIntExtra("session_count", 1) ?: 1
        startForegroundWithSessionCount(sessionCount)
        acquireWakeLockIfNeeded()
        return START_STICKY
    }

    private fun startForegroundWithSessionCount(count: Int) {
        val text =
            if (count <= 1) {
                "Terminal session active"
            } else {
                "$count terminal sessions active"
            }
        val openIntent =
            Intent(this, MainActivity::class.java).apply {
                flags = Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_CLEAR_TOP
            }
        val pending =
            PendingIntent.getActivity(
                this,
                0,
                openIntent,
                PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
            )
        val notification =
            Notification
                .Builder(this, CHANNEL_ID)
                .setContentTitle("Torvox")
                .setContentText(text)
                .setSmallIcon(android.R.drawable.ic_dialog_info)
                .setOngoing(true)
                .setContentIntent(pending)
                .setCategory(Notification.CATEGORY_SERVICE)
                .build()
        startForeground(NOTIFICATION_ID, notification)
    }

    private fun acquireWakeLockIfNeeded() {
        if (wakeLock?.isHeld == true) return
        val pm = getSystemService(Context.POWER_SERVICE) as PowerManager
        wakeLock =
            pm
                .newWakeLock(
                    PowerManager.PARTIAL_WAKE_LOCK,
                    WAKE_LOCK_TAG,
                ).apply {
                    setReferenceCounted(false)
                    acquire(MAX_WAKE_LOCK_DURATION_MS)
                }
    }

    private fun releaseWakeLock() {
        wakeLock?.takeIf { it.isHeld }?.release()
        wakeLock = null
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onDestroy() {
        releaseWakeLock()
        super.onDestroy()
    }

    override fun onTaskRemoved(rootIntent: Intent?) {
        super.onTaskRemoved(rootIntent)
        releaseWakeLock()
        stopSelf()
    }
}

private const val MAX_WAKE_LOCK_DURATION_MS = 30L * 60L * 1000L
