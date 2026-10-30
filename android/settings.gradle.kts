// The Android wrapper around the Rust game: it hosts GameActivity and packages
// liblithicrivers_client.so (built by `just build-android`) into an APK.
pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}
dependencyResolutionManagement {
    repositories {
        google()
        mavenCentral()
    }
}
rootProject.name = "LithicRivers"
include(":app")
