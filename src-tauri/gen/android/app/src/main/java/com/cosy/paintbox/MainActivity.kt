package com.cosy.paintbox

import android.os.Bundle
import androidx.activity.enableEdgeToEdge

class MainActivity : TauriActivity() {
  // 返回键由 JS 侧 onBackButtonPress 接管（lib/back.svelte.ts）：
  // 关闭 WryActivity 的 goBack 回调，避免它先于 AppPlugin 的监听器感知回调执行
  override val handleBackNavigation: Boolean = false

  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
  }
}
