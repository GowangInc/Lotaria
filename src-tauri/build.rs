fn main() {
    #[cfg(target_os = "windows")]
    println!("cargo:rustc-link-lib=dwmapi");

    tauri_build::build()
}
