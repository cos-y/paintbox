import type { Component } from 'svelte';

/**
 * 全局视图栈：Android activity 式的段内导航。
 * push 压栈（slide-in 全屏视图，不销毁底层视图），pop 弹栈；
 * 每次 push 追加一条 history entry（state 标记），系统返回键/手势
 * 通过 popstate 触发 pop，栈空时返回行为交给浏览器。
 * 段（底部导航）切换时由 layout 清空栈。
 */
export interface StackView {
	key: string;
	component: Component<any>;
	props?: Record<string, unknown>;
}

const stack = $state<StackView[]>([]);

// 关闭动画协作：导航（查看全部/调配/切段）时 sheet 先播关闭动画再清栈，
// 使“跳转与关闭”同时发生。ViewSheet 挂载时注册动画执行器（closeHandler），
// 动画播完弹栈后由 ViewSheet 调 resolveClose 统一清空剩余栈。
let closeHandler: (() => void) | null = null;
let closeAllPending = false;

export const viewStack = {
	get stack(): StackView[] {
		return stack;
	},
	get size(): number {
		return stack.length;
	},
	push(view: StackView): void {
		stack.push(view);
		history.pushState({ paintboxView: view.key }, '');
	},
	/** 弹栈；栈空返回 false（交给浏览器默认返回行为） */
	pop(): boolean {
		if (stack.length === 0) return false;
		stack.pop();
		return true;
	},
	/**
	 * 导航时的清栈：有 ViewSheet 动画执行器则先播关闭动画（页面切换与之并行），
	 * 动画播完由 resolveClose 清空栈；无执行器（桌面 ViewOverlay）直接清。
	 */
	clear(): void {
		if (stack.length === 0) return;
		if (closeHandler) {
			closeAllPending = true;
			closeHandler();
		} else {
			stack.length = 0;
		}
	},
	/** ViewSheet 挂载时注册关闭动画执行器，卸载时注销 */
	setCloseHandler(fn: (() => void) | null): void {
		closeHandler = fn;
	},
	/** 关闭动画播放完毕（ViewSheet 弹栈后调用）：执行清栈（导航场景） */
	resolveClose(): void {
		if (closeAllPending) {
			closeAllPending = false;
			stack.length = 0;
		}
	}
};

// 系统返回键 / 手势 → popstate → 弹栈
if (typeof window !== 'undefined') {
	window.addEventListener('popstate', () => {
		viewStack.pop();
	});
}
