<script lang="ts">
	import { SURFACE_BITS, rgbToHex, type PaintInfo } from '$lib/paints.svelte';
	import { onDestroy, onMount } from 'svelte';
	import { clamp } from '$lib/utils.svelte';

	interface Props {
		paint: PaintInfo;
		disablePaging?: boolean;
		pagingStyle?: string;
		// provider 输出（bindable）：每页的语义色名（对应 layout.css 的 --pa-base-*）
		bases?: string[];
		/** 当前页（多页模式，外部可读写） */
		idx?: number;
		/**
		 * 拖拽灵敏度：鼠标位移 → 滑动距离的映射倍数（>1 拖一点滑很远、更易翻页；<1 更迟钝）。
		 * 默认按指针类型：触屏 1，鼠标/触控笔 0.85。
		 */
		dragSensitivity?: number;
		/**
		 * 拖拽阻力（0~∞）：越大越难拖拽（越界后轨道跟手比例越小）。
		 * 0 = 无阻力（越界完全跟手），∞ = 拉不动。
		 * 映射：越界跟手比例 = 1/(1+resistance)。默认按指针类型：触屏 4，鼠标/触控笔 10。
		 */
		dragResistance?: number;
	}

	let {
		paint,
		disablePaging,
		pagingStyle,
		bases = $bindable(),
		idx = 0,
		dragSensitivity,
		dragResistance
	}: Props = $props();

	const hex = $derived(rgbToHex(paint.rgb));

	// 特种色示意性渲染模式：Clear > Metallic > Pearl > Fluo，其余平涂
	// 视觉语言刻意保持"美术示意"而非拟真（数据只有 hex，不能假装对实际观感负责）：
	// 金属=全局跟随光带 / 透明=叠层+网格 / 珠光=全局跟随光晕 / 荧光=呼吸
	const mode = $derived(
		paint.surfaces & SURFACE_BITS.PA
			? 'pearl'
			: paint.surfaces & SURFACE_BITS.ME
				? 'metallic'
				: paint.surfaces & SURFACE_BITS.C
					? 'clear'
					: paint.surfaces & SURFACE_BITS.FL
						? 'fluo'
						: 'flat'
	);

	// ---- provider 抽象：输入 paint，输出多页 { base, css }；目前只实现 pa（珠光变体） ----
	// base = 语义色名（渲染 var(--pa-base-<base>)，token 定义在 layout.css :root）；
	// css  = 该页的 CSS 变量串（--t0/--t1/--t2）。
	// 单页时 pages 为空 → 走原有单 div 渲染，行为不变。
	let exPages: { base: string; css: string }[] = $derived.by(() => {
		if (mode != 'pearl') {
			return [];
		}

		const li: any = paint.extra?.pa ?? [];
		return li.map(({ base, t0, t1, t2 }: any) => {
			let css = `--t1:${t1};`;
			if (t2) {
				css += ` --t2:${t2};`;
			}
			if (t0) {
				css += ` --t0:${t0};`;
			}
			return { base, css };
		});
	});

	// 语义色名数组透传给外部（bindable），供外部做 base 相关展示
	$effect(() => {
		bases = exPages.length ? exPages.map((p) => p.base) : undefined;
	});

	const N = $derived(exPages.length);
	const useEx = $derived(N > 0);
	const useExTrack = $derived(useEx && !disablePaging);
	const prev = $derived(useExTrack ? (idx - 1 + N) % N : 0);
	const cur = $derived(useExTrack ? idx % N : 0);
	const next = $derived(useExTrack ? (idx + 1) % N : 0);

	let el = $state<HTMLDivElement | undefined>();
	let trackPos = $state(0);
	let anim: number | null = null;
	let animTo = 0; // 当前动画目标偏移
	let animDir = $state(0); // 当前动画翻页方向（0 = 弹回不翻页）

	// ---- 手势（仅多页启用）：触摸走 touch 事件（浏览器触摸滚动时会 pointercancel，
	// pointer 事件不可靠——同 Drawer 经验），鼠标/触控笔走 pointer 事件。
	// 两条路径共用跟手账本与释放判定；位移超过阈值才接管，tap 不接管（保住圆点/卡片点击）。 ----
	// 手势参数：按指针类型给默认（PC 鼠标行程大、精度高：跟手映射更钝 + 更强阻尼），
	// 传入 props 时以 props 为准。
	let finePointer = $state(false);
	onMount(() => {
		finePointer = window.matchMedia('(pointer: fine)').matches;
	});
	// dragSens：位移映射倍数（鼠标位移 → 滑动距离），>1 更易滑，<1 更钝；接管/翻页判定同用映射后位移
	const dragSens = $derived(dragSensitivity ?? (finePointer ? 0.85 : 1));
	// dragResistance（0~∞）越大越难拖：越界后轨道仍跟手的比例 rubber = 1/(1+resistance)
	const rubber = $derived(1 / (1 + (dragResistance ?? (finePointer ? 10 : 4))));
	const DRAG_START = 6; // px：接管阈值（按映射后位移判定）
	// 目标页：动画开始时即切换（不等动画完成），消除 active 高亮滞后；窗口内容仍按动画完成换（平移无缝）
	const activeIdx = $derived(animDir !== 0 ? (idx + animDir + N) % N : idx);
	let armed = false; // 本元素收到过 down：从别的卡滑过来（未 down）的卡不接管
	let dragging = false;
	let sx = 0; // 起始 X
	let sy = 0; // 起始 Y（touch 方向判定用）
	let grabX = 0; // 接管瞬间的 X（位移从接管点起算，接管前可能有滚动/微动）
	let basePos = 0;
	let lastX = 0;
	let lastT = 0;
	let samples: { x: number; t: number }[] = []; // 最近 80ms 采样窗口，速度不掺水
	let suppressClick = false;

	function cancelAnim() {
		if (anim !== null) {
			cancelAnimationFrame(anim);
			anim = null;
			// 打断动画时提交到终点：翻页动画补上换窗口，弹回动画归位。
			// 否则轨道停在半路，后续手势的基准/判定全部错位（回滚不动、连翻乱跳）。
			if (animDir !== 0) {
				idx = (idx + animDir + N) % N;
				animTo = -(el?.clientWidth ?? 0);
			}
			trackPos = animTo;
			animDir = 0;
		}
	}

	/**
	 * 松手/圆点点击后的滑动动画：easeOutCubic 干净收敛（无振荡/拖尾），
	 * 时长按位移/初速度换算（100–240ms，避免“松手后还滚很久”的拖尾感）。
	 * dir ≠ 0 表示翻页：动画到达目标位后换窗口（idx 取模，首尾无缝循环）并瞬移回中心。
	 */
	function slide(to: number, dir: number, v0: number, done?: () => void) {
		cancelAnim();
		const W = el?.clientWidth ?? 0;
		const from = trackPos;
		const dist = Math.abs(to - from);
		const dur = clamp(dist / Math.max(Math.abs(v0), 0.45), 100, 240);
		animTo = to;
		animDir = dir;
		const t0 = performance.now();
		const step = (now: number) => {
			const t = Math.min((now - t0) / dur, 1);
			trackPos = from + (to - from) * (1 - Math.pow(1 - t, 3)); // easeOutCubic
			if (t < 1) {
				anim = requestAnimationFrame(step);
			} else {
				anim = null;
				trackPos = to;
				if (dir !== 0) {
					idx = (idx + dir + N) % N;
					trackPos = -W;
				}
				animDir = 0;
				done?.();
			}
		};
		anim = requestAnimationFrame(step);
	}

	function onpointerdown(e: PointerEvent) {
		if (e.pointerType === 'touch') return; // 触摸走 touch 路径
		if (e.button !== 0) return;
		// 阻止 pointerdown 启动原生文本选择/HTML5 拖拽：否则一旦有选区残留，
		// 下次从选区上拖会进原生拖拽（not-allowed 光标 + 指针事件被抢，拖拽全失效）。
		// pointerdown 的 preventDefault 不影响 click 派发（圆点/卡片选中照常）。
		e.preventDefault();
		armed = true; // 只有收到 down 的卡片能接管（跨卡滑动不误触）
		cancelAnim(); // 打断遗留动画并提交到终点，新手势从对齐位置起算
		dragging = false;
		suppressClick = false;
		sx = lastX = e.clientX;
		lastT = performance.now();
		basePos = trackPos;
		samples = [];
	}

	/** 手势移动（鼠标/触控笔）：必须处于按下状态（hover 移动 buttons=0 不接管），
	 * 超过阈值才接管 + capture（保住 tap 的 click 目标） */
	function onpointermove(e: PointerEvent) {
		if (e.pointerType === 'touch') return;
		if (!armed) return; // 本卡没收到 down（从别的卡滑过来）：不接管
		if (!(e.buttons & 1)) return; // 未按下：hover 移动/悬停不动，绝不接管
		if (!dragging) {
			if (Math.abs(e.clientX - sx) * dragSens < DRAG_START) return;
			dragging = true;
			grabX = e.clientX;
			lastX = e.clientX;
			lastT = performance.now();
			samples = [];
			el?.setPointerCapture(e.pointerId);
			e.preventDefault();
		}
		track(e.clientX);
	}

	/** 手势开始（触摸）：记录账本，不立即接管 */
	function ontouchstart(e: TouchEvent) {
		const t = e.touches[0];
		if (!t) return;
		armed = true; // 只有收到 down 的卡片能接管
		cancelAnim(); // 打断遗留动画并提交到终点
		dragging = false;
		suppressClick = false;
		sx = t.clientX;
		sy = t.clientY;
		lastX = t.clientX;
		lastT = performance.now();
		basePos = trackPos;
		samples = [];
	}

	/** 手势移动（触摸）：横向判定后才 preventDefault 接管；纵向放行给页面滚动 */
	function ontouchmove(e: TouchEvent) {
		const t = e.touches[0];
		if (!t) return;
		if (!armed) return; // 本卡没收到 down：不接管
		if (!dragging) {
			const dx = t.clientX - sx;
			const dy = t.clientY - sy;
			// 未横向接管：微动或纵向手势 → 放行（浏览器滚动 / 保住 tap）
			if (Math.abs(dx) * dragSens < DRAG_START || Math.abs(dx) <= Math.abs(dy)) return;
			e.preventDefault();
			dragging = true;
			grabX = t.clientX;
			lastX = t.clientX;
			lastT = performance.now();
			samples = [];
			return;
		}
		track(t.clientX);
	}

	/** 跟手：更新轨道位移 + 采样速度（80ms 窗口） */
	function track(x: number) {
		const now = performance.now();
		if (samples.length === 0 || x !== samples[samples.length - 1].x) {
			samples.push({ x, t: now });
		}
		while (samples.length > 1 && now - samples[0].t > 80) samples.shift();
		lastX = x;
		lastT = now;
		// 跟手位移：原始位移 × 映射灵敏度（dragSens）。3 窗口轨道合法范围 [-2W, 0]，
		// 越界部分按 rubber 阻尼（=1/(1+resistance)，resistance 越大越拉不动），
		// 拖再多也不露底色、松手弹性归位。接管/翻页判定同用映射后位移。
		const W = el?.clientWidth ?? 0;
		const raw = basePos + (x - grabX) * dragSens;
		const min = -2 * W;
		const max = 0;
		if (raw < min) trackPos = min + (raw - min) * rubber;
		else if (raw > max) trackPos = max + (raw - max) * rubber;
		else trackPos = raw;
	}

	function onpointerup(e: PointerEvent) {
		if (e.pointerType === 'touch') return;
		finishDrag();
	}

	function ontouchend() {
		finishDrag();
	}

	function onpointercancel(e: PointerEvent) {
		if (e.pointerType === 'touch') return;
		cancelDrag();
	}

	function ontouchcancel() {
		cancelDrag();
	}

	/** 释放判定：速度优先，位移过半兜底 */
	function finishDrag() {
		armed = false;
		if (!dragging) return;
		dragging = false;
		const W = el?.clientWidth ?? 0;
		let v = 0;
		if (samples.length >= 2) {
			const a = samples[0];
			const b = samples[samples.length - 1];
			const dt = b.t - a.t;
			if (dt > 0) v = (b.x - a.x) / dt; // px/ms
		}
		samples = [];
		const dx = (lastX - grabX) * dragSens; // 映射后等效位移
		let dir = 0;
		if (v * dragSens < -0.35 || dx < -W / 2) dir = 1;
		else if (v * dragSens > 0.35 || dx > W / 2) dir = -1;
		// 真拖拽过（翻页/弹回）：拦截随后的 click，防误触卡片选中；tap 不拦截
		if (dir !== 0) {
			suppressClick = true;
			const h = (ev: Event) => {
				ev.stopPropagation();
				ev.preventDefault();
			};
			document.addEventListener('click', h, true);
			setTimeout(() => document.removeEventListener('click', h, true), 0);
		}
		slide(dir === 1 ? -2 * W : dir === -1 ? 0 : -W, dir, v * dragSens);
	}

	/** 手势取消：弹回原位（pointercancel / touchcancel 共用） */
	function cancelDrag() {
		armed = false;
		if (!dragging) return;
		dragging = false;
		samples = [];
		slide(-(el?.clientWidth ?? 0), 0, 0);
	}

	/** 手势开始（鼠标/触控笔）：记录账本，不立即接管 */
	const handlers = $derived(
		useExTrack && N > 1
			? {
					onpointerdown,
					onpointerup,
					onpointermove,
					onpointercancel,
					ontouchstart,
					ontouchend,
					ontouchmove,
					ontouchcancel
				}
			: {}
	);

	/** 圆点点击：3 窗口轨道一次只动一页，跨页逐页滑到目标 */
	function go(i: number) {
		if (i === idx) return;
		cancelAnim(); // 打断遗留动画并提交，再从对齐位置起滑
		trackPos = -(el?.clientWidth ?? 0);
		const dir = i > idx ? 1 : -1;
		slide(dir === 1 ? -(el?.clientWidth ?? 0) * 2 : 0, dir, 0, () => {
			if (idx !== i) go(i);
		});
	}

	// 容器尺寸变化（响应式网格）后轨道基准跟随
	$effect(() => {
		if (!useExTrack || !el) return;
		const ro = new ResizeObserver(() => {
			trackPos = -(el?.clientWidth ?? 0);
		});
		ro.observe(el);
		return () => ro.disconnect();
	});

	onDestroy(() => {
		// 组件销毁时停掉残留动画，避免 rAF 继续写已卸载组件的 $state
		if (anim !== null) cancelAnimationFrame(anim);
		anim = null;
		animDir = 0;
	});
