<script lang="ts">
	import type { Snippet } from 'svelte';

	/**
	 * PanText：超长单行文本的拖拽预览控件。
	 * - 内容不超长：表现与普通文本一致。
	 * - 内容超长：单行横向滚动容器（滚动条隐藏），两端渐隐指示还有内容；
	 *   滚动到头的一侧渐变收拢为 0（避免"内容被切断"的错觉）；
	 *   鼠标/触控笔按住拖拽平移查看，触屏走原生横向滑动（overflow-x-auto）。
	 * - 拖拽超过阈值后吞掉紧随的 click，避免误触外层卡片/按钮。
	 * 父容器为 flex 时无需额外处理（自带 min-w-0）。
	 */
	interface Props {
		children: Snippet;
		/** 传给根容器的样式（字号/颜色等） */
		class?: string;
	}

	let { children, class: cls = '' }: Props = $props();

	let el = $state<HTMLDivElement | null>(null);
	let dragging = $state(false);
	// 非响应式的拖拽账本（避免高频渲染）
	let startX = 0;
	let startLeft = 0;
	let moved = 0;

	const canScroll = $derived(el ? el.scrollWidth > el.clientWidth + 1 : false);

	// 两端渐隐宽度（滚动到头的一侧归零）。mask-image 渐变本身不可插值，
	// 用 @property 注册的长度变量驱动，CSS transition 平滑过渡。
	const FADE = '1.6em';

	function updateFade() {
		if (!el) return;
		const atStart = el.scrollLeft <= 0;
		const atEnd = el.scrollLeft + el.clientWidth >= el.scrollWidth - 1;
		el.style.setProperty('--fade-l', atStart ? '0px' : FADE);
		el.style.setProperty('--fade-r', atEnd ? '0px' : FADE);
	}

	$effect(() => {
		children; // 文本变化时 scrollWidth 变（元素 box 尺寸不变，ResizeObserver 感知不到），需重算
		if (!el) return;
		updateFade();
		// 容器尺寸变化（PC 端拖拽窗口缩放等）时重新计算两端渐隐
		const ro = new ResizeObserver(() => updateFade());
		ro.observe(el); // observe 后立即回调一次，覆盖初始状态
		return () => ro.disconnect();
	});

	function onPointerDown(e: PointerEvent) {
		if (e.pointerType !== 'mouse') return; // 触屏由原生横向滚动接管（自带惯性）
		if (!el || !canScroll) return;
		if (e.button !== 0) return;
		startX = e.clientX;
		startLeft = el.scrollLeft;
		moved = 0;
		dragging = true;
		el.setPointerCapture(e.pointerId);
		e.preventDefault(); // 阻止拖拽过程中选中文本
	}

	function onPointerMove(e: PointerEvent) {
		if (!dragging || !el) return;
		const dx = e.clientX - startX;
		moved = Math.max(moved, Math.abs(dx));
		el.scrollLeft = startLeft - dx; // 触发 scroll 事件 → updateFade
	}

	function release() {
		if (!dragging) return;
		dragging = false;
		if (moved > 5) {
			// 拖拽过就算滑动（不是点击）：在 document 捕获阶段吞掉紧随的 click，
			// 防止误触外层卡片/按钮；监听器为一次性，click 在 pointerup 后必到。
			const swallow = (e: MouseEvent) => {
				e.preventDefault();
				e.stopPropagation();
				document.removeEventListener('click', swallow, true);
			};
			document.addEventListener('click', swallow, true);
		}
		moved = 0;
	}

	function cancel() {
		dragging = false;
		moved = 0;
	}
</script>

<div class="relative max-w-full min-w-0 whitespace-nowrap {cls}" role="presentation">
	<div
		bind:this={el}
		role="presentation"
		class="fade-edge scrollbar-none overflow-x-auto overscroll-x-contain"
		class:cursor-grab={canScroll && !dragging}
		class:cursor-grabbing={dragging}
		class:select-none={dragging}
		onpointerdown={onPointerDown}
		onpointermove={onPointerMove}
		onpointerup={release}
		onpointercancel={cancel}
		onscroll={updateFade}
	>
		<span>{@render children()}</span>
	</div>
</div>

<style>
	/* 两端渐隐驱动变量：长度可插值，transition 平滑（@property 需要全局注册，
	   Svelte scoped 编译会原样保留 at-rule） */
	@property --fade-l {
		syntax: '<length>';
		inherits: false;
		initial-value: 0px;
	}
	@property --fade-r {
		syntax: '<length>';
		inherits: false;
		initial-value: 0px;
	}
	/* 隐藏横向滚动条（Firefox + WebKit） */
	.scrollbar-none {
		scrollbar-width: none;
	}
	.scrollbar-none::-webkit-scrollbar {
		display: none;
	}
	/* 左右两端渐隐：没到头的一侧淡出内容，指示还有内容可拖 */
	.fade-edge {
		transition:
			--fade-l 0.25s ease,
			--fade-r 0.25s ease;
		mask-image: linear-gradient(
			to right,
			transparent 0,
			black var(--fade-l),
			black calc(100% - var(--fade-r)),
			transparent 100%
		);
		-webkit-mask-image: linear-gradient(
			to right,
			transparent 0,
			black var(--fade-l),
			black calc(100% - var(--fade-r)),
			transparent 100%
		);
	}
</style>
