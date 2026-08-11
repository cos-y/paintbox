<script lang="ts">
	import { fade } from 'svelte/transition';
	import { cubicOut } from 'svelte/easing';
	import { onDestroy, onMount } from 'svelte';
	import { viewStack } from '$lib/viewstack.svelte';
	import { clamp } from '$lib/utils.svelte';

	/** 栈顶视图；多层时卡片内容随栈顶切换（等价色/相近色可逐层 scrim 返回） */
	const top = $derived(viewStack.stack[viewStack.size - 1]);

	function sheetSlide(node: HTMLElement, { duration = 240, easing = cubicOut } = {}) {
		return {
			duration,
			easing,
			css: (t: number) => `transform: translateY(${(1 - t) * 100}%)`
		};
	}

	// ---- 跟手拖拽关闭（rAF 物理动画，保持松手初速度） ----
	const DRAG_START = 8; // px，位移超过才接管拖拽（保住按钮点击/文本选择）
	const CLOSE_THRESHOLD = 0.25; // 位移占面板高度比例，超过触发关闭
	const VELOCITY_CLOSE = 0.6; // px/ms，向下甩动关闭
	const VELOCITY_UP = 0.3; // px/ms，向上甩动回弹（永不关闭）
	const GRAVITY = 0.03; // px/ms²，关闭下滑轻微加速

	let sheetEl = $state<HTMLElement | null>(null);
	let scrimEl = $state<HTMLElement | null>(null);
	// 面板实际高度（dvh 与 innerHeight 在移动端地址栏场景有差异，直接量元素）
	const panelHeight = () => sheetEl?.getBoundingClientRect().height ?? window.innerHeight * 0.75;

	let dragY = $state.raw(0);
	let scrimOpacity = $state.raw(1);

	// 手势/动画账本用 $state.raw：不参与渲染，且组件重渲染不会重置（普通 let 会在重渲染时归零）
	let dragging = $state.raw(false);
	let closing = $state.raw(false);
	let startY = $state.raw(0);
	let panelH = $state.raw(0); // 手势开始时的面板高度（避免动画中反复读布局）
	let samples = $state.raw<{ y: number; t: number }[]>([]);
	let animTimer = $state.raw<ReturnType<typeof setTimeout> | undefined>(undefined); // 关闭/回弹 CSS transition 计时

	onMount(() => {
		// 供 viewStack.clear/onPopstate 调用的关闭动画执行器（导航/后退时先播关闭动画）
		viewStack.setCloseHandler(() => {
			panelH = panelHeight();
			closeCard(0, panelH);
		});
		return () => viewStack.setCloseHandler(null);
	});

	onDestroy(() => {
		if (animTimer) clearTimeout(animTimer);
	});

	function resetSheet() {
		closing = false;
		dragY = 0;
		scrimOpacity = 1;
		applyDrag();
		// 清掉关闭动画残留的 transition（下次打开是全新元素，这里双保险）
		if (sheetEl) sheetEl.style.transition = '';
		if (scrimEl) scrimEl.style.transition = '';
	}

	/** 手动同步面板位移与遮罩透明度（不经 Svelte 样式绑定，避免覆盖内联 transition） */
	function applyDrag() {
		if (sheetEl) sheetEl.style.transform = `translateY(${dragY}px)`;
		if (scrimEl) scrimEl.style.opacity = String(scrimOpacity);
	}

	/** 关闭：整卡下滑出屏（CSS transition 驱动，GPU 合成不掉帧）；初速度决定时长 */
	function closeCard(initialV: number, h: number) {
		if (closing) return;
		closing = true;
		viewStack.setAnimating(true);
		if (animTimer) clearTimeout(animTimer);
		const v = Math.max(initialV, 0.2);
		// 匀加速 p = v·t + ½G·t² 解出到达 h 的时长；速度越快越短
		const t = (-v + Math.sqrt(v * v + 2 * GRAVITY * h)) / GRAVITY;
		const dur = clamp(t, 120, 400);
		if (sheetEl) sheetEl.style.transition = `transform ${dur}ms ease-in`;
		if (scrimEl) scrimEl.style.transition = `opacity ${dur}ms ease-in`;
		dragY = h;
		scrimOpacity = 0;
		applyDrag();
		animTimer = setTimeout(() => {
			animTimer = undefined;
			viewStack.resolveClose(); // 按触发源弹栈/清栈（拖拽/后退/导航）
			resetSheet();
		}, dur);
	}

	/** 回弹到顶：easeOutCubic，时长按初速度换算（同样走 CSS transition） */
	function springBack(v: number, h: number) {
		if (closing) return;
		closing = true;
		viewStack.setAnimating(true);
		if (animTimer) clearTimeout(animTimer);
		const from = dragY;
		const dur = clamp((3 * from) / Math.max(v, 0.05), 80, 350);
		if (sheetEl)
			sheetEl.style.transition = `transform ${dur}ms cubic-bezier(0.215, 0.61, 0.355, 1)`;
		if (scrimEl) scrimEl.style.transition = `opacity ${dur}ms cubic-bezier(0.215, 0.61, 0.355, 1)`;
		dragY = 0;
		scrimOpacity = 1;
		applyDrag();
		animTimer = setTimeout(() => {
			animTimer = undefined;
			closing = false;
			viewStack.setAnimating(false);
			if (sheetEl) sheetEl.style.transition = '';
			if (scrimEl) scrimEl.style.transition = '';
		}, dur);
	}

	function onPointerDown(e: PointerEvent) {
		if (closing) return; // 关闭/回弹动画中禁止交互，动画自行跑完
		startY = e.clientY;
		dragging = false; // 延迟接管：先不锁定，确认是拖拽再接管
		panelH = panelHeight();
		samples = [{ y: e.clientY, t: performance.now() }];
	}

	function onPointerMove(e: PointerEvent) {
		if (closing) return; // 动画中忽略指针移动
		const dy = e.clientY - startY;
		if (!dragging) {
			if (dy <= DRAG_START) return; // 未超过起点：不接管，按钮点击照常触发
			dragging = true;
			(e.currentTarget as HTMLElement | null)?.setPointerCapture?.(e.pointerId);
			e.preventDefault();
		}
		dragY = dy > 0 ? dy : 0; // 只允许向下；往上拖则位置跟随（回弹判定看释放瞬间）
		scrimOpacity = clamp(1 - dragY / panelH, 0, 1);
		applyDrag();
		const now = performance.now();
		// 只在真正移动时采样；停顿后甩动，旧样本会被 80ms 窗口淘汰，速度不掺水
		if (samples.length === 0 || e.clientY !== samples[samples.length - 1].y) {
			samples.push({ y: e.clientY, t: now });
		}
		while (samples.length > 1 && now - samples[0].t > 80) samples.shift();
	}

	function onPointerUp() {
		if (closing) return; // 动画中忽略指针释放
		if (!dragging) return;
		dragging = false;
		// 瞬时速度：最近 80ms 采样窗口的位移/时间
		let v = 0;
		if (samples.length >= 2) {
			const first = samples[0];
			const last = samples[samples.length - 1];
			const dt = last.t - first.t;
			if (dt > 0) v = (last.y - first.y) / dt;
		}
		samples = [];
		// 释放瞬间判定，速度优先：
		// 1) 明显向下甩 → 关闭
		// 2) 明显向上甩 → 回弹（划下去又划回来、初速度向上时永不关闭）
		// 3) 接近静止 → 看释放位置是否越过阈值
		const threshold = panelH * CLOSE_THRESHOLD;
		if (v >= VELOCITY_CLOSE) {
			closeCard(v, panelH);
		} else if (v <= -VELOCITY_UP) {
			springBack(v, panelH);
		} else if (dragY >= threshold) {
			closeCard(v, panelH);
		} else if (dragY > 0) {
			springBack(v, panelH);
		}
	}

	function onPointerCancel() {
		dragging = false;
		dragY = 0;
		scrimOpacity = 1;
		applyDrag();
		samples = [];
	}

	/** 点击遮罩 / Esc 关闭：与拖拽共用同一套重力+初速度动画 */
	function onScrimClick() {
		if (closing) return;
		panelH = panelHeight();
		closeCard(0, panelH);
	}

	// ---- 打开期间：锁定背景滚动与交互；Esc 关闭；焦点移入/归还 ----
	$effect(() => {
		if (viewStack.size === 0) return;
		const prevOverflow = document.body.style.overflow;
		const prevOverscroll = document.body.style.overscrollBehavior;
		document.body.style.overflow = 'hidden';
		document.body.style.overscrollBehavior = 'none';
		const prevFocus = document.activeElement as HTMLElement | null;
		sheetEl?.focus();
		const onKey = (e: KeyboardEvent) => {
			if (e.key === 'Escape') {
				e.preventDefault();
				onScrimClick();
			}
		};
		window.addEventListener('keydown', onKey);
		return () => {
			document.body.style.overflow = prevOverflow;
			document.body.style.overscrollBehavior = prevOverscroll;
			window.removeEventListener('keydown', onKey);
			if (viewStack.size === 0) prevFocus?.focus?.();
		};
	});