</script>

<div
	role="button"
	tabindex="-1"
	bind:this={el}
	style="--c: {hex}; {useEx && !useExTrack ? exPages[0].css : ''}"
	class="fx-root {useExTrack ? 'fx-swatch-ex-track' : `${mode} fx-swatch ${useEx ? 'fx-ex' : ''}`}"
	class:gesture={useExTrack}
	{...handlers}
>
	{#if useExTrack}
		<div class="fx-track" style="transform: translateX({trackPos}px)">
			<div class="{mode} fx-swatch fx-ex" style={exPages[prev].css}>{@render swatch()}</div>
			<div class="{mode} fx-swatch fx-ex" style={exPages[cur].css}>{@render swatch()}</div>
			<div class="{mode} fx-swatch fx-ex" style={exPages[next].css}>{@render swatch()}</div>
		</div>
		<div
			class="fx-dots"
			role="tablist"
			aria-label="pages"
			style={pagingStyle ?? 'left:6px;top:6px'}
		>
			{#each exPages as { base }, i}
				<button
					type="button"
					class="fx-dot {i === activeIdx ? 'active' : ''}"
					role="tab"
					aria-selected={i === activeIdx}
					aria-label={`page ${i + 1}`}
					onclick={(e) => {
						e.stopPropagation();
						go(i);
					}}
					style="--base: var(--pa-base-{base})"
				></button>
			{/each}
		</div>
	{:else}
		{@render swatch()}
	{/if}
</div>

{#snippet swatch()}
	{#if mode === 'metallic' || mode === 'pearl'}
		<div class="fx-band"></div>
	{/if}

	{#if mode === 'clear'}
		<div class="fx-tint"></div>
		<div class="fx-grid"></div>
	{/if}
{/snippet}

<style>
	.fx-root {
		position: absolute;
		inset: 0;
		overflow: hidden;
		pointer-events: none;
		user-select: none; /* 防拖选文本残留 → 原生拖拽（not-allowed 光标 + 拖拽失效） */
		-webkit-user-drag: none;
		/* 容器查询：图案尺寸用 cqmax（容器最小边）等比缩放，跨尺寸/aspect 几何一致 */
		container-type: size;
	}

	.fx-root {
		background-color: var(--c);
	}

	/* ---- 多页手势：接管横向拖动（纵向滚动留给页面） ---- */
	.fx-root.gesture {
		pointer-events: auto;
		touch-action: pan-y;
	}
	.fx-root.fx-swatch-ex-track {
		background-color: black;
	}
	.fx-track {
		position: absolute;
		inset: 0;
		display: flex;
		will-change: transform;
	}
	.fx-track .fx-swatch {
		flex: 0 0 100%;
		min-width: 0;
		position: relative;
		overflow: hidden;
	}
	/* 多页时根 ::after 闪粉关掉，由每页 .fx-page::after 负责（轨道平移才不会穿帮） */
	.fx-root.gesture::after {
		content: none;
	}

	.fx-band {
		position: absolute;
		transform-origin: 50% 50%;
		inset: 50%;
		translate: calc(-50% + var(--trans-x, 0px)) calc(-50% + var(--trans-y, 0px));
		rotate: 45deg;
		--hypot: hypot(100cqmax, 100cqmin);
		width: var(--hypot);
		height: var(--hypot);
	}

	/* ---- 金属：平涂基色 + 全局跟随光带（高光即立体感，无静态渐变） ---- */
	.fx-swatch.metallic {
		--c0: oklch(from var(--c) 0.9 c h);
		/* --c1: color-mix(in srgb, var(--c0) 60%, white); */
		--c1: oklch(from var(--c) 0.95 calc(c * 0.6) h);
		--c2: oklch(from var(--c) calc(l * 1.2 + 0.2) calc(c * 0.8) h);
	}
	/* .fx-swatch.metallic .fx-band {
		width: 50cqmax;
		height: var(--hypot);
		background: linear-gradient(
			to right,
			transparent,
			color-mix(in srgb, var(--c2) 10%, transparent) 50%,
			transparent
		);
		--trans-x: 18cqmax;
	} */
	.fx-swatch.metallic .fx-band::before {
		content: '';
		position: absolute;
		height: var(--hypot);
		width: 50cqmax;
		margin-left: -25cqmax;
		left: 60%;
		background: linear-gradient(
			to right,
			/* */ /* */ transparent,
			color-mix(in srgb, var(--c2) 10%, transparent) 50%,
			transparent
		);
	}
	.fx-swatch.metallic .fx-band::after {
		content: '';
		position: absolute;
		height: 100%;
		width: 2%;
		margin-left: -1%;
		left: 60%;
		background: linear-gradient(
			to bottom,
			color-mix(in srgb, var(--c2) 30%, transparent),
			color-mix(in srgb, var(--c1) 75%, transparent),
			color-mix(in srgb, var(--c2) 50%, transparent)
		);
		/* background: var(--c2); */
		box-shadow: 0 0 10cqmax var(--c2);
	}

	/* ---- 珠光：光带几何与金属一致（斜向高光），两端偏色相反色相（双色性），
	   表面叠加微量闪粉。无 flip 数据，示意性近似 ---- */
	.fx-swatch.pearl:not(.fx-ex) {
		--v1: oklch(from var(--c) calc(l * 1.5) c calc(h + 60));
		--v0: oklch(from var(--v1) calc(l * 1.5) c h);
		--v2: oklch(from var(--c) l calc(c * 1.5) calc(h - 60));
		background-color: var(--c);
	}
	.fx-swatch.pearl.fx-ex {
		--v1: var(--t1);
		--v0: oklch(from var(--v1) calc(l * 2) c h);
		--v2: var(--t2, var(--v1));
		/* --v2: var(--t2, color-mix(in srgb, var(--v1) 60%, transparent)); */
		background-color: var(--t0, var(--c));
	}
	.fx-swatch.pearl .fx-band {
		background: linear-gradient(
			/* */ to right,
			var(--v2) 15%,
			color-mix(in srgb, var(--v2) 50%, transparent) 28%,
			transparent 45%,
			transparent 52%,
			color-mix(in srgb, var(--v1) 30%, transparent) 56%,
			var(--v1) 60%,
			color-mix(in srgb, var(--v1) 50%, transparent) 72%,
			transparent 92%
		);
	}
	.fx-swatch.pearl .fx-band::after {
		content: '';
		position: absolute;
		top: 0;
		bottom: 0;
		left: 0;
		left: calc(60% - 4%);
		width: 8%;
		background: linear-gradient(to bottom, var(--v1), var(--v0), var(--v1));
	}
	/* 生成闪粉表（cqmax 直书格式）：27 点，半径 1-2.5 正态分布（μ=1.75 σ=0.3 拒绝采样），
	 位置 halton 低差异序列均匀覆盖（百分比），透明度伪随机 0.3-0.75
	 const N = 27;
	 function norm() {
	   let u = 0, v = 0;
	   while (u === 0) u = Math.random();
	   while (v === 0) v = Math.random();
	   return Math.sqrt(-2 * Math.log(u)) * Math.cos(2 * Math.PI * v);
	 }
	 function radius() {
	   while (true) {
	     const r = 1.75 + norm() * 0.3;
	     if (r >= 1 && r <= 2.5) return r;
	   }
	 }
	 function halton(idx, base) {
	   let f = 1, r = 0;
	   while (idx > 0) { f /= base; r += f * (idx % base); idx = Math.floor(idx / base); }
	   return r;
	 }
	 const rows = [];
	 for (let i = 0; i < N; i++) {
	   const x = halton(i + 1, 2) * 100;
	   const y = halton(i + 1, 3) * 100;
	   const r = radius();
	   const a = 0.3 + Math.random() * 0.45;
	   rows.push(`\t\t\tradial-gradient(circle ${r.toFixed(1)}cqmax at ${x.toFixed(2)}% ${y.toFixed(2)}%, rgba(255, 255, 255, ${a.toFixed(2)}) 50%, transparent 51%),`);
	 }
	 console.log(rows.join('\n')); */
	/* 微量闪粉：单一大 tile 内 27 个错位点（halton 低差异序列均匀覆盖，位置/透明度伪随机），
	   平铺后呈伪随机分布，避免规律网格的机械感。半径 1-2.5cqmax 正态分布（μ=1.75 σ=0.3），
	   半径直书 cqmax、坐标百分比化，tile 尺寸随容器等比缩放 */
	/* .fx-swatch.pearl::after {
		content: '';
		position: absolute;
		inset: 0;
		background-image:
			radial-gradient(circle 2.1cqmax at 50% 33.33%, rgba(255, 255, 255, 0.3) 50%, transparent 51%),
			radial-gradient(circle 2.2cqmax at 25% 66.67%, rgba(255, 255, 255, 0.7) 50%, transparent 51%),
			radial-gradient(circle 1.6cqmax at 75% 11.11%, rgba(255, 255, 255, 0.5) 50%, transparent 51%),
			radial-gradient(
				circle 2.1cqmax at 12.5% 44.44%,
				rgba(255, 255, 255, 0.42) 50%,
				transparent 51%
			),
			radial-gradient(
				circle 1.9cqmax at 62.5% 77.78%,
				rgba(255, 255, 255, 0.67) 50%,
				transparent 51%
			),
			radial-gradient(
				circle 1.6cqmax at 37.5% 22.22%,
				rgba(255, 255, 255, 0.51) 50%,
				transparent 51%
			),
			radial-gradient(
				circle 1.5cqmax at 87.5% 55.56%,
				rgba(255, 255, 255, 0.57) 50%,
				transparent 51%
			),
			radial-gradient(
				circle 1.7cqmax at 6.25% 88.89%,
				rgba(255, 255, 255, 0.42) 50%,
				transparent 51%
			),
			radial-gradient(
				circle 1.5cqmax at 56.25% 3.7%,
				rgba(255, 255, 255, 0.3) 50%,
				transparent 51%
			),
			radial-gradient(
				circle 1.5cqmax at 31.25% 37.04%,
				rgba(255, 255, 255, 0.53) 50%,
				transparent 51%
			),
			radial-gradient(
				circle 2cqmax at 81.25% 70.37%,
				rgba(255, 255, 255, 0.58) 50%,
				transparent 51%
			),
			radial-gradient(
				circle 1.1cqmax at 18.75% 14.81%,
				rgba(255, 255, 255, 0.69) 50%,
				transparent 51%
			),
			radial-gradient(
				circle 1.6cqmax at 68.75% 48.15%,
				rgba(255, 255, 255, 0.42) 50%,
				transparent 51%
			),
			radial-gradient(
				circle 1.8cqmax at 43.75% 81.48%,
				rgba(255, 255, 255, 0.6) 50%,
				transparent 51%
			),
			radial-gradient(
				circle 2.3cqmax at 93.75% 25.93%,
				rgba(255, 255, 255, 0.35) 50%,
				transparent 51%
			),
			radial-gradient(
				circle 2.4cqmax at 3.13% 59.26%,
				rgba(255, 255, 255, 0.47) 50%,
				transparent 51%
			),
			radial-gradient(
				circle 2.1cqmax at 53.13% 92.59%,
				rgba(255, 255, 255, 0.56) 50%,
				transparent 51%
			),
			radial-gradient(
				circle 2.1cqmax at 28.13% 7.41%,
				rgba(255, 255, 255, 0.59) 50%,
				transparent 51%
			),
			radial-gradient(
				circle 2.1cqmax at 78.13% 40.74%,
				rgba(255, 255, 255, 0.52) 50%,
				transparent 51%
			),
			radial-gradient(
				circle 1.7cqmax at 15.63% 74.07%,
				rgba(255, 255, 255, 0.31) 50%,
				transparent 51%
			),
			radial-gradient(
				circle 1.9cqmax at 65.63% 18.52%,
				rgba(255, 255, 255, 0.49) 50%,
				transparent 51%
			),
			radial-gradient(
				circle 1.7cqmax at 40.63% 51.85%,
				rgba(255, 255, 255, 0.51) 50%,
				transparent 51%
			),
			radial-gradient(
				circle 1.8cqmax at 90.63% 85.19%,
				rgba(255, 255, 255, 0.67) 50%,
				transparent 51%
			),
			radial-gradient(
				circle 1.5cqmax at 9.38% 29.63%,
				rgba(255, 255, 255, 0.44) 50%,
				transparent 51%
			),
			radial-gradient(
				circle 1.4cqmax at 59.38% 62.96%,
				rgba(255, 255, 255, 0.44) 50%,
				transparent 51%
			),
			radial-gradient(
				circle 2.1cqmax at 34.38% 96.3%,
				rgba(255, 255, 255, 0.63) 50%,
				transparent 51%
			),
			radial-gradient(
				circle 1.9cqmax at 84.38% 1.23%,
				rgba(255, 255, 255, 0.54) 50%,
				transparent 51%
			);
		opacity: 0.25;
		pointer-events: none;
	} */

	/* ---- 透明：金属灰底 + 半透色 + 网格（图层隐喻，示意而非拟真） ---- */
	.fx-swatch.clear {
		background-image: linear-gradient(to bottom, #e8e8e8, #bcbcbc 45%, #8c8c8c);
	}
	.fx-swatch.clear .fx-tint {
		position: absolute;
		inset: 0;
		background-color: var(--c);
		opacity: 0.72;
	}
	.fx-swatch.clear .fx-grid {
		position: absolute;
		inset: 0;
		background-image:
			linear-gradient(rgba(255, 255, 255, 0.24) 1px, transparent 1px),
			linear-gradient(90deg, rgba(255, 255, 255, 0.24) 1px, transparent 1px);
		background-size: 25cqmin 25cqmin;
	}

	/* ---- 荧光：平涂基色 + 圆角矩形辉光。
	   亮度分布用 radial（等值线圆、自然衰减无硬边），外形由 border-radius 裁成圆角矩形，
	   角部是圆的边界而非两个正交渐变的乘积塌陷（即"radial 画法 + 矩形外形"） ---- */
	.fx-swatch.fluo::after {
		content: '';
		position: absolute;
		width: 100%;
		height: 100%;
		left: 0;
		top: 0;
		background: radial-gradient(
			ellipse,
			oklch(from var(--c) calc(l * 1.2) c h),
			oklch(from var(--c) calc(l * 1.1) c h) 40%,
			oklch(from var(--c) calc(l * 1) c h) 60%,
			oklch(from var(--c) calc(l * 0.8) c h) 100%
		);
		pointer-events: none;
	}

	/* ---- 左上角页指示器：半透明胶囊 + 圆点，尽量少压色卡视觉 ---- */
	/* 圆点是 UI 控件（固定尺寸），图案才随容器缩放；超小容器（<56px）隐藏分页指示，翻页靠拖拽 */
	.fx-dots {
		position: absolute;
		display: none;
		gap: 7px;
	}
	@container (min-width: 56px) {
		.fx-dots {
			display: flex;
		}
	}
	.fx-dot {
		position: relative;
		width: 8px;
		height: 8px;
		border-radius: 50%;
		padding: 0;

		background-color: var(--base);
		box-shadow: 0 0 0 1px rgba(0 0 0 / 0.3);

		cursor: pointer;
		transition: transform 0.2s ease;
	}
	.fx-dot:hover {
		transform: scale(1.15);
	}
	.fx-dot::after {
		content: '';
		position: absolute;
		inset: -4px;
		border-radius: 50%;
		transition: box-shadow 0.2s ease;
	}
	.fx-dot.active::after {
		box-shadow:
			0 0 0 2px oklch(from var(--base) l c h / 0.3) inset,
			0 0 0 2px rgba(255, 255, 255, 1) inset,
			0 0 0 3px rgb(0 0 0 / 0.5) inset,
			0 0 0 1px rgb(0 0 0 / 0.5),
			0 0 0 1px rgb(0 0 0 / 0.3);
	}
</style>
