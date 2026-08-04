<script lang="ts">
	import { useMode, modeHwb, modeRgb, modeOklch, modeHsl, type Oklch } from 'culori/fn';

	import Hsl from '$lib/components/Hsl.svelte';
	import Rgb from '$lib/components/Rgb.svelte';
	import { Box, Camera, ChevronDown, Cylinder, Palette, Pipette, Funnel } from '@lucide/svelte';
	import { Badge, Button, Dropdown } from 'flowbite-svelte';
	import DropdownButton from '$lib/components/DropdownButton.svelte';
	import CameraPicker from '$lib/components/CameraPicker.svelte';
	import { listPaints, getCatalog, paintId, floatRgbToCss, type SearchResult } from '$lib/paints';
	import { searchAsync } from '$lib/searchClient';
	import { stock } from '$lib/stock.svelte';
	import { getBrandMeta, getSerieMeta, serieThumb } from '$lib/meta';
	import { clamp, similarity, isTauri } from '$lib/utils';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import MultiSelect from '$lib/components/MultiSelect.svelte';
	import Select from '$lib/components/Select.svelte';
	import { searchFilters } from '$lib/searchFilters.svelte';
	import { t } from '$lib/i18n.svelte';

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
	const canCamera = $derived(
		isTauri && typeof navigator !== 'undefined' && !!navigator.mediaDevices?.getUserMedia
	);

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

	const toggleSerie = (brand: string, serie: string) => {
		const key = serieKey(brand, serie);
		const next = new Set(searchFilters.selectedSeries);
		if (next.has(key)) next.delete(key);
		else next.add(key);
		searchFilters.selectedSeries = next;
	};

	const isBrandFullySelected = (brand: string) =>
		Object.keys(catalog[brand]).every((s) => searchFilters.selectedSeries.has(serieKey(brand, s)));

	const selectedCountInBrand = (brand: string) =>
		Object.keys(catalog[brand]).filter((s) => searchFilters.selectedSeries.has(serieKey(brand, s)))
			.length;

	const toggleBrandAll = (brand: string) => {
		const on = !isBrandFullySelected(brand);
		const next = new Set(searchFilters.selectedSeries);
		for (const s of Object.keys(catalog[brand])) {
			const key = serieKey(brand, s);
			if (on) next.add(key);
			else next.delete(key);
		}
		searchFilters.selectedSeries = next;
	};

	const isDefaultFilter = $derived(
		searchFilters.selectedSeries.size == 0 &&
			searchFilters.surfaceTypes.length == 0 &&
			searchFilters.baseTypes.length == 0 &&
			!searchFilters.searchScope &&
			searchFilters.mixingLimit == 0
	);

	const resetFilter = () => {
		searchFilters.reset();
	};

	let results: SearchResult[] = $state([]);
	let searching = $state(false);
	let searchSeq = 0;

	$effect(() => {
		const targetRgb = rgbInt;
		const seq = ++searchSeq;
		searching = true;

		const series = [...searchFilters.selectedSeries].map((key) => {
			const [brand, serie] = key.split('::');
			return [brand, serie];
		});
		const all =
			// FIXME: optimize performance
			searchFilters.searchScope == 0
				? undefined
				: allPaints.filter((p) => stock.has(paintId(p))).map((p) => p.index);
		const opts = {
			series,
			all,
			surfaces: [...searchFilters.surfaceTypes] as string[],
			bases: searchFilters.baseTypes.map((x) => +x),
			mix: searchFilters.mixingLimit,
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
		searchFilters.selectedSeries;
		searchFilters.surfaceTypes;
		searchFilters.baseTypes;
		searchFilters.searchScope;
		searchFilters.mixingLimit;
		searchFilters.model;
		searchFilters.persist();
	});
</script>

{#snippet srcPalette()}
	<Palette class="size-4" />
	{t('search.sourcePalette')}
{/snippet}

{#snippet srcCamera()}
	<Camera class="size-4" />
	{t('search.sourceCamera')}
{/snippet}

{#snippet srcBtn()}
	{#if source === 'camera'}
		<Camera class="size-4" />
	{:else}
		<Palette class="size-4" />
	{/if}
{/snippet}

{#snippet sourceSwitcher()}
	{#if isTauri}
		<div class="absolute top-1.5 right-1.5 z-10">
			<DropdownButton
				buttonClass="cursor-pointer rounded-md !border-transparent !bg-black/40 !p-1.5 !text-white backdrop-blur-sm transition-colors hover:!bg-black/60"
				placement="bottom-end"
				options={[
					{ children: srcPalette, onclick: () => (source = 'palette') },
					{
						children: srcCamera,
						onclick: () => (source = 'camera'),
						disabled: !canCamera
					}
				]}
			>
				{@render srcBtn()}
			</DropdownButton>
		</div>
	{/if}
{/snippet}

{#snippet colorPicker()}
	{@const Picker = [Hsl, Rgb][searchFilters.model]}

	<div>
		<div class="relative mb-3">
			<div
				class="relative h-24 overflow-hidden rounded-xl border border-gray-700 bg-(--picker-color-srgb)"
			>
				{#if hasEyeDropper}
					<button
						type="button"
						class="cursor-pointer absolute right-1.5 bottom-1.5 rounded-md bg-black/40 p-1.5 text-white backdrop-blur-sm transition-colors hover:bg-black/60"
						onclick={eyedrop}
					>
						<Pipette size="1rem" />
					</button>
				{/if}
			</div>

			{@render sourceSwitcher()}
		</div>

		{#snippet hsl()}
			<span class="inline-flex items-center gap-1"><Cylinder class="size-4" />HSL</span>
		{/snippet}

		{#snippet rgb()}
			<span class="inline-flex items-center gap-1"><Box class="size-4" />RGB</span>
		{/snippet}

		<Select class="w-full" options={[hsl, rgb]} bind:value={searchFilters.model} />
	</div>

	<div class="min-w-45 sm:max-w-135">
		<Picker bind:oklch />
	</div>
{/snippet}

{#snippet selectSeries()}
	<Button size="xs" color="alternative" class="relative w-32 cursor-pointer justify-start gap-1">
		{t('search.series')}
		{#if searchFilters.selectedSeries.size > 0}
			<Badge
				class="absolute top-1.5 right-7 rounded-full bg-primary-500 pr-1.5 pl-1.5 text-xs dark:bg-primary-500 dark:text-white"
			>
				{searchFilters.selectedSeries.size}
			</Badge>
		{:else}
			{t('search.any')}
		{/if}
		<ChevronDown class="ms-auto h-3 w-3" />
	</Button>
	<Dropdown class="w-136 p-0" placement="bottom-start">
		<div class="flex h-96">
			<div class="w-40 shrink-0 overflow-y-auto border-r border-gray-200 py-1 dark:border-gray-700">
				{#each Object.entries(catalog) as [brand, series]}
					{@const selectedCount = selectedCountInBrand(brand)}
					{@const name = getBrandMeta(brand)?.name ?? brand}
					<button
						type="button"
						onmouseenter={() => (activeFilterBrand = brand)}
						onclick={() => (activeFilterBrand = brand)}
						title={name}
						class="flex w-full cursor-pointer items-center gap-2 px-2.5 py-2 text-left text-sm text-gray-700 dark:text-gray-200 {activeFilterBrand ===
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
			<div class="flex-1 overflow-y-auto p-3">
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
								{isBrandFullySelected(brand) ? t('search.cancelAll') : t('search.selectAll')}
							</button>
						</div>
						<div class="grid grid-cols-4 gap-2.5">
							{#each Object.entries(series) as [serie, paints]}
								{@const serieMeta = getSerieMeta(brand, serie)}
								{@const selected = searchFilters.selectedSeries.has(serieKey(brand, serie))}
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

<div class="flex h-full flex-col overflow-y-auto px-6 py-4">
	{#if source === 'camera'}
		<div class="relative">
			<CameraPicker
				onsample={(r, g, b) => {
					oklch = toOklch({ mode: 'rgb', r, g, b });
					source = 'palette';
				}}
			/>
			{@render sourceSwitcher()}
		</div>
	{:else}
		<div
			class="color-picker-root grid gap-3 sm:auto-cols-[125px_1fr] sm:grid-flow-col"
			style="
    --slider-thumb-l: {oklch.l};
    --slider-thumb-c: {oklch.c};
    --slider-thumb-h: {oklch.h ?? 0};
    --slider-thumb-hue: {hwb.h ?? 0};
    --picker-color-srgb: rgb({rgb.r * 255} {rgb.g * 255} {rgb.b * 255});"
		>
			{@render colorPicker()}
		</div>
	{/if}

	<div class="mt-4 flex flex-row gap-2 border-y border-gray-200 py-2 dark:border-gray-700">
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
				bind:value={searchFilters.surfaceTypes}
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
				bind:value={searchFilters.baseTypes}
			/>

			<Select
				tooltip={t('search.scopeTooltip')}
				class="w-28 text-xs"
				options={[t('search.market'), t('search.myStock')]}
				bind:value={searchFilters.searchScope}
			/>

			<Select
				tooltip={t('search.mixTooltip')}
				class="w-28 text-xs"
				options={[t('search.mixOff'), t('search.mix1'), t('search.mix2')]}
				bind:value={searchFilters.mixingLimit}
				disabled={searchFilters.searchScope != 1}
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
