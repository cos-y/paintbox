<script lang="ts">
	import { type Oklch } from 'culori/fn';

	import Hsl from '$lib/components/Hsl.svelte';
	import Rgb from '$lib/components/Rgb.svelte';
	import {
		Box,
		Camera,
		ChevronDown,
		Cylinder,
		FlaskConical,
		Palette,
		Droplet,
		Pipette,
		ArrowLeftRight
	} from '@lucide/svelte';
	import { Badge, Button, Dropdown } from 'flowbite-svelte';
	import CameraPicker from '$lib/components/CameraPicker.svelte';
	import PanText from '$lib/components/PanText.svelte';
	import { getCatalog, floatRgbToCss, type PaintInfo, getPaintById } from '$lib/paints.svelte';
	import { baseLabels, surfaceLabels } from '$lib/paintInfo';
	import PaintSearch from '$lib/components/PaintSearch.svelte';
	import ColorCode from '$lib/components/ColorCode.svelte';
	import PaintDetail from '$lib/components/PaintDetail.svelte';
	import { drawer } from '$lib/drawer.svelte';
	import { getBrandMeta, getSerieMeta, serieThumb } from '$lib/meta';
	import { clamp, similarity, isSm, toRgb, toOklch, toHwb } from '$lib/utils.svelte';
	import MultiSelect from '$lib/components/MultiSelect.svelte';
	import Select from '$lib/components/Select.svelte';
	import { store, rt } from './search.svelte';
	import { t } from '$lib/i18n.svelte';
	import { untrack } from 'svelte';
	import { paintDesc } from '$lib/i18ndyn.svelte';
	import { isTauri } from '@tauri-apps/api/core';

	/** store.color（rgb int）与色板 oklch 互转 */
	const oklchToInt = (c: Oklch): number => {
		const { r, g, b } = toRgb(c);
		return (
			(clamp(Math.round(r * 255), 0, 255) << 16) |
			(clamp(Math.round(g * 255), 0, 255) << 8) |
			clamp(Math.round(b * 255), 0, 255)
		);
	};
	const intToOklch = (int: number): Oklch =>
		toOklch({
			mode: 'rgb',
			r: ((int >> 16) & 0xff) / 255,
			g: ((int >> 8) & 0xff) / 255,
			b: (int & 0xff) / 255
		});

	// 取色板初始颜色：store.color（localStorage 持久化 / 详情页调配设置），唯一事实源，不再读 URL
	let oklch: Oklch = $state(intToOklch(store.color));
	const rgb = $derived(toRgb(oklch));
	const hwb = $derived(toHwb(oklch));

	const hasEyeDropper = $derived('EyeDropper' in window);

	// 展示面板模式（store.source 持久化，刷新/重开按模式直接进入对应面板）：
	// 调色板 / 摄像机（仅 Tauri 应用内可用）/ 油漆（详情页「调配/查看全部」经 store.paintKey 带入）
	const hasCamera = $derived(
		isTauri() && typeof navigator !== 'undefined' && !!navigator.mediaDevices?.getUserMedia
	);
	// 摄像机模式仅 Tauri 内可用：持久化的 camera 模式在无相机环境降级为调色板
	$effect(() => {
		if (store.source === 'camera' && !untrack(() => hasCamera)) {
			store.source = 'palette';
		}
	});

	// 设备横屏（横屏时摄像机全屏覆盖右侧内容区）
	let isLandscape = $state(false);
	$effect(() => {
		const mq = window.matchMedia('(orientation: landscape)');
		isLandscape = mq.matches;
		const onChange = () => (isLandscape = mq.matches);
		mq.addEventListener('change', onChange);
		return () => mq.removeEventListener('change', onChange);
	});

	const eyedrop = () => {
		let EyeDropper = (window as any).EyeDropper;
		const eyeDropper = new EyeDropper();
		eyeDropper.open().then((result: any) => {
			const hex = parseInt(result.sRGBHex.slice(1), 16);
			const b = (hex & 0xff) / 255;
			const g = ((hex >> 8) & 0xff) / 255;
			const r = ((hex >> 16) & 0xff) / 255;
			oklch = toOklch({ mode: 'rgb', r, g, b });
		});
	};

	const catalog = getCatalog();

	// 油漆来源：store.paintKey（详情页「调配/查看全部」设置），唯一指定当前油漆
	const paintColor = $derived.by(() => {
		const key = store.paintKey;
		return key ? getPaintById(key) : null;
	});

	// 油漆信息标签（溶剂/漆面位拆解，与 PaintDetail 共享逻辑）
	// 油漆模式只读色值展示（跟随 store.color，切换 palette 后返回 paint 仍显示油漆色）
	const hexText = $derived(`#${store.color.toString(16).padStart(6, '0').toLowerCase()}`);
	// paint 面板搜索态：无油漆锚点（待选择）时直接显示搜索框
	let searching = $state(!store.paintKey);

	// 切回油漆模式时颜色复位为油漆色（palette 调色后再切回，颜色回到油漆原始色）
	$effect(() => {
		if (store.source === 'paint' && paintColor) {
			oklch = intToOklch(paintColor.rgb);
		}
	});

	// 打开油漆详情：打开单层覆盖层（不离开本页，关闭后回到搜索结果）
	const openDetail = (brand: string, code: string) => {
		const paint = getPaintById(`${brand}:${code}`);
		if (paint) {
			drawer.open({
				key: paint.id,
				component: PaintDetail,
				props: { paint, isStockPage: false }
			});
		}
	};

	/** 混合比例渐变条：按各组分比例生成横向 linear-gradient（色卡下方细细的一条） */
	const mixGradient = (portions: { t: number; rgb: [number, number, number] }[]): string => {
		const sum = portions.reduce((s, p) => s + p.t, 0) || 1;
		let acc = 0;
		const stops: string[] = [];
		for (const p of portions) {
			const start = (acc / sum) * 100;
			acc += p.t;
			const end = (acc / sum) * 100;
			const c = floatRgbToCss(p.rgb);
			stops.push(`${c} ${start}%`, `${c} ${end}%`);
		}
		return `linear-gradient(to right, ${stops.join(', ')})`;
	};

	const serieKey = (brand: string, serie: string) => `${brand}::${serie}`;

	let activeFilterBrand: string | null = $state(null);
	let seriesOpen = $state(false);

	// 品牌 chip 条横向滚动的渐隐遮罩状态（仅手机端显示）
	let brandStrip = $state<HTMLElement | null>(null);
	let stripLeft = $state(false);
	let stripRight = $state(true);

	const updateStrip = () => {
		const el = brandStrip;
		if (!el) return;
		stripLeft = el.scrollLeft > 4;
		stripRight = el.scrollLeft + el.clientWidth < el.scrollWidth - 4;
	};

	$effect(() => {
		const el = brandStrip;
		if (!el) return;
		// 打开时 DOM 可能尚未完成布局（scrollWidth 为 0），用 rAF 延迟到下一帧再算
		const tick = () => requestAnimationFrame(updateStrip);
		tick();
		// chip 尺寸/容器尺寸变化（如字体加载）时重算遮罩状态
		const ro = new ResizeObserver(tick);
		ro.observe(el);
		for (const child of el.children) ro.observe(child);
		el.addEventListener('scroll', updateStrip, { passive: true });
		window.addEventListener('resize', tick);
		return () => {
			el.removeEventListener('scroll', updateStrip);
			ro.disconnect();
			window.removeEventListener('resize', tick);
		};
	});

	// 打开系列筛选时未选中任何品牌则默认选中第一个，避免面板空白
	$effect(() => {
		if (seriesOpen && !activeFilterBrand) {
			const first = Object.keys(catalog)[0];
			if (first) activeFilterBrand = first;
		}
	});

	const toggleSerie = (brand: string, serie: string) => {
		const key = serieKey(brand, serie);
		const next = new Set(store.selectedSeries);
		if (next.has(key)) next.delete(key);
		else next.add(key);
		store.selectedSeries = next;
	};

	const isBrandFullySelected = (brand: string) =>
		Object.keys(catalog[brand]).every((s) => store.selectedSeries.has(serieKey(brand, s)));

	const selectedCountInBrand = (brand: string) =>
		Object.keys(catalog[brand]).filter((s) => store.selectedSeries.has(serieKey(brand, s))).length;

	const toggleBrandAll = (brand: string) => {
		const on = !isBrandFullySelected(brand);
		const next = new Set(store.selectedSeries);
		for (const s of Object.keys(catalog[brand])) {
			const key = serieKey(brand, s);
			if (on) next.add(key);
			else next.delete(key);
		}
		store.selectedSeries = next;
	};

	const isDefaultFilter = $derived(
		store.selectedSeries.size == 0 &&
			store.surfaceTypes.length == 0 &&
			store.baseTypes.length == 0 &&
			!store.searchScope &&
			store.mixingLimit == 0
	);

	const resetFilter = () => {
		store.reset();
	};

	// search 的薄封装，保持原有接口。底层走通用 wasm RPC 客户端：
	// 新请求会取消（terminate）仍在执行的旧请求。被取消的请求这里静默返回空数组，
	// 把取色板当前颜色写入模块级运行时状态并触发搜索；maybeSearch 内部比较
	// (color, filter, stock) 签名，无变化时跳过（切页回来不会重新搜索）。
	// ---- 数据流：store.color 是唯一事实源 ----
	// 色板 oklch → store.color（用户拖色板/取色）：单向。
	// Picker 拖拽期间绝不反向写 oklch（否则浮点往返造成滑条往复跳变），
	// 用 lastColor 记录已同步值，避免对 store.color 建立依赖（外部写入不被拉回）。
	let lastColor = store.color;
	// Picker 引起的 store.color 更新标记：当帧内 effect B 不反向同步 oklch
	let fromPicker = false;
	$effect(() => {
		oklch;
		const int = oklchToInt(oklch);
		if (Math.abs(int - lastColor) > 1) {
			lastColor = int;
			fromPicker = true;
			store.color = int;
			queueMicrotask(() => (fromPicker = false));
		}
	});

	// store.color → 持久化 + 触发搜索 + 色板跟随（仅外部来源更新时同步 oklch）
	let lastSyncedColor = store.color;
	$effect(() => {
		const c = store.color;
		if (!fromPicker) oklch = intToOklch(c);
		// 非油漆渠道（调色板/相机/滴管）更新了颜色 → 解除油漆锚点：
		// paint 面板退回「仅搜索栏」状态（油漆卡片不显示）；untrack 使本 effect 仍只依赖 color，
		// 且仅在颜色实际变化时执行（组件首次运行不清理）。
		if (c !== lastSyncedColor) {
			lastSyncedColor = c;
			if (untrack(() => store.source) !== 'paint') store.paintKey = null;
		}
		store.persist();
		rt.search();
	});

	// 展示面板模式（source/paintKey）变化 → 持久化，刷新/重开后按模式直接进入对应面板
	$effect(() => {
		store.source;
		store.paintKey;
		store.persist();
	});

	// 筛选条件（系列/漆面/溶剂/范围/混色/显示模式）变化 → 持久化 + 立即重搜
	$effect(() => {
		store.selectedSeries;
		store.surfaceTypes;
		store.baseTypes;
		store.searchScope;
		store.mixingLimit;
		store.model;
		store.persist();
		rt.search();
	});
