import { onBackButtonPress } from '@tauri-apps/api/app';
import { isTauri, type PluginListener } from '@tauri-apps/api/core';
import { drawer } from './drawer.svelte';

/**
 * 返回手势分发器（Tauri Android，动态接管）。
 *
 * 优先级固定：抽屉/覆盖层 > 当前页面（挂载时注册的 handler）> 根级退出应用。
 *
 * 与常驻 listener 方案的区别：listener 按需注册——
 * - 抽屉打开 / 页面有可退层级时注册：back 由 JS 处理（关抽屉 / 逐层退），行为完全可控；
 * - 根级（无抽屉、无层级）时注销：back 回到系统 onBackPressed —— predictive back
 *   动画 + 系统退出，不需要任何应用级退出命令。
 *
 * 零 history 记录：纯内存状态，返回导航永远不会干扰路由。
 * web/桌面：不接入任何返回机制——浏览器返回=退出 SPA，抽屉只靠下滑/遮罩/Esc 关闭。
 */
type BackHandler = () => boolean;

const handlers: BackHandler[] = [];

export function registerBackHandler(fn: BackHandler): void {
	if (handlers.includes(fn)) return;
	handlers.push(fn);
	sync();
}

export function unregisterBackHandler(fn: BackHandler): void {
	const i = handlers.indexOf(fn);
	if (i >= 0) handlers.splice(i, 1);
	sync();
}

// 抽屉打开标志（Drawer 组件挂载时设置，卸载时清除）
let drawerNeedsBack = false;

/** 抽屉是否打开（组件挂载时由 Drawer 调用） */
export function setDrawerBackNeeded(v: boolean): void {
	if (drawerNeedsBack === v) return;
	drawerNeedsBack = v;
	sync();
}

/** 分发一次返回手势（仅当 listener 注册时由系统触发） */
function dispatchBack(): void {
	// 抽屉/覆盖层优先：打开时它拥有返回键
	if (drawer.isOpen) {
		drawer.closeAnimated();
		return;
	}
	// 当前页面（挂载时注册）：返回 true 表示已消费
	for (const h of [...handlers].reverse()) {
		if (h()) return;
	}
	// 根级兜底：正常流程下根级无 listener（系统接管退出），这里仅防御性注销
	if (unlisten) {
		void unlisten.unregister();
		unlisten = null;
	}
}

// ---- listener 按需注册（仅 Tauri Android） ----
let unlisten: PluginListener | null = null;
let syncPending = false;

function sync() {
	if (typeof window === 'undefined' || !isTauri() || !/android/i.test(navigator.userAgent)) {
		return;
	}
	const needed = drawerNeedsBack || handlers.length > 0;
	if (needed && !unlisten) {
		if (syncPending) return; // 注册进行中，完成后会复核
		syncPending = true;
		onBackButtonPress(() => dispatchBack())
			.then((u) => {
				syncPending = false;
				unlisten = u;
				// 注册期间需求可能已变化（如抽屉已关）：不再需要则立即注销
				if (!(drawerNeedsBack || handlers.length > 0)) {
					void u.unregister();
					unlisten = null;
				}
			})
			.catch(() => {
				syncPending = false;
			});
	} else if (!needed && unlisten) {
		void unlisten.unregister();
		unlisten = null;
	}
}
