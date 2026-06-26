plugins {
    id("com.android.library")
    id("androidx.baselineprofile")
}

android {
    namespace = "io.torvox.baselineprofile"
    compileSdk = 36

    defaultConfig {
        minSdk = 33
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}

dependencies {
    implementation(project(":app"))
    implementation("androidx.benchmark:benchmark-macro-junit4:1.5.0-alpha06")
    implementation("androidx.test.ext:junit:1.3.0")
    implementation("androidx.test.espresso:espresso-core:3.7.0")
    implementation("androidx.test.uiautomator:uiautomator:2.3.0")
}

baselineProfile {
    automaticGenerationDuringBuild = false
    saveInSrc = true
}
