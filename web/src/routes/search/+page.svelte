<script lang="ts">
	import { useMode, modeHwb, modeRgb, modeOklch, modeHsl, type Oklch } from 'culori/fn';

	import Hsl from '$lib/components/Hsl.svelte';
	import Rgb from '$lib/components/Rgb.svelte';
	import { Box, Camera, ChevronDown, Cylinder, Palette, Pipette, Funnel } from '@lucide/svelte';
	import { Badge, Button, Dropdown } from 'flowbite-svelte';
	import CameraPicker from '$lib/components/CameraPicker.svelte';
	import {
		listPaints,
		getCatalog,
		paintId,
		floatRgbToCss,
		SURFACE_BITS,
		type SearchResult,
		type FilterOptions
	} from '$lib/paints.svelte';
	import { stock } from '$lib/stock.svelte';
	import { getBrandMeta, getSerieMeta, serieThumb } from '$lib/meta';
	import { clamp, similarity, isSm } from '$lib/utils.svelte';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import MultiSelect from '$lib/components/MultiSelect.svelte';
	import Select from '$lib/components/Select.svelte';
	import { search } from './search.svelte';
	import { t } from '$lib/i18n.svelte';
	import { isTauri } from '@tauri-apps/api/core';
	import { callWasm, WorkerCancelled } from '$lib/wasmClient';

	useMode(modeHsl);
	const toHwb = useMode(modeHwb);
	const toRgb = useMode(modeRgb);
	const toOklch = useMode(modeOklch);

	let oklch: Oklch = $state(toOklch({ mode: 'hsl', h: 189, s: 0.797, l: 0.465 }));
	const hwb = $derived(toHwb(oklch));
	const rgb = $derived(toRgb(oklch));

	// 取色板初始颜色优先级：URL的?color=参数 > localStorage里存的上次颜色 > 默认黑色
	const LAST_COLOR_KEY = 'paintbox:lastColor';
	const initialColorParam =
		page.url.searchParams.get('color') ?? localStorage.getItem(LAST_COLOR_KEY);
	if (initialColorParam && /^[0-9a-fA-F]{6}$/.test(initialColorParam)) {
		const hex = parseInt(initialColorParam, 16);
		const r = ((hex >> 16) & 0xff) / 255;
		const g = ((hex >> 8) & 0xff) / 255;
		const b = (hex & 0xff) / 255;
		oklch = toOklch({ mode: 'rgb', r, g, b });
	}

	const hasEyeDropper = $derived('EyeDropper' in window);

	// 取色源：调色板 / 摄像机（仅 Tauri 应用内可用）
	let source: 'palette' | 'camera' = $state('palette');
	const hasCamera = $derived(
		isTauri() && typeof navigator !== 'undefined' && !!navigator.mediaDevices?.getUserMedia
	);

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

	const rgbInt = $derived.by(() => {
		const r = clamp(Math.round(rgb.r * 255), 0, 255);
		const g = clamp(Math.round(rgb.g * 255), 0, 255);
		const b = clamp(Math.round(rgb.b * 255), 0, 255);
		return (r << 16) | (g << 8) | b;
	});

	const allPaints = listPaints();
	const catalog = getCatalog(allPaints);
	const paintByKey = new Map(allPaints.map((p) => [`${p.brand}:${p.code}`, p]));
	const stockLink = (brand: string, code: string) => {
		const paint = paintByKey.get(`${brand}:${code}`);
		const params = new URLSearchParams({ brand });
		if (paint) params.set('serie', paint.serie);
		params.set('code', code);
		return `/stock?${params.toString()}`;
	};

	$effect(() => {
		const hex = rgbInt.toString(16).padStart(6, '0');
		const handle = setTimeout(() => {
			localStorage.setItem(LAST_COLOR_KEY, hex);
			const url = new URL(page.url);
			if (url.searchParams.get('color') !== hex) {
				url.searchParams.set('color', hex);
				goto(url, { replaceState: true, keepFocus: true, noScroll: true });
			}
		}, 300);
		return () => clearTimeout(handle);
	});

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
		const next = new Set(search.selectedSeries);
		if (next.has(key)) next.delete(key);
		else next.add(key);
		search.selectedSeries = next;
	};

	const isBrandFullySelected = (brand: string) =>
		Object.keys(catalog[brand]).every((s) => search.selectedSeries.has(serieKey(brand, s)));

	const selectedCountInBrand = (brand: string) =>
		Object.keys(catalog[brand]).filter((s) => search.selectedSeries.has(serieKey(brand, s))).length;

	const toggleBrandAll = (brand: string) => {
		const on = !isBrandFullySelected(brand);
		const next = new Set(search.selectedSeries);
		for (const s of Object.keys(catalog[brand])) {
			const key = serieKey(brand, s);
			if (on) next.add(key);
			else next.delete(key);
		}
		search.selectedSeries = next;
	};

	const isDefaultFilter = $derived(
		search.selectedSeries.size == 0 &&
			search.surfaceTypes.length == 0 &&
			search.baseTypes.length == 0 &&
			!search.searchScope &&
			search.mixingLimit == 0
	);

	const resetFilter = () => {
		search.reset();
	};

	// search 的薄封装，保持原有接口。底层走通用 wasm RPC 客户端：
	// 新请求会取消（terminate）仍在执行的旧请求。被取消的请求这里静默返回空数组，
	// 维持旧行为（旧实现里被取消的 Promise 直接丢弃、不 resolve）。
	const searchAsync = async (rgb: number, opts: FilterOptions): Promise<SearchResult[]> => {
		try {
			const results = await callWasm<SearchResult[] | null>('search', [rgb, opts], {
				cancelInFlight: true
			});
			return results ?? [];
		} catch (err) {
			if (err instanceof WorkerCancelled) {
				// 被更新的请求抢占，返回空，让调用方忽略这次结果
				return [];
			}
			throw err;
		}
	};

	let results: SearchResult[] = $state([]);
	let searching = $state(false);
	let searchSeq = 0;

	$effect(() => {
		const targetRgb = rgbInt;
		const seq = ++searchSeq;
		searching = true;

		const series = [...search.selectedSeries].map((key) => {
			const [brand, serie] = key.split('::');
			return [brand, serie];
		});
		const all =
			// FIXME: optimize performance
			search.searchScope == 0
				? undefined
				: allPaints.filter((p) => stock.has(paintId(p))).map((p) => p.index);
		const opts = {
			series,
			all,
			surfaces: search.surfaceTypes.reduce((m, k) => m | SURFACE_BITS[k], 0),
			bases: search.baseTypes.map((x) => +x),
			mix: search.mixingLimit,
			limit: 12
		};

		const handle = setTimeout(async () => {
			const r = await searchAsync(targetRgb, opts);
			if (seq === searchSeq) {
				results = r;
				searching = false;
			}
		}, 200);
		return () => clearTimeout(handle);
	});

	$effect(() => {
		// track all filter state for persistence
		search.selectedSeries;
		search.surfaceTypes;
		search.baseTypes;
		search.searchScope;
		search.mixingLimit;
		search.model;
		search.persist();
	});
