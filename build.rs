fn main() {
    // The settings window (Tauri) only exists in the Windows build; Linux builds and tests
    // skip it and need no frontend.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        tauri_build::build();
    }
}
