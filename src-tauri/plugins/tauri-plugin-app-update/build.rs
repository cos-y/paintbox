fn main() {
    tauri_plugin::Builder::new(&["check", "start_update"])
        .android_path("android")
        .build();
}