</script>

{#snippet sourceSwitcher()}
	<div
		class="absolute top-1.5 right-1.5 z-10 flex flex-row overflow-hidden rounded-md bg-black/40 backdrop-blur-sm"
	>
		{#if isTauri()}
			<button
				type="button"
				class="cursor-pointer p-1.5 text-white transition-colors hover:bg-white/15 disabled:cursor-not-allowed disabled:opacity-40 {store.source ===
				'camera'
					? 'bg-white/25'
					: ''}"
				title={t('search.sourceCamera')}
				onclick={() => (store.source = 'camera')}
				disabled={!hasCamera}
			>
				<Camera class="size-4" />
			</button>
			<div class="w-px bg-white/25"></div>
		{/if}
		<button
			type="button"
			class="cursor-pointer p-1.5 text-white transition-colors hover:bg-white/15 {store.source ===
			'palette'
				? 'bg-white/25'
				: ''}"
			title={t('search.sourcePalette')}
			onclick={() => (store.source = 'palette')}
		>
			<Palette class="size-4" />
		</button>
		<div class="w-px bg-white/25"></div>
		<button
			type="button"
			class="cursor-pointer p-1.5 text-white transition-colors hover:bg-white/15 {store.source ===
			'paint'
				? 'bg-white/25'
				: ''}"
			title={t('search.sourcePaint')}
			onclick={() => (store.source = 'paint')}
		>
			<Droplet class="size-4" />
		</button>
	</div>
{/snippet}

