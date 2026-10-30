plugins {
    id("com.android.application")
}

android {
    namespace = "com.meltingscales.lithicrivers"
    compileSdk = 36

    defaultConfig {
        applicationId = "com.meltingscales.lithicrivers"
        // wgpu needs Vulkan, which Android has from 7.0; 8.0 (26) is a safe floor
        minSdk = 26
        targetSdk = 36
        versionCode = 1
        versionName = "0.1.0"
        ndk {
            abiFilters += listOf("arm64-v8a", "x86_64")
        }
    }

    // Release signing comes from the environment (CI secrets); without it, release
    // builds fall back to the debug key so they can still be sideloaded
    val keystore = System.getenv("LITHICRIVERS_KEYSTORE")
    signingConfigs {
        if (keystore != null) {
            create("release") {
                storeFile = file(keystore)
                storePassword = System.getenv("LITHICRIVERS_KEYSTORE_PASSWORD")
                keyAlias = System.getenv("LITHICRIVERS_KEY_ALIAS")
                keyPassword = System.getenv("LITHICRIVERS_KEY_PASSWORD")
            }
        }
    }
    buildTypes {
        release {
            isMinifyEnabled = false
            signingConfig = signingConfigs.findByName("release") ?: signingConfigs.getByName("debug")
        }
    }
}

dependencies {
    implementation("androidx.appcompat:appcompat:1.8.0")
    // Must match the C glue in the android-activity crate (0.6.1 wants 4.4.0)
    implementation("androidx.games:games-activity:4.4.0")
}
