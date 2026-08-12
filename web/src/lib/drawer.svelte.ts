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
}

let current = $state<SheetView | null>(null);

// 关闭动画执行器：ViewSheet（手机端底部卡片）挂载时注册，动画播完由 ViewSheet 调 close()；
// 桌面端 ViewOverlay 无动画执行器，closeAnimated() 直接清状态。
let closeAnimator: (() => void) | null = null;

export const drawer = {
	get view(): SheetView | null {
		return current;
	},
	get isOpen(): boolean {
		return current !== null;
	},
	open(view: SheetView): void {
		current = view;
	},
	/** 播放关闭动画后关闭（Tauri 返回键 / 页面导航时调用） */
	closeAnimated(): void {
		if (!current) return;
		if (closeAnimator) closeAnimator();
		else current = null;
	},
	/** 直接关闭（拖拽/遮罩关闭的动画播完后由 ViewSheet 调用） */
	close(): void {
		current = null;
	},
	setCloseAnimator(fn: (() => void) | null): void {
		closeAnimator = fn;
	}
};