{#snippet colorSwatch()}
	<div
		class="relative h-24 overflow-hidden rounded-xl border border-gray-700 bg-(--picker-color-srgb)"
	>
		{#if store.source === 'paint' && paintColor}
			<img
				src="/brands/{paintColor.brand}.png"
				alt=""
				class="absolute top-1.5 left-1.5 h-8 w-8 object-contain drop-shadow"
			/>
		{/if}
		{@render sourceSwitcher()}
		{#if store.source === 'palette'}
			{#if !isTauri() && hasEyeDropper}
				<button
					type="button"
					class="absolute right-1.5 bottom-1.5 cursor-pointer rounded-md bg-black/40 p-1.5 text-white backdrop-blur-sm transition-colors hover:bg-black/60"
					onclick={eyedrop}
				>
					<Pipette size="1rem" />
				</button>
			{/if}
		{/if}
	</div>
{/snippet}

{#snippet paintPanel(p: PaintInfo | null)}
	{#if p}
		<div class="relative">
			<div class="rounded-xl border border-gray-200 p-3 sm:p-2 dark:border-gray-700">
				<div class="flex items-start gap-3">
					<div class="min-w-0 flex-1">
						<div class="flex min-w-0 items-baseline gap-2">
							<span class="shrink-0 text-2xl font-bold">{p.code}</span>
							<PanText class="text-lg font-bold text-gray-500 dark:text-gray-400">
								{paintDesc(p)}
							</PanText>
						</div>
						<PanText class="mt-0.5 text-xs text-gray-400">
							{getSerieMeta(p.brand, p.serie)?.name ?? p.serie} /
							{getBrandMeta(p.brand)?.name ?? p.brand}
						</PanText>
						{#if baseLabels(p).length > 0 || surfaceLabels(p).length > 0}
							<div class="mt-2 flex flex-wrap gap-1.5">
								{#each baseLabels(p) as l}
									<span
										class="flex items-center gap-1 rounded-md bg-gray-100 px-2 py-0.5 text-xs text-gray-600 dark:bg-gray-800 dark:text-gray-300"
									>
										<FlaskConical class="size-3" />
										{l}
									</span>
								{/each}
								{#each surfaceLabels(p) as l}
									<span
										class="rounded-md bg-gray-100 px-2 py-0.5 text-xs text-gray-600 dark:bg-gray-800 dark:text-gray-300"
									>
										{l}
									</span>
								{/each}
							</div>
						{/if}
					</div>

					<!-- 右侧：只读 hex + 切换按钮 -->
					<div class="flex shrink-0 flex-col items-end gap-2">
						<ColorCode
							re="^#([0-9a-fA-F]{6})$"
							text={hexText}
							readonly
							class="w-24"
							inputClass="text-left dark:bg-gray-800"
						/>
						<button
							type="button"
							class="flex h-9 w-9 cursor-pointer items-center justify-center rounded-full text-gray-500 transition-colors hover:bg-gray-100 hover:text-gray-700 dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-gray-200 {searching
								? 'bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-white'
								: ''}"
							title={t('gamut.change')}
							onclick={() => (searching = !searching)}
						>
							<ArrowLeftRight class="h-4 w-4" />
						</button>
					</div>
				</div>
			</div>

			{#if searching}
				<!-- 从卡片下方弹出的搜索框（文档流插入，选中/取消自动收回） -->
				<div class="mt-2">
					<PaintSearch
						onselect={(p2) => {
							store.paintKey = p2.id;
							searching = false;
						}}
						oncancel={() => (searching = false)}
					/>
				</div>
			{/if}
		</div>
	{:else}
		<!-- 无油漆锚点：直接显示搜索框 -->
		<PaintSearch
			onselect={(p2) => {
				store.paintKey = p2.id;
				searching = false;
			}}
		/>
	{/if}
{/snippet}

{#snippet colorPicker()}
	{#if store.source === 'paint'}
		<div class="grid gap-0 sm:auto-cols-[150px_1fr] sm:grid-flow-col sm:gap-3">
			{#if isSm()}
				{@render colorSwatch()}
			{/if}
			<div class="min-w-45 sm:max-w-135">
				{@render paintPanel(paintColor)}
			</div>
		</div>
	{:else}
		{@const Picker = [Hsl, Rgb][store.model]}

		<div class="grid gap-3 sm:auto-cols-[150px_1fr] sm:grid-flow-col">
			<div>
				{#if isSm()}
					<div class="mb-3">
						{@render colorSwatch()}
					</div>
				{/if}

				{#snippet hsl()}
					<span class="inline-flex items-center gap-1"><Cylinder class="size-4" />HSL</span>
				{/snippet}

				{#snippet rgb()}
					<span class="inline-flex items-center gap-1"><Box class="size-4" />RGB</span>
				{/snippet}

				<Select class="w-full" options={[hsl, rgb]} bind:value={store.model} />
			</div>

			<div class="min-w-45 sm:max-w-135">
				<Picker bind:oklch />
			</div>
		</div>
	{/if}
{/snippet}

{#snippet selectSeries()}
	<Button size="xs" color="alternative" class="relative w-32 cursor-pointer justify-start gap-1">
		{t('search.series')}
		{#if store.selectedSeries.size > 0}
			<Badge
				class="absolute top-1.5 right-7 rounded-full bg-primary-500 pr-1.5 pl-1.5 text-xs dark:bg-primary-500 dark:text-white"
			>
				{store.selectedSeries.size}
			</Badge>
		{:else}
			{t('search.any')}
		{/if}
		<ChevronDown class="ms-auto h-3 w-3" />
	</Button>
	<Dropdown
		bind:isOpen={seriesOpen}
		class="w-136 max-w-[calc(100vw-3rem)] overflow-hidden! p-0"
		placement="bottom-start"
	>
		<div class="flex max-h-[55vh] flex-col overflow-hidden sm:h-96 sm:flex-row">
			<!-- 品牌列表：手机为横向滚动 chip 条，桌面为纵向列表 -->
			<div
				class="relative w-full shrink-0 sm:w-40 sm:border-r sm:border-gray-200 sm:dark:border-gray-700"
			>
				<div
					bind:this={brandStrip}
					onscroll={updateStrip}
					class="mx-1 flex gap-1.5 overflow-x-auto px-2 py-2 sm:mx-0 sm:flex-col sm:gap-0 sm:overflow-y-auto sm:p-0"
				>
					{#each Object.entries(catalog) as [brand, series]}
						{@const selectedCount = selectedCountInBrand(brand)}
						{@const name = getBrandMeta(brand)?.name ?? brand}
						<!-- 手机 chip -->
						<button
							type="button"
							onclick={() => (activeFilterBrand = brand)}
							title={name}
							class="flex shrink-0 cursor-pointer items-center gap-1.5 rounded-full border px-2.5 py-1.5 text-xs whitespace-nowrap sm:hidden {activeFilterBrand ===
							brand
								? 'border-primary-500 bg-primary-50 text-primary-700 ring-1 ring-primary-500 dark:border-primary-600 dark:bg-primary-900/50 dark:text-primary-200'
								: 'border-gray-200 bg-transparent text-gray-900 hover:bg-gray-100 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-400 dark:hover:bg-gray-700'}"
						>
							<img
								src="/brands/{brand}.png"
								alt=""
								class="h-5 w-5 shrink-0 rounded-full bg-white object-cover ring-1 ring-black/10"
							/>
							<span class="max-w-28 truncate">{name}</span>
							{#if selectedCount > 0}
								<Badge
									class="rounded-full bg-primary-500 text-white dark:bg-primary-500 dark:text-white"
								>
									{selectedCount}
								</Badge>
							{/if}
						</button>
						<!-- 桌面行 -->
						<button
							type="button"
							onmouseenter={() => (activeFilterBrand = brand)}
							onclick={() => (activeFilterBrand = brand)}
							title={name}
							class="hidden w-full cursor-pointer items-center gap-2 px-2.5 py-2 text-left text-sm text-gray-700 sm:flex dark:text-gray-200 {activeFilterBrand ===
							brand
								? 'bg-gray-100 dark:bg-gray-600'
								: 'hover:bg-gray-50 dark:hover:bg-gray-800'}"
						>
							<img
								src="/brands/{brand}.png"
								alt=""
								class="h-7 w-7 shrink-0 rounded-full bg-white object-cover ring-1 ring-black/10"
							/>
							<span class="min-w-0 flex-1 truncate">{name}</span>
							{#if selectedCount > 0}
								<Badge
									class="rounded-full bg-primary-500 text-white dark:bg-primary-500 dark:text-white"
								>
									{selectedCount}
								</Badge>
							{/if}
						</button>
					{/each}
				</div>
				{#if stripRight}
					<div
						class="pointer-events-none absolute inset-y-1 right-0 w-6 bg-linear-to-l from-white via-white/70 to-transparent sm:hidden dark:from-gray-700 dark:via-gray-700/70"
					></div>
				{/if}
				{#if stripLeft}
					<div
						class="pointer-events-none absolute inset-y-1 left-0 w-6 bg-linear-to-r from-white via-white/70 to-transparent sm:hidden dark:from-gray-700 dark:via-gray-700/70"
					></div>
				{/if}
			</div>
			<div class="min-h-0 flex-1 overflow-y-auto p-3">
				{#if activeFilterBrand}
					{@const series = catalog[activeFilterBrand]}
					{#if series}
						{@const brand = activeFilterBrand}
						<div class="mb-2 flex items-center justify-between">
							<span class="text-xs text-gray-400"
								>{t('search.seriesCount', { n: Object.keys(series).length })}</span
							>
							<button
								type="button"
								class="text-xs text-primary-500 hover:underline dark:text-primary-400"
								onclick={() => toggleBrandAll(brand)}
							>
								{isBrandFullySelected(brand) ? t('search.resetFilter') : t('search.selectAll')}
							</button>
						</div>
						<div class="grid grid-cols-4 gap-2.5">
							{#each Object.entries(series) as [serie, paints]}
								{@const serieMeta = getSerieMeta(brand, serie)}
								{@const selected = store.selectedSeries.has(serieKey(brand, serie))}
								<div
									role="button"
									tabindex="0"
									onclick={() => toggleSerie(brand, serie)}
									onkeydown={(e) => e.key === 'Enter' && toggleSerie(brand, serie)}
									title={serieMeta?.name}
									class="group relative aspect-square w-full cursor-pointer overflow-hidden rounded-md bg-gray-100 shadow-sm transition-transform hover:scale-105 dark:bg-gray-800 {selected
										? 'ring-[3px] ring-primary-500'
										: 'ring-1 ring-black/10 hover:ring-black/30 dark:ring-white/10 dark:hover:ring-white/30'}"
								>
									<img
										src={serieThumb(brand, serie)}
										alt=""
										class="h-full w-full object-cover"
										onerror={(e) => {
											if (e.currentTarget instanceof HTMLElement) {
												e.currentTarget.style.visibility = 'hidden';
											}
										}}
									/>
									<div
										class="absolute inset-x-0 bottom-0 bg-black/55 px-1 py-0.5 backdrop-blur-[1px]"
									>
										<div class="truncate text-[10px] leading-tight font-semibold text-white">
											{serieMeta?.name ?? serie}
										</div>
										<div class="truncate text-[9px] leading-tight text-white/75">
											{t('search.paintsCount', { n: paints.length })}
										</div>
									</div>
								</div>
							{/each}
						</div>
					{/if}
				{:else}
					<div class="flex h-full items-center justify-center text-center text-xs text-gray-400">
						{@html t('search.hoverBrandHint')}
					</div>
				{/if}
			</div>
		</div>
	</Dropdown>
{/snippet}

<div
	class="color-provider relative flex h-full flex-col {store.source === 'camera' && isLandscape
		? 'overflow-hidden'
		: 'overflow-y-auto'}"
	style="--slider-thumb-l: {oklch.l};
    --slider-thumb-c: {oklch.c};
    --slider-thumb-h: {oklch.h ?? 0};
    --slider-thumb-hue: {hwb.h ?? 0};
    --picker-color-srgb: rgb({rgb.r * 255} {rgb.g * 255} {rgb.b * 255});"
>
	{#if store.source === 'camera'}
		{#if isLandscape}
			<!-- 横屏：摄像机无圆角全屏覆盖右侧内容区，锁定滚动，挡住所有内容 -->
			<div class="fixed inset-y-0 right-0 left-0 z-40 flex flex-col bg-black sm:left-16">
				<CameraPicker
					fill
					onsample={(r, g, b) => {
						oklch = toOklch({ mode: 'rgb', r, g, b });
						store.source = 'palette';
					}}
				/>
				{@render sourceSwitcher()}
			</div>
		{:else}
			<div
				class="sticky top-0 z-20 bg-white px-6 pt-4 pb-4 shadow-sm dark:bg-gray-900 dark:shadow-black/30"
			>
				<div class="relative overflow-hidden rounded-xl border border-gray-700">
					<CameraPicker
						onsample={(r, g, b) => {
							oklch = toOklch({ mode: 'rgb', r, g, b });
							store.source = 'palette';
						}}
					/>
					{@render sourceSwitcher()}
				</div>
			</div>
		{/if}
	{:else}
		{#if !isSm()}
			<div
				class="sticky top-0 z-20 bg-white px-6 pt-4 pb-2 shadow-sm dark:bg-gray-900 dark:shadow-black/30"
			>
				{@render colorSwatch()}
			</div>
			<div class="px-6 pt-2 pb-4">
				{@render colorPicker()}
			</div>
		{:else}
			<div class="w-full max-w-360 px-6 py-4">
				{@render colorPicker()}
			</div>
		{/if}
	{/if}

	<!-- sm+：搜索区域全宽（不套 max-w，宽屏下比顶部区域宽甚至溢出）；<sm：px-6 原样 -->
	<div class={isSm() ? 'mx-auto w-full px-6' : 'px-6'}>
		<div class="flex flex-row gap-2 border-y border-gray-200 py-2 dark:border-gray-700">
			<div class="flex flex-auto flex-wrap items-center gap-2">
				{@render selectSeries()}

				<MultiSelect
					tooltip={t('search.surfaceTooltip')}
					class="w-48 text-xs"
					options={{
						G: t('search.surface.G'),
						SG: t('search.surface.SG'),
						M: t('search.surface.M'),
						ME: t('search.surface.ME'),
						C: t('search.surface.C'),
						PA: t('search.surface.PA'),
						FL: t('search.surface.FL'),
						W: t('search.surface.W')
					}}
					title={t('search.surfaceTitle')}
					bind:value={store.surfaceTypes}
				/>

				<MultiSelect
					tooltip={t('search.baseTooltip')}
					class="w-28 text-xs"
					options={{
						0: t('search.base.0'),
						1: t('search.base.1'),
						2: t('search.base.2'),
						3: t('search.base.3')
					}}
					title={t('search.baseTitle')}
					bind:value={store.baseTypes}
				/>

				<Select
					tooltip={t('search.scopeTooltip')}
					class="w-28 text-xs"
					options={[t('search.market'), t('search.myStock')]}
					bind:value={store.searchScope}
				/>

				<Select
					tooltip={t('search.mixTooltip')}
					class="w-28 text-xs"
					options={[t('search.mixOff'), t('search.mix1'), t('search.mix2')]}
					bind:value={store.mixingLimit}
					disabled={store.searchScope != 1}
					disabledValue={0}
					disabledTooltip={t('search.mixScopeRequired')}
				/>

				{#if !isDefaultFilter}
					<button
						type="button"
						class="text-xs whitespace-nowrap text-primary-500 hover:underline dark:text-primary-400"
						onclick={() => {
							resetFilter();
						}}
					>
						{t('search.resetFilter')}
					</button>
				{/if}
			</div>
		</div>

		<div class="mt-4 pb-4">
			<h3 class="mb-2 text-sm font-semibold">
				{t('search.results', { n: rt.results.length })}
			</h3>
			<div
				class="grid gap-3 {isSm()
					? 'grid-cols-[repeat(auto-fill,minmax(150px,220px))]'
					: 'grid-cols-[repeat(auto-fill,minmax(150px,1fr))]'}"
			>
				{#if rt.searching}
					{#each Array(8) as _}
						<div
							class="animate-pulse overflow-hidden rounded-lg border border-gray-200 dark:border-gray-700"
						>
							<div class="h-16 w-full bg-gray-200 dark:bg-gray-700"></div>
							<div class="space-y-1.5 p-2">
								<div class="h-2.5 w-3/4 rounded bg-gray-200 dark:bg-gray-700"></div>
								<div class="h-2 w-1/2 rounded bg-gray-200 dark:bg-gray-700"></div>
							</div>
						</div>
					{/each}
				{:else}
					{#each rt.results as r, i (i)}
						{@const isMix = r.portions.length > 1}
						<div
							class="flex flex-col overflow-hidden rounded-lg border border-gray-200 dark:border-gray-700"
						>
							<div class="h-16 w-full" style="background-color: {floatRgbToCss(r.rgb)}"></div>
							{#if isMix}
								<div
									class="h-1.5 w-full border-t border-gray-200 dark:border-gray-700/50"
									style="background: {mixGradient(r.portions)}"
								></div>
							{/if}
							<div class="flex flex-1 flex-col p-2">
								<div class="flex flex-col gap-1">
									{#each r.portions as p}
										<button
											type="button"
											onclick={() => openDetail(p.brand, p.code)}
											class="flex w-full cursor-pointer items-center gap-1.5 rounded-sm text-left text-[11px] hover:bg-gray-50 dark:hover:bg-gray-800"
										>
											<span
												class="h-4 w-4 shrink-0 rounded-sm ring-1 ring-black/10 dark:ring-white/10"
												style="background-color: {floatRgbToCss(p.rgb)}"
											></span>
											<span class="min-w-0 flex-1 truncate font-medium uppercase"
												>{p.brand}/{p.code}</span
											>
											{#if isMix}
												<span
													class="shrink-0 rounded-sm bg-gray-100 px-1.5 py-0.5 font-medium text-primary-700 dark:bg-gray-700 dark:text-primary-300"
												>
													{(p.t * 100).toFixed(0)}%
												</span>
											{/if}
										</button>
									{/each}
								</div>
								{#if !isMix}
									<div class="mt-0.5 pl-5.5 text-[10px] text-gray-500 dark:text-gray-400">
										<PanText>
											{paintDesc(r.portions[0])}
										</PanText>
									</div>
								{/if}
								<div
									class="mt-auto flex items-center justify-between pt-1.5 text-[10px] text-gray-400"
								>
									<span>ΔE {r.delta_e.toFixed(2)}</span>
									<span>{t('search.similarity', { n: similarity(r.delta_e).toFixed(0) })}</span>
								</div>
							</div>
						</div>
					{:else}
						<!-- no result -->
					{/each}
				{/if}
			</div>
		</div>
	</div>
</div>
