# JNA (boltffi runtime dependency)
-keep class com.sun.jna.** { *; }
-dontwarn java.awt.**
-dontwarn sun.awt.**
# boltffi bridge types
-keep class io.torvox.bridge.** { *; }

# Dagger Hilt
-keep class dagger.hilt.** { *; }
-keep class javax.inject.** { *; }

# Lifecycle (ViewModel with Hilt)
-keep class androidx.lifecycle.** { *; }

# Kotlin Metadata (required for Hilt to reflect upon injected ViewModels)
-keep class kotlin.Metadata { *; }
