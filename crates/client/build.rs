//! With the `steam` feature, give the game binary an rpath to its own folder, so it finds
//! the Steam API library shipped beside it without `LD_LIBRARY_PATH`. Without the feature
//! (every development build) this does nothing.

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    if std::env::var_os("CARGO_FEATURE_STEAM").is_none() {
        return;
    }
    match std::env::var("CARGO_CFG_TARGET_OS").as_deref() {
        Ok("linux") => {
            println!("cargo::rustc-link-arg-bin=lithicrivers-client=-Wl,-rpath,$ORIGIN");
        }
        Ok("macos") => {
            println!("cargo::rustc-link-arg-bin=lithicrivers-client=-Wl,-rpath,@executable_path");
        }
        _ => {}
    }
}
