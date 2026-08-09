<script lang="ts">
	import { listPaints, paintId, type PaintInfo } from '$lib/paints.svelte';
	import { stock } from '$lib/stock.svelte';
	import { Plus, X, Search, Package, GripVertical, Eye, EyeOff } from '@lucide/svelte';
	import RangeSlider from '$lib/components/RangeSlider.svelte';
	import { tick, type Snippet } from 'svelte';
	import { Canvas } from '@threlte/core';
	import Scene from './scene.svelte';
	import CollapseGroup from '$lib/components/CollapseGroup.svelte';
	import DropdownButton from '$lib/components/DropdownButton.svelte';
	import { store, sceneProps, ndiv, rangeL, rangeA, rangeB, type SerializedSource } from './gamut.svelte';
	import ColorCode from '$lib/components/ColorCode.svelte';
	import { t } from '$lib/i18n.svelte';
	import { isSm, isCoarse } from '$lib/utils.svelte';

	const allPaints = listPaints();

	interface ColorSource {
		id: string;
		type: 'color';
		text: string;
		rgb: number;
		valid: boolean;
		hidden: boolean;
	}

	interface PaintSource {
		id: string;
		type: 'paint';
		paint: PaintInfo | null;
		searchText: string;
		hidden: boolean;
	}

	interface StockSource {
		id: string;
		type: 'stock';
		hidden: boolean;
	}
	type Source = ColorSource | PaintSource | StockSource;

	function hydrateSources(serialized: SerializedSource[]): Source[] {
		return serialized.map((s) => {
			if (s.type === 'color') {
				const rgb = s.rgb ?? 0;
				const hex = rgb ? '#' + rgb.toString(16).padStart(6, '0') : '';
				return {
					id: s.id,
					type: 'color',
					text: hex,
					rgb,
					valid: rgb !== 0,
					hidden: s.hidden ?? false
				};
			} else if (s.type === 'paint') {
				const paint = s.paintId ? (allPaints.find((p) => paintId(p) === s.paintId) ?? null) : null;
				return { id: s.id, type: 'paint', paint, searchText: '', hidden: s.hidden ?? false };
			} else {
				return { id: s.id, type: 'stock', hidden: s.hidden ?? false };
			}
		});
	}

	function serializeSources(sources: Source[]): SerializedSource[] {
		return sources.map((s) => {
			if (s.type === 'color')
				return { id: s.id, type: 'color', rgb: s.valid ? s.rgb : undefined, hidden: s.hidden };
			else if (s.type === 'paint')
				return {
					id: s.id,
					type: 'paint',
					paintId: s.paint ? paintId(s.paint) : undefined,
					hidden: s.hidden
				};
			else return { id: s.id, type: 'stock', hidden: s.hidden };
		});
	}

	const initialSources: Source[] = store.persisted
		? hydrateSources(store.sources)
		: [{ id: String(store.nextId), type: 'stock', hidden: false }];
	let sources: Source[] = $state(initialSources);
	let nextId = store.persisted ? store.nextId : store.nextId + 1;

	const colors = $derived.by(() => {
		const li = new Set<number>();
		for (const src of sources) {
			if (src.hidden) continue; // 隐藏的来源暂时移出色域（卡片保留）
			if (src.type === 'color' && src.valid) li.add(src.rgb);
			else if (src.type === 'paint' && src.paint) li.add(src.paint.rgb);
			else if (src.type === 'stock') {
				for (const paint of allPaints) if (stock.has(paintId(paint))) li.add(paint.rgb);
			}
		}
		return li;
	});

	// gamut 维护流程（重建/增量/句柄生命周期）在 gamut.svelte.ts 的 sceneProps 里，
	// 这里只把颜色集合喂给它；切页回来输入相同则不重算。
	$effect(() => {
		sceneProps.updateColors(colors);
	});

	const hasStock = $derived(sources.some((s) => s.type === 'stock'));
	const stockCount = $derived(allPaints.filter((p) => stock.has(paintId(p))).length);

	function addColor() {
		const id = String(nextId++);
		sources.push({ id, type: 'color', text: '', rgb: 0, valid: false, hidden: false });
		tick().then(() => {
			(document.querySelector(`[data-card-id="${id}"] input`) as HTMLInputElement | null)?.focus();
		});
	}
	function addPaint() {
		const id = String(nextId++);
		sources.push({ id, type: 'paint', paint: null, searchText: '', hidden: false });
		tick().then(() => {
			(document.querySelector(`[data-card-id="${id}"] input`) as HTMLInputElement | null)?.focus();
		});
	}
	function addStock() {
		if (!hasStock) sources.push({ id: String(nextId++), type: 'stock', hidden: false });
	}
	function removeSource(id: string) {
		sources = sources.filter((s) => s.id !== id);
	}

	// ---- 触屏滑动删除（pointerType touch 才生效，桌面仍用 X 按钮）----
	let swipeId: string | null = $state(null);
	let swipeX = $state(0); // 跟手水平位移
	let swipeOut = $state(false); // 正在滑出（带 transition 的删除动画）
	let swipeW = $state(0);
	let swipeStartX = 0;
	let swipeStartY = 0;
	let swipeMoved = false; // 本次手势是否产生实际位移（用于抑制回弹后的误触 click）
	let swipeCardEl: HTMLElement | null = null;
	let swipeStartEl: HTMLElement | null = null; // 起点在文本框时，位移开始即放弃输入焦点（避免软键盘弹出）

	function onCardPointerDown(e: PointerEvent, src: Source) {
		// 仅触屏：children 区域滑动删除（含 input 上）；grip（垂直排序）除外。
		// 意图区分：点击无位移 → 正常聚焦输入/click；位移 → 滑动删除。
		if (e.pointerType !== 'touch') return;
		if ((e.target as HTMLElement).closest('[data-grip]')) return;
		const el = e.currentTarget as HTMLElement;
		swipeId = src.id;
		swipeCardEl = el;
		swipeStartEl =
			e.target instanceof HTMLElement && (e.target as HTMLElement).closest('input')
				? (e.target as HTMLElement)
				: null;
		swipeW = el.getBoundingClientRect().width;
		swipeStartX = e.clientX;
		swipeStartY = e.clientY;
		swipeX = 0;
		swipeOut = false;
		swipeMoved = false;
		window.addEventListener('pointermove', onSwipeMove);
		window.addEventListener('pointerup', onSwipeUp);
		window.addEventListener('pointercancel', onSwipeCancel);
	}

	function onSwipeMove(e: PointerEvent) {
		if (swipeId === null || swipeOut) return;
		// 排序由左侧 grip 触发，删除手势只管水平跟手
		swipeX = e.clientX - swipeStartX;
		if (Math.abs(swipeX) > 8) {
			swipeMoved = true;
			// 起点是文本框：位移开始即失焦，避免软键盘弹出遮挡
			if (swipeStartEl?.isConnected && typeof swipeStartEl.blur === 'function') {
				swipeStartEl.blur();
			}
		}
	}

	function onSwipeCancel() {
		if (swipeId === null) return;
		swipeX = 0;
		resetSwipe();
	}

	function onSwipeUp() {
		if (swipeId === null) return;
		if (Math.abs(swipeX) > swipeW * 0.35) {
			// 超过阈值：先滑出再删除
			swipeOut = true;
			swipeX = swipeX > 0 ? swipeW : -swipeW;
			const id = swipeId;
			setTimeout(() => {
				removeSource(id);
				resetSwipe();
			}, 180);
		} else {
			// 未达阈值：回弹；若有过位移则抑制随后的 click，防止误触按钮
			if (swipeMoved && swipeCardEl) suppressNextClick(swipeCardEl);
			swipeX = 0;
			resetSwipe();
		}
	}

	// 在一次点击后屏蔽下一次 click（capture 阶段拦截，阻止子元素按钮被误触）
	function suppressNextClick(el: HTMLElement) {
		const handler = (e: Event) => {
			e.preventDefault();
			e.stopPropagation();
			el.removeEventListener('click', handler, true);
		};
		el.addEventListener('click', handler, true);
	}

	function resetSwipe() {
		swipeId = null;
		swipeCardEl = null;
		swipeOut = false;
		swipeStartEl = null;
		window.removeEventListener('pointermove', onSwipeMove);
		window.removeEventListener('pointerup', onSwipeUp);
		window.removeEventListener('pointercancel', onSwipeCancel);
	}

	function parseHexInput(raw: string): { text: string; rgb: number; valid: boolean } {
		let t = raw.trim();
		if (t.startsWith('#')) t = t.slice(1);
		else if (t.startsWith('0x') || t.startsWith('0X')) t = t.slice(2);
		if (/^[0-9a-fA-F]{3}$/.test(t)) t = t[0] + t[0] + t[1] + t[1] + t[2] + t[2];
		if (/^[0-9a-fA-F]{6}$/.test(t)) return { text: raw, rgb: parseInt(t, 16), valid: true };
		return { text: raw, rgb: 0, valid: false };
	}
	function updateColorHex(src: ColorSource, text: string) {
		const parsed = parseHexInput(text);
		src.text = parsed.text;
		src.rgb = parsed.rgb;
		src.valid = parsed.valid;
	}

	function searchPaints(query: string): PaintInfo[] {
		if (!query || query.length < 1) return [];
		const q = query.toLowerCase();
		return allPaints
			.filter(
				(p) =>
					p.code.toLowerCase().includes(q) ||
					p.brand.toLowerCase().includes(q) ||
					p.desc.toLowerCase().includes(q) ||
					`${p.brand} ${p.code}`.toLowerCase().includes(q)
			)
			.slice(0, 20);
	}
	function selectPaint(src: PaintSource, paint: PaintInfo) {
		src.paint = paint;
		src.searchText = '';
	}
	function clearPaint(src: PaintSource) {
		src.paint = null;
		src.searchText = '';
	}

	let paintResults: Record<string, PaintInfo[]> = $state({});
	let highlightedIdx: Record<string, number> = $state({});
	function updatePaintSearch(src: PaintSource, text: string) {
		src.searchText = text;
		paintResults[src.id] = searchPaints(text);
		highlightedIdx[src.id] = -1;
	}

	let selectedColor: { rgb: [number, number, number]; hex: string } | null = $state(null);
	function handleSelect(rgb: [number, number, number], hex: string) {
		selectedColor = { rgb, hex };
	}

	function handleInputTab(e: KeyboardEvent, src: Source) {
		if (e.key === 'Tab' && !e.shiftKey && src.id === sources.at(-1)?.id) {
			e.preventDefault();
			(document.querySelector('[data-add-btn] button') as HTMLElement | null)?.focus();
		}
	}

	// ---- 卡片拖拽排序（pointer events，兼容触屏/鼠标）----
	// 拖拽中只做视觉反馈（被拖卡片 translateY 跟手 + 其他卡片让位动画），
	// drop 时才一次性更新数组并触发持久化。
	let dragId: string | null = $state(null);
	let dragFrom = $state(0); // 起始 index（拖拽期间不变）
	let dragTo = $state(0); // 目标 index（move 时实时计算）
	let dragDy = $state(0); // 被拖卡片跟手位移
	let dragH = $state(0); // 被拖卡片高度 + 卡片间距（让位距离）
	let dragStartY = 0;
	let dragMinDy = -Infinity; // 跟手位移下限（面板顶部边缘）
	let dragMaxDy = Infinity; // 跟手位移上限（面板底部边缘）

	// 找离卡片最近的可滚动/裁剪祖先（列表面板），并算出卡片在其中的跟手位移范围
	function findDragBounds(el: HTMLElement): [number, number] {
		let bound = el.parentElement;
		let p = el.parentElement;
		while (p) {
			if (p.scrollHeight > p.clientHeight || p.scrollWidth > p.clientWidth) {
				bound = p;
				break;
			}
			p = p.parentElement;
		}
		const r = el.getBoundingClientRect();
		const b = bound?.getBoundingClientRect();
		if (!b) return [-Infinity, Infinity];
		return [b.top - r.top, b.bottom - r.bottom];
	}

	function startDrag(e: PointerEvent, src: Source) {
		if (e.button !== 0) return;
		const idx = sources.findIndex((s) => s.id === src.id);
		if (idx < 0) return;
		const el = (e.currentTarget as HTMLElement).closest('[data-card-id]') as HTMLElement | null;
		if (!el) return;
		dragFrom = dragTo = idx;
		dragDy = 0;
		dragH = el.getBoundingClientRect().height + 8; // space-y-2 = 8px gap
		[dragMinDy, dragMaxDy] = findDragBounds(el);
		dragStartY = e.clientY;
		dragId = src.id;
		window.addEventListener('pointermove', onDragMove);
		window.addEventListener('pointerup', endDrag);
	}

	function onDragMove(e: PointerEvent) {
		if (dragId === null) return;
		// 跟手但不超过面板可视边缘（卡片贴边，避免拖出面板被裁剪"消失"）
		dragDy = Math.min(Math.max(e.clientY - dragStartY, dragMinDy), dragMaxDy);
		const cards = Array.from(document.querySelectorAll<HTMLElement>('[data-card-id]'));
		if (!cards.length) return;
		let to = dragFrom;
		for (let i = 0; i < cards.length; i++) {
			const r = cards[i].getBoundingClientRect();
			if (e.clientY >= r.top && e.clientY <= r.bottom) {
				to = e.clientY < r.top + r.height / 2 ? i : i + 1;
				break;
			}
		}
		if (e.clientY < cards[0].getBoundingClientRect().top) to = 0;
		if (e.clientY > cards[cards.length - 1].getBoundingClientRect().bottom) to = cards.length;
		dragTo = to;
	}

	function endDrag() {
		if (dragId !== null && dragTo !== dragFrom && dragTo !== dragFrom + 1) {
			const next = [...sources];
			const [item] = next.splice(dragFrom, 1);
			next.splice(dragTo > dragFrom ? dragTo - 1 : dragTo, 0, item);
			sources = next;
		}
		dragId = null;
		window.removeEventListener('pointermove', onDragMove);
		window.removeEventListener('pointerup', endDrag);
	}

	$effect(() => {
		store.sources = serializeSources(sources);
		store.clipL = sceneProps.clipL;
		store.clipA = sceneProps.clipA;
		store.clipB = sceneProps.clipB;
		store.nextId = nextId;
		store.persist();
	});