</script>

{#snippet sourceSwitcher()}
	<div
		class="absolute top-1.5 right-1.5 z-10 flex flex-col overflow-hidden rounded-md bg-black/40 backdrop-blur-sm"
	>
		<button
			type="button"
			class="cursor-pointer p-1.5 text-white transition-colors hover:bg-white/15 {source ===
			'palette'
				? 'bg-white/25'
				: ''}"
			title={t('search.sourcePalette')}
			onclick={() => (source = 'palette')}
		>
			<Palette class="size-4" />
		</button>
		<div class="h-px bg-white/25"></div>
		<button
			type="button"
			class="cursor-pointer p-1.5 text-white transition-colors hover:bg-white/15 disabled:cursor-not-allowed disabled:opacity-40 {source ===
			'camera'
				? 'bg-white/25'
				: ''}"
			title={t('search.sourceCamera')}
			onclick={() => (source = 'camera')}
			disabled={!hasCamera}
		>
			<Camera class="size-4" />
		</button>
	</div>
{/snippet}

{#snippet colorSwatch()}
	<div
		class="relative h-24 overflow-hidden rounded-xl border border-gray-700 bg-(--picker-color-srgb)"
	>
		{#if !isTauri()}
			{#if hasEyeDropper}
				<button
					type="button"
					class="absolute right-1.5 bottom-1.5 cursor-pointer rounded-md bg-black/40 p-1.5 text-white backdrop-blur-sm transition-colors hover:bg-black/60"
					onclick={eyedrop}
				>
					<Pipette size="1rem" />
				</button>
			{/if}
		{:else}
			{@render sourceSwitcher()}
		{/if}
	</div>
{/snippet}

{#snippet colorPicker()}
	{@const Picker = [Hsl, Rgb][search.model]}

	<div class="grid gap-3 sm:auto-cols-[125px_1fr] sm:grid-flow-col">
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

			<Select class="w-full" options={[hsl, rgb]} bind:value={search.model} />
		</div>

		<div class="min-w-45 sm:max-w-135">
			<Picker bind:oklch />
		</div>
	</div>
{/snippet}

{#snippet selectSeries()}
	<Button size="xs" color="alternative" class="relative w-32 cursor-pointer justify-start gap-1">
		{t('search.series')}
		{#if search.selectedSeries.size > 0}
			<Badge
				class="absolute top-1.5 right-7 rounded-full bg-primary-500 pr-1.5 pl-1.5 text-xs dark:bg-primary-500 dark:text-white"
			>
				{search.selectedSeries.size}
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
								{@const selected = search.selectedSeries.has(serieKey(brand, serie))}
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
	class="color-provider relative flex h-full flex-col px-6 {source === 'camera' && isLandscape
		? 'overflow-hidden'
		: 'overflow-y-auto'}"
	style="--slider-thumb-l: {oklch.l};
    --slider-thumb-c: {oklch.c};
    --slider-thumb-h: {oklch.h ?? 0};
    --slider-thumb-hue: {hwb.h ?? 0};
    --picker-color-srgb: rgb({rgb.r * 255} {rgb.g * 255} {rgb.b * 255});"
>
	{#if source === 'camera'}
		{#if isLandscape}
			<!-- 横屏：摄像机无圆角全屏覆盖右侧内容区，锁定滚动，挡住所有内容 -->
			<div class="fixed inset-y-0 right-0 left-0 z-40 flex flex-col bg-black sm:left-16">
				<CameraPicker
					fill
					onsample={(r, g, b) => {
						oklch = toOklch({ mode: 'rgb', r, g, b });
						source = 'palette';
					}}
				/>
				{@render sourceSwitcher()}
			</div>
		{:else}
			<div
				class="sticky top-0 z-20 -mx-6 bg-white px-6 pt-4 pb-4 shadow-sm dark:bg-gray-900 dark:shadow-black/30"
			>
				<div class="relative overflow-hidden rounded-xl border border-gray-700">
					<CameraPicker
						onsample={(r, g, b) => {
							oklch = toOklch({ mode: 'rgb', r, g, b });
							source = 'palette';
						}}
					/>
					{@render sourceSwitcher()}
				</div>
			</div>
		{/if}
	{:else}
		{#if !isSm()}
			<div
				class="sticky top-0 z-20 -mx-6 bg-white px-6 pt-4 pb-2 shadow-sm dark:bg-gray-900 dark:shadow-black/30"
			>
				{@render colorSwatch()}
			</div>
			<div class="pt-2 pb-4">
				{@render colorPicker()}
			</div>
		{:else}
			<div class="py-4">
				{@render colorPicker()}
			</div>
		{/if}
	{/if}

	<div class="flex flex-row gap-2 border-y border-gray-200 py-2 dark:border-gray-700">
		<!-- <span
			class="-ml-4 flex items-center gap-1 text-xs whitespace-nowrap text-gray-500 dark:text-gray-400"
		>
			<Funnel class="h-4 w-4" />
		</span> -->

		<div class="flex flex-auto flex-wrap items-center gap-2">
			{@render selectSeries()}

			<MultiSelect
				tooltip={t('search.surfaceTooltip')}
				class="w-36 text-xs"
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
				bind:value={search.surfaceTypes}
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
				bind:value={search.baseTypes}
			/>

			<Select
				tooltip={t('search.scopeTooltip')}
				class="w-28 text-xs"
				options={[t('search.market'), t('search.myStock')]}
				bind:value={search.searchScope}
			/>

			<Select
				tooltip={t('search.mixTooltip')}
				class="w-28 text-xs"
				options={[t('search.mixOff'), t('search.mix1'), t('search.mix2')]}
				bind:value={search.mixingLimit}
				disabled={search.searchScope != 1}
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
		<h3 class="mb-2 text-sm font-semibold">{t('search.results', { n: results.length })}</h3>
		<div class="grid grid-cols-[repeat(auto-fill,minmax(150px,1fr))] gap-3">
			{#if searching}
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
				{#each results as r, i (i)}
					{@const isMix = r.portions.length > 1}
					<div class="overflow-hidden rounded-lg border border-gray-200 dark:border-gray-700">
						<div class="h-16 w-full" style="background-color: {floatRgbToCss(r.rgb)}"></div>
						<div class="p-2">
							<div class="flex flex-col gap-1">
								{#each r.portions as p}
									<a
										href={stockLink(p.brand, p.code)}
										class="flex items-center gap-1.5 rounded-sm text-[11px] hover:bg-gray-50 dark:hover:bg-gray-800"
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
									</a>
								{/each}
							</div>
							{#if !isMix}
								<div class="mt-0.5 truncate pl-5.5 text-[10px] text-gray-500 dark:text-gray-400">
									{r.portions[0].desc}
								</div>
							{/if}
							<div class="mt-1.5 flex items-center justify-between text-[10px] text-gray-400">
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