</script>

{#if top}
	<!-- 遮罩：单击退回上一层；栈空则关闭卡片 -->
	<button
		bind:this={scrimEl}
		type="button"
		aria-label="close detail"
		class="fixed inset-0 z-60 block cursor-pointer touch-none bg-black/50"
		in:fade={{ duration: 150 }}
		onclick={onScrimClick}
	></button>

	<!-- 底部卡片：盖住底部导航（z 高于 nav 的 z-50），整卡可拖拽 -->
	<div
		bind:this={sheetEl}
		role="dialog"
		aria-modal="true"
		tabindex="-1"
		class="fixed inset-x-0 bottom-0 z-61 flex h-[75dvh] touch-none flex-col rounded-t-2xl bg-white shadow-2xl outline-none dark:bg-gray-900"
		in:sheetSlide={{ duration: 200, easing: cubicOut }}
		onpointerdown={onPointerDown}
		onpointermove={onPointerMove}
		onpointerup={onPointerUp}
		onpointercancel={onPointerCancel}
	>
		<!-- 拖拽把手：纯视觉提示（整卡可拖），浮在内容顶部中央，不占纵向空间 -->
		<div
			class="pointer-events-auto absolute inset-x-0 top-2 z-10 flex cursor-grab touch-none justify-center active:cursor-grabbing"
		>
			<div class="h-1.5 w-12 rounded-full bg-gray-300/90 dark:bg-gray-600/90"></div>
		</div>
		<div class="min-h-0 flex-1 touch-none">
			{#key top.key}
				<div class="h-full" transition:fade={{ duration: 150 }}>
					<svelte:component this={top.component} {...top.props ?? {}} />
				</div>
			{/key}
		</div>
	</div>
{/if}
