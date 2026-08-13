import type { Component } from 'svelte';

/**
 * 全局视图覆盖层（单层，Android dialog 语义）。
 *
 * - 纯内存状态：不写 URL、不进 history、不监听 popstate。
 * - 浏览器（web）：关闭只能通过下滑拖拽、点击遮罩、Esc。
 * - Tauri Android：系统返回键经 onBackButtonPress → closeAnimated()，
 *   先播关闭动画再清状态（见 lib/back.svelte.ts 的返回分发器）。
 * - 调用方 open() 一次后放手：关闭后自动回到调用方页面，双方互不感知细节。
 */
export interface SheetView {
	key: string;
	component: Component<any>;
	props?: Record<string, unknown>;
	/** 面板高度（任意 CSS 长度，如 '75dvh'、'auto'）；缺省 '75dvh' */
	height?: string;
}

let current = $state<SheetView | null>(null);

// 关闭动画执行器：Drawer（手机端底部卡片）挂载时注册，动画播完由 Drawer 调 close()；
// 桌面端无动画执行器，closeAnimated() 直接清状态。
let closeAnimator: (() => void) | null = null;

// 关闭安全兜底：动画启动后若 Drawer 组件在动画中销毁（animTimer 被 onDestroy 清除、
// closeCard 的 closing 守卫永久短路），drawer 状态也必须最终清空——否则抽屉永远卡死
// （所有关闭路径——拖拽/scrim/返回键——都依赖 closing=false 才放行）。
// 正常关闭（动画播完 drawer.close()）会清掉这个超时，无副作用。
let forceCloseTimer: ReturnType<typeof setTimeout> | undefined;

export const drawer = {
	get view(): SheetView | null {
		return current;
	},
	get isOpen(): boolean {
		return current !== null;
	},
	open(view: SheetView): void {
		clearTimeout(forceCloseTimer);
		forceCloseTimer = undefined;
		current = view;
	},
	/** 播放关闭动画后关闭（Tauri 返回键 / 页面导航时调用） */
	closeAnimated(): void {
		if (!current) return;
		if (closeAnimator) {
			closeAnimator();
			// 安全兜底：动画最长 400ms，800ms 后无论组件生死都强制清状态
			clearTimeout(forceCloseTimer);
			forceCloseTimer = setTimeout(() => {
				forceCloseTimer = undefined;
				current = null;
			}, 800);
		} else {
			current = null;
		}
	},
	/** 直接关闭（拖拽/遮罩关闭的动画播完后由 Drawer 调用） */
	close(): void {
		clearTimeout(forceCloseTimer);
		forceCloseTimer = undefined;
		current = null;
	},
	setCloseAnimator(fn: (() => void) | null): void {
		closeAnimator = fn;
	}
};
