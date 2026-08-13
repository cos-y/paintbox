<script lang="ts">
	import { fade } from 'svelte/transition';
	import { cubicOut } from 'svelte/easing';
	import { onDestroy, onMount } from 'svelte';
	import { drawer } from '$lib/drawer.svelte';
	import { setDrawerBackNeeded } from '$lib/back.svelte';
	import { clamp } from '$lib/utils.svelte';

	/** 当前视图；单层，null 表示关闭 */
	const top = $derived(drawer.view);

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
	// 内容滚动容器：可滚动；滚动到顶后才允许下拉拖关闭
	let scrollEl = $state<HTMLElement | null>(null);
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
		// 供 drawer.closeAnimated 调用的关闭动画执行器（Tauri 返回键/页面导航时先播关闭动画）
		drawer.setCloseAnimator(() => {
			panelH = panelHeight();
			closeCard(0, panelH);
		});
		return () => drawer.setCloseAnimator(null);
	});

	// 抽屉打开时登记 back 需求（Android 根级注销后由系统接管退出）；关闭即清理
	$effect(() => {
		if (!drawer.isOpen) return;
		setDrawerBackNeeded(true);
		return () => setDrawerBackNeeded(false);
	});

	onDestroy(() => {
		if (animTimer) clearTimeout(animTimer);
		// 组件销毁时抽屉可能还开着（布局重建/异常）且动画 timer 已被清：
		// 立即清状态，防止 drawer 卡死（scrim 残留遮挡全屏）
		if (drawer.isOpen) drawer.close();
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
			drawer.close(); // 动画播完清状态（拖拽/遮罩/返回键/导航共用）
			resetSheet();
		}, dur);
	}

	/** 回弹到顶：easeOutCubic，时长按初速度换算（同样走 CSS transition） */
	function springBack(v: number, h: number) {
		if (closing) return;
		closing = true;
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
			if (sheetEl) sheetEl.style.transition = '';
			if (scrimEl) scrimEl.style.transition = '';
		}, dur);
	}

	// ---- 手势：触摸走 touch 事件（浏览器在触摸滚动时会 pointercancel，pointer 事件不可靠），
	// 鼠标/触控笔走 pointer 事件。两条路径共用 dragY/采样账本与 closeCard/springBack。
	let grabStartY = $state.raw(0); // 拖拽接管瞬间的指针 Y（接管前可能有滚动，位移从接管点算起）

	function onPointerDown(e: PointerEvent) {
		if (e.pointerType === 'touch') return; // 触摸由 touch 路径处理
		if (closing) return; // 关闭/回弹动画中禁止交互，动画自行跑完
		startY = e.clientY;
		dragging = false; // 延迟接管：先不锁定，确认是拖拽再接管
		panelH = panelHeight();
		samples = [{ y: e.clientY, t: performance.now() }];
	}

	function onPointerMove(e: PointerEvent) {
		if (e.pointerType === 'touch') return;
		if (closing) return; // 动画中忽略指针移动
		const dy = e.clientY - startY;
		if (!dragging) {
			if (dy <= DRAG_START) return; // 未超过起点：不接管，按钮点击照常触发
			// 内容未滚动到顶时，下拉 = 滚动内容；只有顶部下拉才接管拖拽关闭
			if (scrollEl && scrollEl.scrollTop > 0) return;
			dragging = true;
			grabStartY = e.clientY;
			samples = [];
			(e.currentTarget as HTMLElement | null)?.setPointerCapture?.(e.pointerId);
			e.preventDefault();
		}
		dragY = e.clientY - grabStartY;
		if (dragY < 0) dragY = 0; // 只允许向下
		scrimOpacity = clamp(1 - dragY / panelH, 0, 1);
		applyDrag();
		const now = performance.now();
		// 只在真正移动时采样；停顿后甩动，旧样本会被 80ms 窗口淘汰，速度不掺水
		if (samples.length === 0 || e.clientY !== samples[samples.length - 1].y) {
			samples.push({ y: e.clientY, t: now });
		}
		while (samples.length > 1 && now - samples[0].t > 80) samples.shift();
	}

	function onPointerUp(e: PointerEvent) {
		if (e.pointerType === 'touch') return; // 触摸的释放由 touchend 处理
		finishDrag();
	}

	function onPointerCancel(e: PointerEvent) {
		if (e.pointerType === 'touch') return; // 触摸的取消由 touchcancel 处理
		resetDrag();
	}

	// ---- 触摸路径：内容滚动到顶后下拉 → 阻止原生滚动并接管拖拽关闭 ----
	function onTouchStart(e: TouchEvent) {
		if (closing) return;
		const touch = e.touches[0];
		if (!touch) return;
		startY = touch.clientY;
		dragging = false;
		panelH = panelHeight();
		samples = [{ y: touch.clientY, t: performance.now() }];
	}

	function onTouchMove(e: TouchEvent) {
		if (closing) return; // 动画中忽略触摸移动
		const el = scrollEl;
		const touch = e.touches[0];
		if (!el || !touch) return;
		const dy = touch.clientY - startY;
		if (dy <= 0) return; // 上拉：放行原生滚动
		if (el.scrollTop > 0) return; // 内容未到顶：下拉 = 滚动内容，放行
		if (dy < DRAG_START && !dragging) return; // 微动：未达接管阈值
		e.preventDefault(); // 顶部下拉：阻止原生滚动（下拉刷新的标准机制）
		if (!dragging) {
			dragging = true;
			grabStartY = touch.clientY; // 位移从接管点算起（接管前可能滚过）
			samples = [];
		}
		dragY = touch.clientY - grabStartY;
		if (dragY < 0) dragY = 0;
		scrimOpacity = clamp(1 - dragY / panelH, 0, 1);
		applyDrag();
		const now = performance.now();
		if (samples.length === 0 || touch.clientY !== samples[samples.length - 1].y) {
			samples.push({ y: touch.clientY, t: now });
		}
		while (samples.length > 1 && now - samples[0].t > 80) samples.shift();
	}

	function onTouchEnd() {
		finishDrag();
	}

	function onTouchCancel() {
		resetDrag();
	}

	/** 释放瞬间判定，速度优先（pointerup / touchend 共用） */
	function finishDrag() {
		if (closing) return; // 动画中忽略释放
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

	/** 手势取消：恢复原位（pointercancel / touchcancel 共用） */
	function resetDrag() {
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

	// 顶部下拉 = 拖拽关闭：touchmove 里阻止原生滚动并接管（见 onTouchMove），
	// 不再需要额外的 touch-action 处理——触摸手势完整走 touch 事件。

	// ---- 打开期间：锁定背景滚动与交互；Esc 关闭；焦点移入/归还 ----
	$effect(() => {
		if (!drawer.isOpen) return;
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
			if (!drawer.isOpen) prevFocus?.focus?.();
		};
	});
</script>

{#if top}
	<!-- 遮罩：单击关闭卡片 -->
	<button
		bind:this={scrimEl}
		type="button"
		aria-label="close detail"
		class="fixed inset-0 z-60 block cursor-pointer touch-none bg-black/50"
		in:fade={{ duration: 150 }}
		onclick={onScrimClick}
	></button>

	<!-- 底部卡片：盖住底部导航（z 高于 nav 的 z-50），内容可滚动、到顶后可下拉关闭 -->
	<div
		bind:this={sheetEl}
		role="dialog"
		aria-modal="true"
		tabindex="-1"
		class="fixed inset-x-0 bottom-0 z-61 flex flex-col rounded-t-2xl bg-white shadow-2xl outline-none dark:bg-gray-900"
		style:height={top.height ?? '75dvh'}
		in:sheetSlide={{ duration: 200, easing: cubicOut }}
		onpointerdown={onPointerDown}
		onpointermove={onPointerMove}
		onpointerup={onPointerUp}
		onpointercancel={onPointerCancel}
	>
		<!-- 拖拽把手：纯视觉提示（整卡可拖），浮在内容顶部中央，不拦触摸 -->
		<div class="pointer-events-none absolute inset-x-0 top-2 z-10 flex justify-center">
			<div class="h-1.5 w-12 rounded-full bg-gray-300/90 dark:bg-gray-600/90"></div>
		</div>
		<!-- 内容滚动容器：原生滚动；顶部下拉手势被 touchmove 拦截转拖拽关闭 -->
		<div
			bind:this={scrollEl}
			role="presentation"
			class="min-h-0 flex-1 overflow-y-auto overscroll-contain"
			ontouchstart={onTouchStart}
			ontouchmove={onTouchMove}
			ontouchend={onTouchEnd}
			ontouchcancel={onTouchCancel}
		>
			{#key top.key}
				<div transition:fade={{ duration: 150 }} class="p-4">
					<svelte:component this={top.component} {...top.props ?? {}} />
				</div>
			{/key}
		</div>
	</div>
{/if}