</script>

{#snippet colorCard(src: any)}
	<div class="flex items-center gap-2">
		<div
			class="h-7 w-7 shrink-0 rounded-md border border-black/10"
			class:opacity-50={!src.valid}
			style="background-color: {src.valid
				? '#' + src.rgb.toString(16).padStart(6, '0')
				: '#e5e5e5'}"
		></div>
		<div class="min-w-0 flex-1">
			<input
				type="text"
				class="w-full rounded-md border border-gray-300 bg-white px-2 py-1 text-xs text-gray-900 outline-none placeholder:text-gray-400 focus:border-primary-500 focus:ring-1 focus:ring-primary-500 dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 dark:placeholder:text-gray-500"
				placeholder="#ff0000"
				maxlength="9"
				spellcheck="false"
				value={src.text}
				oninput={(e) => updateColorHex(src, (e.target as HTMLInputElement).value)}
				onkeydown={(e) => handleInputTab(e, src)}
			/>
		</div>
	</div>
{/snippet}

{#snippet paintCard(src: any)}
	{#if src.paint}
		<div class="flex items-center gap-2">
			<div
				class="h-7 w-7 shrink-0 rounded-md border border-black/10"
				style="background-color: #{src.paint.rgb.toString(16).padStart(6, '0')}"
			></div>
			<div class="min-w-0 flex-1 text-xs">
				<div class="font-semibold uppercase">{src.paint.brand}/{src.paint.code}</div>
				<div class="truncate text-gray-500 dark:text-gray-400">{src.paint.desc}</div>
			</div>
			<button
				type="button"
				class="cursor-pointer text-[10px] text-gray-400 hover:text-primary-500"
				tabindex="-1"
				onclick={() => clearPaint(src)}>{t('gamut.change')}</button
			>
		</div>
	{:else}
		<div class="min-w-0">
			<div class="relative">
				<Search
					class="pointer-events-none absolute top-1/2 left-2.5 h-3.5 w-3.5 -translate-y-1/2 text-gray-400"
				/>
				<input
					type="text"
					class="w-full rounded-md border border-gray-300 bg-white py-1 pr-2 pl-7 text-xs text-gray-900 outline-none placeholder:text-gray-400 focus:border-primary-500 focus:ring-1 focus:ring-primary-500 dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 dark:placeholder:text-gray-500"
					placeholder={t('gamut.searchPlaceholder')}
					value={src.searchText}
					oninput={(e) => updatePaintSearch(src, (e.target as HTMLInputElement).value)}
					onblur={() => {
						setTimeout(() => {
							paintResults[src.id] = [];
							highlightedIdx[src.id] = -1;
						}, 150);
					}}
					onkeydown={(e) => {
						if (e.key === 'Tab') {
							handleInputTab(e, src);
							return;
						}
						const items = paintResults[src.id];
						if (!items?.length) return;
						let idx = highlightedIdx[src.id] ?? -1;
						if (e.key === 'ArrowDown') {
							e.preventDefault();
							idx = idx + 1 >= items.length ? 0 : idx + 1;
							highlightedIdx[src.id] = idx;
							requestAnimationFrame(() => {
								(e.target as HTMLElement)
									.closest('[data-card-id]')
									?.querySelector('[data-hl]')
									?.scrollIntoView({ block: 'nearest' });
							});
						} else if (e.key === 'ArrowUp') {
							e.preventDefault();
							idx = idx <= 0 ? items.length - 1 : idx - 1;
							highlightedIdx[src.id] = idx;
							requestAnimationFrame(() => {
								(e.target as HTMLElement)
									.closest('[data-card-id]')
									?.querySelector('[data-hl]')
									?.scrollIntoView({ block: 'nearest' });
							});
						} else if (e.key === 'Enter') {
							e.preventDefault();
							if (idx >= 0 && idx < items.length) selectPaint(src, items[idx]);
						} else if (e.key === 'Escape') {
							paintResults[src.id] = [];
							highlightedIdx[src.id] = -1;
						}
					}}
				/>
			</div>
			{#if paintResults[src.id]?.length}
				<div
					class="absolute z-20 mt-1 max-h-48 w-[calc(100%-2rem)] overflow-y-auto rounded-md border border-gray-200 bg-white shadow-lg dark:border-gray-600 dark:bg-gray-700"
				>
					{#each paintResults[src.id] as p, i}
						{@const hl = (highlightedIdx[src.id] ?? -1) === i}
						<button
							type="button"
							data-hl={hl ? '' : undefined}
							class="flex w-full cursor-pointer items-center gap-2 px-2.5 py-1.5 text-left text-xs {hl
								? 'bg-gray-100 dark:bg-gray-600'
								: 'hover:bg-gray-100 dark:hover:bg-gray-600'}"
							onclick={() => selectPaint(src, p)}
						>
							<span
								class="h-4 w-4 shrink-0 rounded-sm border border-black/10"
								style="background-color: #{p.rgb.toString(16).padStart(6, '0')}"
							></span>
							<span class="font-semibold uppercase">{p.brand}/{p.code}</span>
							<span class="truncate text-gray-500 dark:text-gray-400">{p.desc}</span>
						</button>
					{/each}
				</div>
			{/if}
		</div>
	{/if}
{/snippet}

{#snippet stockCard(src: any)}
	<div class="flex items-center gap-2">
		<div
			class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-primary-100 text-primary-600 dark:bg-primary-900 dark:text-primary-300"
		>
			<Package class="h-4 w-4" />
		</div>
		<div class="min-w-0 flex-1 text-xs">
			<div class="font-semibold text-gray-700 dark:text-gray-200">{t('gamut.myStock')}</div>
			<div class="text-gray-500 dark:text-gray-400">
				{t('gamut.stockCount', { n: stockCount })}
			</div>
		</div>
	</div>
{/snippet}

{#snippet cardContent(s: any)}
	{#if s.type === 'color'}
		{@render colorCard(s)}
	{:else if s.type === 'paint'}
		{@render paintCard(s)}
	{:else}
		{@render stockCard(s)}
	{/if}
{/snippet}

{#snippet sourceCard(src: any, children: Snippet<[any]>)}
	{@const valid =
		src.type === 'stock' ? true : src.type === 'color' ? src.valid : src.paint !== null}
	{@const idx = sources.indexOf(src)}
	{@const shift =
		src.id === dragId
			? dragDy
			: dragId !== null
				? dragTo > dragFrom && idx > dragFrom && idx <= dragTo
					? -dragH
					: dragTo < dragFrom && idx >= dragTo && idx < dragFrom
						? dragH
						: 0
				: 0}
	<div
		class="relative flex touch-pan-y items-center gap-2 rounded-lg border p-2 pl-8 select-none {valid
			? 'border-gray-200 bg-gray-50 dark:border-gray-700 dark:bg-gray-800'
			: 'border-dashed border-gray-300 bg-gray-100/70 dark:border-gray-600 dark:bg-gray-800/60'}"
		class:z-20={src.id === dragId}
		class:shadow-lg={src.id === dragId}
		class:ring-2={src.id === dragId}
		class:ring-primary-400={src.id === dragId}
		class:opacity-50={src.hidden}
		class:grayscale={src.hidden}
		data-card-id={src.id}
		role="group"
		onpointerdown={(e) => onCardPointerDown(e, src)}
		style={`transform: translate(${swipeId === src.id ? swipeX : 0}px, ${shift}px); opacity: ${
			swipeId === src.id
				? (1 - Math.min(Math.abs(swipeX) / swipeW, 1)) * (src.hidden ? 0.5 : 1)
				: ''
		}; transition: ${
			src.id === dragId || (swipeId === src.id && !swipeOut)
				? 'none'
				: 'transform 150ms ease, opacity 150ms, box-shadow 150ms'
		}`}
	>
		<!-- 左侧全高触控条：垂直拖拽排序（hidden 时仍可排序，仅 children 内容被禁用） -->
		<button
			type="button"
			data-grip
			class="absolute top-0 bottom-0 left-0 z-10 flex w-8 cursor-grab touch-none items-center justify-center rounded-l-lg text-gray-400 transition-colors hover:bg-gray-200/60 hover:text-gray-600 dark:hover:bg-gray-700/60 dark:hover:text-gray-300"
			class:cursor-grabbing={src.id === dragId}
			aria-label={t('gamut.drag')}
			onpointerdown={(e) => startDrag(e, src)}
		>
			<GripVertical class="h-4 w-4" />
		</button>
		<!-- 正文：卡片内容（input 除外，触控条由 grip 处理，其余区域均可滑动删除） -->
		<div class="m-1 min-w-0 flex-1" class:pointer-events-none={src.hidden}>
			{@render children(src)}
		</div>
		<!-- 右侧操作列：flex 竖排自适应卡片高度，不会溢出 -->
		<div class="flex shrink-0 flex-col items-center">
			{#if !isCoarse()}
				<button
					type="button"
					class="cursor-pointer rounded p-0.5 text-gray-400 hover:bg-gray-200 hover:text-gray-600 dark:hover:bg-gray-700 dark:hover:text-gray-300"
					tabindex="-1"
					aria-label={t('gamut.delete')}
					onclick={() => removeSource(src.id)}><X class="h-4 w-4" /></button
				>
			{/if}
			<button
				type="button"
				class="cursor-pointer rounded p-0.5 text-gray-400 hover:bg-gray-200 hover:text-gray-600 dark:hover:bg-gray-700 dark:hover:text-gray-300"
				tabindex="-1"
				title={src.hidden ? t('gamut.show') : t('gamut.hide')}
				onclick={() => (src.hidden = !src.hidden)}
			>
				{#if src.hidden}
					<EyeOff class="h-4 w-4" />
				{:else}
					<Eye class="h-4 w-4" />
				{/if}
			</button>
		</div>
	</div>
{/snippet}

{#snippet addSourceBtn()}
	{#snippet color()}<span
			class="h-3.5 w-3.5 rounded-full border border-black/15 bg-gradient-to-br from-red-400 via-green-400 to-blue-500"
		></span>{t('gamut.color')}{/snippet}
	{#snippet paint()}<Search class="h-3.5 w-3.5" />{t('gamut.paint')}{/snippet}
	{#snippet stock()}<Package class="h-3.5 w-3.5" />{t('gamut.myStock')}{/snippet}
	{@const options = [
		{ onclick: addColor, children: color },
		{ onclick: addPaint, children: paint },
		{ onclick: addStock, children: stock, disabled: hasStock }
	]}
	<DropdownButton {options} placement="bottom-end">
		<Plus class="h-3.5 w-3.5" />{t('gamut.add')}
	</DropdownButton>
{/snippet}

<div class="flex h-full flex-col sm:flex-row">
	<div class="relative h-80 min-w-0 bg-gray-950 sm:h-auto sm:flex-1">
		<Canvas>
			<Scene {ndiv} {...sceneProps} onselect={handleSelect} />
		</Canvas>
	</div>

	<div
		class="flex flex-1 flex-col overflow-hidden bg-white sm:w-[40%] sm:max-w-86 sm:flex-none sm:shrink-0 dark:bg-gray-900"
	>
		<CollapseGroup
			title={t('gamut.clipping')}
			isOpen={isSm()}
			class="range-sliders-root grid grid-flow-row gap-3 overflow-hidden px-6 py-4"
		>
			<RangeSlider
				gradient={['#000', '#fff']}
				min={rangeL[0]}
				max={rangeL[1]}
				step={1}
				bind:value={sceneProps.clipL}
			/>
			<RangeSlider
				gradient={['#67ff00', '#ff0000']}
				min={rangeA[0]}
				max={rangeA[1]}
				step={1}
				bind:value={sceneProps.clipA}
			/>
			<RangeSlider
				gradient={['#0021ff', '#fff504']}
				min={rangeB[0]}
				max={rangeB[1]}
				step={1}
				bind:value={sceneProps.clipB}
			/>
		</CollapseGroup>

		<CollapseGroup title={t('gamut.sources')} class="flex-1 space-y-2 overflow-y-auto px-6 py-2">
			<div data-add-btn class="flex items-center justify-between">
				<span class="text-xs text-gray-500 dark:text-gray-400"
					>{t('gamut.colorsInGamut', { n: colors.size })}</span
				>
				{@render addSourceBtn()}
			</div>
			{#each sources as src (src.id)}
				{@render sourceCard(src, cardContent)}
			{:else}
				<div class="flex flex-col items-center justify-center py-12 text-center text-gray-400">
					<div class="mb-2 text-sm">{t('gamut.noSources')}</div>
					<div class="text-xs">{@html t('gamut.clickAddHint')}</div>
				</div>
			{/each}
		</CollapseGroup>

		{#if selectedColor}
			<div class="mt-auto border-t border-gray-200 px-2 py-2 dark:border-gray-700">
				<div class="flex items-center gap-2">
					<div
						class="h-7 w-7 shrink-0 rounded-md border border-black/10"
						style="background-color: rgb({selectedColor.rgb[0]},{selectedColor
							.rgb[1]},{selectedColor.rgb[2]})"
					></div>
					<ColorCode
						re="#([0-9a-fA-F]{3}|[0-9a-fA-F]{6})"
						text={selectedColor.hex}
						class="w-full"
						onfocus={(e) => (e.target as HTMLInputElement).select()}
						readonly
					/>
				</div>
			</div>
		{/if}
	</div>
</div>
