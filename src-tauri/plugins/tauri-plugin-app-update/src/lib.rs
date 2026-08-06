use tauri::{plugin::{Builder, TauriPlugin}, Runtime};

/// Android 侧 Kotlin 插件类所在包名（与 AppUpdatePlugin.kt 的 package 声明一致）
#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "app.tauri.appupdate";

/// 应用内更新检测（Google Play In-app updates）。
/// Android 侧由 AppUpdatePlugin (Kotlin) 实现；JS 通过
/// invoke('plugin:app-update|check') / invoke('plugin:app-update|start_update') 调用。
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("app-update")
        .setup(|_, api| {
            // 将 Kotlin 插件实例化并注册进 Android 的 PluginManager，
            // 否则前端 invoke('plugin:app-update|...') 会报 "Plugin app-update not initialized"
            #[cfg(target_os = "android")]
            api.register_android_plugin(PLUGIN_IDENTIFIER, "AppUpdatePlugin")?;
            Ok(())
        })
        .build()
}
