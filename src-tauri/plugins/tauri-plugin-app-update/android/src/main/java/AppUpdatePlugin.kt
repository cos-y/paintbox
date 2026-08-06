package app.tauri.appupdate

import android.app.Activity
import android.content.IntentSender
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import com.google.android.play.core.appupdate.AppUpdateManager
import com.google.android.play.core.appupdate.AppUpdateManagerFactory
import com.google.android.play.core.install.model.AppUpdateType
import com.google.android.play.core.install.model.UpdateAvailability

/**
 * Google Play 应用内更新：检测商店是否有新版本（按 versionCode 对比），
 * 并可发起官方更新流程（FLEXIBLE：后台下载 + 系统提示安装）。
 * JS: invoke('plugin:app-update|check') -> { available, inProgress, versionCode }
 *     invoke('plugin:app-update|start_update')
 */
@TauriPlugin
class AppUpdatePlugin(private val activity: Activity) : Plugin(activity) {
    private val manager: AppUpdateManager = AppUpdateManagerFactory.create(activity)

    @Command
    fun check(invoke: Invoke) {
        manager.appUpdateInfo
            .addOnSuccessListener { info ->
                val obj = JSObject()
                obj.put(
                    "available",
                    info.updateAvailability() == UpdateAvailability.UPDATE_AVAILABLE
                )
                obj.put(
                    "inProgress",
                    info.updateAvailability() ==
                        UpdateAvailability.DEVELOPER_TRIGGERED_UPDATE_IN_PROGRESS
                )
                obj.put("versionCode", info.availableVersionCode().toLong())
                invoke.resolve(obj)
            }
            .addOnFailureListener { e -> invoke.reject(e.message) }
    }

    @Command
    fun startUpdate(invoke: Invoke) {
        manager.appUpdateInfo
            .addOnSuccessListener { info ->
                if (info.updateAvailability() == UpdateAvailability.UPDATE_AVAILABLE) {
                    try {
                        manager.startUpdateFlowForResult(
                            info,
                            AppUpdateType.FLEXIBLE,
                            activity,
                            REQUEST_CODE
                        )
                        invoke.resolve()
                    } catch (e: IntentSender.SendIntentException) {
                        invoke.reject(e.message)
                    }
                } else {
                    invoke.reject("no update available")
                }
            }
            .addOnFailureListener { e -> invoke.reject(e.message) }
    }

    companion object {
        private const val REQUEST_CODE = 4321
    }
}
