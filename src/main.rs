#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

#[cfg(windows)]
fn main() {
    std::process::exit(no_drama_llama::win::main());
}

#[cfg(not(windows))]
fn main() {
    eprintln!("No Drama Llama runs on Windows. On this OS only the library and its tests build.");
    std::process::exit(1);
}
