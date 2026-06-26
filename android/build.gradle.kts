plugins {
    id("com.android.application") version "9.0.1" apply false
    id("com.android.library") version "9.0.1" apply false
    id("org.jetbrains.kotlin.plugin.compose") version "2.3.21" apply false
    id("com.google.dagger.hilt.android") version "2.59.2" apply false
    id("com.google.devtools.ksp") version "2.3.9" apply false
    id("io.github.takahirom.roborazzi") version "1.59.0" apply false
    id("com.diffplug.spotless") version "7.0.2" apply false
    id("io.gitlab.arturbosch.detekt") version "1.23.8" apply false
    id("androidx.benchmark") version "1.5.0-alpha06" apply false
    id("androidx.baselineprofile") version "1.5.0-alpha06" apply false
}

subprojects {
    apply(plugin = "com.diffplug.spotless")
    configure<com.diffplug.gradle.spotless.SpotlessExtension> {
        kotlin {
            ktlint("1.8.0")
                .editorConfigOverride(
                    mapOf(
                        "ktlint_function_naming_ignore_when_annotated_with" to "Composable",
                    ),
                )
            target("src/**/*.kt")
            targetExclude("**/build/**")
        }
        kotlinGradle {
            ktlint()
            target("*.gradle.kts")
        }
    }
}
