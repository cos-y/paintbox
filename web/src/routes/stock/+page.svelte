<script lang="ts">
	import { fade, fly } from 'svelte/transition';
	import { ChevronLeft, Check, Plus, Search, X, ArrowUpDown, Funnel } from '@lucide/svelte';
	import { Card, Button, Badge, Dropdown, DropdownItem } from 'flowbite-svelte';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import {
		listPaints,
		getCatalog,
		paintId,
		rgbToHex,
		searchNearest,
		colorDiff,
		findDirectEquivalences,
		SURFACE_BITS,
		type PaintInfo
	} from '$lib/paints';
	import { stock } from '$lib/stock.svelte';
	import { getBrandMeta, getSerieMeta, serieThumb } from '$lib/meta';
	import { similarity } from '$lib/utils.svelte';
	import { t } from '$lib/i18n.svelte';
	import Select from '$lib/components/Select.svelte';
	import StockFilterRow from '$lib/components/StockFilterRow.svelte';

	const allPaints = listPaints();
	const paintByKey = new Map(allPaints.map((p) => [paintId(p), p]));
	const catalog = getCatalog(allPaints);

	// 用查询参数（?brand=&serie=&code=）驱动状态，而不是纯内部state：
	// 这样浏览时地址栏会实时更新，且任意页面都能通过完整URL直接分享/刷新进入，
	// 同时因为只有查询参数在变、路径始终是同一个/stock，静态文件服务器不需要
	// 为每个品牌/系列/型号单独生成页面文件。
	const selectedBrand = $derived(page.url.searchParams.get('brand') ?? '');
	const selectedSerieParam = $derived(page.url.searchParams.get('serie'));
	const selectedCode = $derived(page.url.searchParams.get('code'));

	const currentBrandGroup = $derived(catalog[selectedBrand] ?? {});
	const selectedSerie = $derived(selectedSerieParam ?? Object.keys(currentBrandGroup)[0] ?? null);
	const currentSerieGroup = $derived(currentBrandGroup[selectedSerie] ?? []);
	const selectedPaint = $derived(
		selectedCode ? (currentSerieGroup.find((p) => p.code === selectedCode) ?? null) : null
	);

	const level = $derived(selectedPaint ? 2 : selectedBrand ? 1 : 0);

	const totalModels = (series: { [key: string]: PaintInfo[] }) =>
		Object.values(series).reduce((n, s) => n + s.length, 0);

	const ownedCountInBrand = (series: { [key: string]: PaintInfo[] }) =>
		Object.values(series).reduce((n, s) => n + ownedCountInSerie(s), 0);

	const ownedCountInSerie = (s: PaintInfo[]) =>
		s.reduce((n, p) => n + (stock.has(paintId(p)) ? 1 : 0), 0);

	const navigateTo = (params: {
		brand?: string | null;
		serie?: string | null;
		code?: string | null;
	}) => {
		const url = new URL(page.url);
		url.search = '';
		if (params.brand) url.searchParams.set('brand', params.brand);
		if (params.serie) url.searchParams.set('serie', params.serie);
		if (params.code) url.searchParams.set('code', params.code);
		goto(url, { replaceState: false, keepFocus: true, noScroll: true });
	};

	const selectBrand = (brand: string) => {
		const serie = Object.keys(catalog[brand] ?? {})[0] ?? null;
		navigateTo({ brand, serie });
	};

	const selectSerie = (serie: string) => {
		if (!selectedBrand) return;
		navigateTo({ brand: selectedBrand, serie });
	};

	const selectPaint = (paint: PaintInfo) => {
		navigateTo({ brand: paint.brand, serie: paint.serie, code: paint.code });
	};

	const goToLevel0 = () => navigateTo({});

	const goToLevel1 = () => {
		if (!selectedBrand) return goToLevel0();
		navigateTo({ brand: selectedBrand, serie: selectedSerie });
	};

	const goBack = () => (level === 2 ? goToLevel1() : goToLevel0());

	// 相近同色漆：按颜色距离查询得到的、颜色相近但名字不一定相关的油漆
	const colorEquivalences = $derived.by(() => {
		if (!selectedPaint) return [];
		const paint = selectedPaint;
		return searchNearest(paint.rgb, { mix: 0, limit: 8 })
			.map((r) => paintByKey.get(paintId(r.portions[0])))
			.filter((p): p is PaintInfo => !!p && !(p.brand === paint.brand && p.code === paint.code));
	});

	// 直接等价：数据来源里的品牌对照表（例如Gunze H9 <-> Gunze C9），名字/型号对应但颜色不一定相近
	const directEquivalences = $derived(
		selectedPaint ? findDirectEquivalences(selectedPaint.index) : []
	);

	// 点击相近同色漆/直接等价里的某个方块时，在原色下方拼接一个对比条（单选，再点一次取消）
	let compareCode = $state<string | null>(null);
	$effect(() => {
		selectedPaint;
		compareCode = null;
	});
	const comparePaint = $derived(compareCode ? (paintByKey.get(compareCode) ?? null) : null);
	const compareDeltaE = $derived(
		selectedPaint && comparePaint ? colorDiff(selectedPaint.rgb, comparePaint.rgb) : null
	);
	const toggleCompare = (p: PaintInfo) => {
		const key = paintId(p);
		compareCode = compareCode === key ? null : key;
	};

	// ---- 排序 / 搜索（品牌内跨系列） ----
	let sortKey = $state(0);
	let sortOpen = $state(false);
	let searchOpen = $state(false);
	let query = $state('');
	// 筛选：漆面 + 溶剂（空数组 = 不限，AND）
	let filterOpen = $state(false);
	let surfSel = $state<string[]>([]);
	let baseSel = $state<string[]>([]);
	const baseFilter = $derived(
		baseSel.length == 0 ? 0x7fffffff : baseSel.reduce((a, b) => a | (1 << +b), 0)
	);
	const surfFilter = $derived(new Set(surfSel.map((k) => SURFACE_BITS[k])));
	const filterCount = $derived(surfSel.length + baseSel.length);
	const passFilter = (p: PaintInfo): boolean => {
		if (surfSel.length > 0 && !surfFilter.has(p.prop)) return false;
		return (baseFilter & p.base) != 0;
	};

	const hslOf = (rgb: number): [number, number, number] => {
		const r = ((rgb >> 16) & 0xff) / 255;
		const g = ((rgb >> 8) & 0xff) / 255;
		const b = (rgb & 0xff) / 255;
		const max = Math.max(r, g, b);
		const min = Math.min(r, g, b);
		const l = (max + min) / 2;
		const d = max - min;
		if (d === 0) return [0, 0, l];
		const s = d / (1 - Math.abs(2 * l - 1));
		let h: number;
		if (max === r) h = ((g - b) / d) % 6;
		else if (max === g) h = (b - r) / d + 2;
		else h = (r - g) / d + 4;
		return [(h * 60 + 360) % 360, s, l];
	};

	const sortPaints = (list: PaintInfo[]): PaintInfo[] => {
		const arr = [...list];
		switch (sortKey) {
			case 1:
				return arr.sort((a, b) => hslOf(a.rgb)[0] - hslOf(b.rgb)[0]);
			case 2:
				return arr.sort((a, b) => hslOf(b.rgb)[1] - hslOf(a.rgb)[1]);
			case 3:
				return arr.sort((a, b) => hslOf(b.rgb)[2] - hslOf(a.rgb)[2]);
			case 4:
				return arr.sort((a, b) => Number(stock.has(paintId(b))) - Number(stock.has(paintId(a))));
			default:
				return arr.sort((a, b) => a.code.localeCompare(b.code, undefined, { numeric: true }));
		}
	};

	const matches = (p: PaintInfo): boolean => {
		const q = query.trim().toLowerCase();
		if (!q) return true;
		return p.code.toLowerCase().includes(q) || (p.desc ?? '').toLowerCase().includes(q);
	};

	const isSearching = $derived(searchOpen && query.trim().length > 0);
	// 筛选激活（无搜索时也启用搜索式联动：左侧栏过滤 + 自动切换系列）
	const isFiltering = $derived(filterCount > 0);
	const isActiveFilter = $derived(isSearching || isFiltering);
	const filterMatch = (p: PaintInfo): boolean => passFilter(p) && (!isSearching || matches(p));
	// 搜索/筛选时左侧栏：仅显示有匹配的系列（品牌内联动）
	const visibleSeries = $derived.by(() => {
		const entries = Object.entries(currentBrandGroup ?? {});
		if (!isActiveFilter) return entries;
		return entries.filter(([, paints]) => paints.some(filterMatch));
	});
	// 搜索/筛选时色卡区：当前选中系列内匹配（左侧导航切换系列）
	const searchSerieResults = $derived.by(() => {
		if (!isActiveFilter) return null;
		return sortPaints(currentSerieGroup.filter(filterMatch));
	});
	// 搜索/筛选激活且当前系列无匹配时，自动切到第一个有匹配的系列
	$effect(() => {
		if (isActiveFilter && visibleSeries.length > 0) {
			const first = visibleSeries.find(([s]) => s === selectedSerie) ?? visibleSeries[0];
			if (first && first[0] !== selectedSerie) selectSerie(first[0]);
		}
	});
	const sortedCurrentSerie = $derived(sortPaints(currentSerieGroup.filter(passFilter)));
</script>

<div class="flex h-full flex-col">
	<div class="shrink-0 border-b border-gray-200 px-4 py-2 dark:border-gray-700">
		<div class="flex min-h-13 items-center gap-2">
			{#if level > 0}
				<button
					type="button"
					onclick={goBack}
					class="cursor-pointer rounded-full p-1 hover:bg-gray-100 dark:hover:bg-gray-700"
				>
					<ChevronLeft class="h-5 w-5" />
				</button>
			{/if}
			<div class="relative min-w-0 flex-1">
				<div
					class="transition-opacity duration-150 {searchOpen && level === 1
						? 'max-sm:pointer-events-none max-sm:opacity-0'
						: 'opacity-100'}"
				>
					<div class="truncate text-xl font-semibold text-gray-900 dark:text-white">
						{level > 0 ? (getBrandMeta(selectedBrand)?.name ?? selectedBrand) : t('stock.brands')}
					</div>
					{#if selectedSerie}
						<nav class="flex items-center gap-1 text-xs text-gray-500 dark:text-gray-400">
							<button type="button" onclick={goToLevel1} class="cursor-pointer hover:underline">
								{getSerieMeta(selectedBrand, selectedSerie)?.name ?? selectedSerie}
							</button>
							{#if selectedPaint}
								<span>/</span>
								<span>{selectedPaint.code}</span>
							{/if}
						</nav>
					{/if}
				</div>
				{#if searchOpen && level === 1}
					<input
						type="search"
						autofocus
						in:fade={{ duration: 150 }}
						out:fade={{ duration: 150 }}
						bind:value={query}
						placeholder={t('stock.searchPlaceholder')}
						class="absolute top-1/2 left-0 h-9 w-full -translate-y-1/2
							rounded-md border border-gray-200 bg-gray-100 px-2 py-1.5 text-sm
							placeholder:text-gray-500 focus:border-primary-500 focus:outline-none
							sm:right-0 sm:left-auto sm:w-2/3
							dark:border-gray-700 dark:bg-gray-800 dark:text-white dark:placeholder:text-gray-400"
					/>
				{/if}
			</div>
			{#if level === 1}
				{#if searchOpen}
					<button
						type="button"
						onclick={() => {
							searchOpen = false;
							query = '';
						}}
						title={t('stock.closeSearch')}
						class="cursor-pointer rounded-full p-1.5 text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-700"
					>
						<X class="h-4 w-4" />
					</button>
				{:else}
					<button
						type="button"
						onclick={() => (searchOpen = true)}
						title={t('stock.searchTitle')}
						class="cursor-pointer rounded-full p-1.5 text-gray-500 hover:bg-gray-100 hover:text-gray-700 dark:hover:bg-gray-700 dark:hover:text-gray-200"
					>
						<Search class="h-4 w-4" />
					</button>
				{/if}
				{@render filterBtn()}
				<Select
					options={[
						t('stock.sortCode'),
						t('stock.sortHue'),
						t('stock.sortSat'),
						t('stock.sortLight'),
						t('stock.sortStock')
					]}
					bind:value={sortKey}
					class="rounded-full border-0 p-1.5
					text-gray-500 ring-0! dark:bg-gray-900 dark:hover:bg-gray-700"
					activeClass="dark:bg-gray-700 dark:text-white"
					lockWidth={false}
					placement="bottom-end"
				>
					<ArrowUpDown class="h-4 w-4" />
				</Select>
			{/if}
		</div>
	</div>

	{#snippet filterBtn()}
		<div class="relative shrink-0">
			<button
				type="button"
				title={t('stock.filterTitle')}
				class="relative cursor-pointer rounded-full p-1.5 text-gray-500 hover:bg-gray-100 hover:text-gray-700 dark:hover:bg-gray-700 dark:hover:text-gray-200 {filterOpen
					? 'bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-white'
					: filterCount > 0
						? 'text-primary-500'
						: ''}"
			>
				<Funnel class="h-4 w-4" />
				{#if filterCount > 0}
					<span
						class="absolute -top-0.5 -right-0.5 flex h-3.5 min-w-3.5 items-center justify-center rounded-full bg-primary-500 px-0.5 text-[9px] font-semibold text-white"
					>
						{filterCount}
					</span>
				{/if}
			</button>
			<Dropdown class="list-none overflow-hidden!" placement="bottom-end" bind:isOpen={filterOpen}>
				<div class="grid min-w-70 grid-cols-2 gap-x-4 p-3">
					<div class="space-y-0.5 border-r border-gray-200 pr-3 dark:border-gray-700">
						<StockFilterRow
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
							bind:values={surfSel}
						>
							{t('stock.surfaceTitle')}
						</StockFilterRow>
					</div>
					<div class="space-y-0.5">
						<StockFilterRow
							options={{
								0: t('search.base.0'),
								1: t('search.base.1'),
								2: t('search.base.2'),
								3: t('search.base.3')
							}}
							bind:values={baseSel}
						>
							{t('stock.baseTitle')}
						</StockFilterRow>
					</div>
				</div>
			</Dropdown>
		</div>
	{/snippet}

	{#snippet labelPrimary(paint: PaintInfo)}
		{@const brandMeta = getBrandMeta(paint.brand)}
		{@const serieMeta = getSerieMeta(paint.brand, paint.serie)}
		<span class=" text-gray-500 dark:text-gray-400">
			<Button
				color="secondary"
				class="inline-block cursor-pointer p-0 text-xs text-gray-500 dark:text-gray-400"
				onclick={() => navigateTo({ brand: paint.brand, serie: paint.serie })}
			>
				{serieMeta?.name ?? paint.serie}
			</Button>
			/
			<Button
				color="secondary"
				class="inline-block cursor-pointer p-0 text-xs text-gray-500 dark:text-gray-400"
				onclick={() => navigateTo({ brand: paint.brand })}
			>
				{brandMeta?.name ?? paint.brand}
			</Button>
		</span>
	{/snippet}

	<div class="flex-1 overflow-hidden">
		{#if level === 0}
			{#key level}
				<div
					class="grid h-full auto-rows-min grid-cols-[repeat(auto-fill,minmax(240px,1fr))] gap-3 overflow-y-auto px-6 py-4"
					in:fly={{ x: -24, duration: 150 }}
				>
					{#each Object.entries(catalog) as [brand, series]}
						<Card
							onclick={() => selectBrand(brand)}
							role="button"
							tabindex={0}
							size="md"
							class="relative cursor-pointer p-3 hover:bg-gray-50 dark:hover:bg-gray-700"
						>
							{@const ownCount = ownedCountInBrand(series)}
							{@const meta = getBrandMeta(brand)}
							{#if ownCount > 0}
								<Badge
									class="absolute top-2 right-2 rounded-full bg-primary-500 text-white dark:bg-primary-500 dark:text-white"
								>
									{ownCount}
								</Badge>
							{/if}
							<div class="flex items-center gap-3">
								<img
									src="/brands/{brand}.png"
									alt={brand}
									class="h-10 w-10 shrink-0 rounded-full bg-white object-cover ring-1 ring-black/10"
								/>
								<div class="min-w-0">
									<div class="truncate font-semibold">{meta?.name ?? brand}</div>
									{#if meta?.desc}
										<div class="truncate text-xs text-gray-500 dark:text-gray-400">{meta.desc}</div>
									{/if}
									<div class="text-[11px] text-gray-400">
										{t('stock.brandStats', {
											series: Object.keys(series).length,
											paints: totalModels(series)
										})}
									</div>
								</div>
							</div>
						</Card>
					{/each}
				</div>
			{/key}
		{:else if level === 1}
			{#key `${level}-${selectedBrand}`}
				<div class="flex h-full" in:fly={{ x: 24, duration: 150 }}>
					<!-- 桌面侧栏：宽栏（图标 + 系列名/描述 + 库存） -->
					<div
						class="hidden w-56 shrink-0 overflow-y-auto border-r border-gray-200 sm:block dark:border-gray-700"
					>
						{#each visibleSeries as [serie, paints]}
							{@const serieMeta = getSerieMeta(selectedBrand, serie)}
							{@const ownCount = ownedCountInSerie(paints)}
							<button
								type="button"
								onclick={() => selectSerie(serie)}
								title={serieMeta?.desc}
								class="flex w-full cursor-pointer items-center gap-2 px-2 py-1.5 text-left text-xs {serie ===
								selectedSerie
									? 'bg-primary-50 font-medium text-primary-700 dark:bg-gray-700 dark:text-white'
									: 'text-gray-700 hover:bg-gray-50 dark:text-gray-300 dark:hover:bg-gray-800'}"
							>
								<img
									src={serieThumb(selectedBrand, serie)}
									alt=""
									class="h-7 w-7 shrink-0 rounded bg-white object-cover ring-1 ring-black/10"
									onerror={(e) => {
										if (e.currentTarget instanceof HTMLElement) {
											e.currentTarget.style.visibility = 'hidden';
										}
									}}
								/>
								<span class="min-w-0 flex-1">
									<span class="block truncate">{serieMeta?.name ?? serie} ({paints.length})</span>
									<span class="block truncate text-[10px] text-gray-400">{serieMeta?.desc}</span>
								</span>
								{#if ownCount > 0}
									<Badge
										class="rounded-full bg-primary-500 text-white dark:bg-primary-500 dark:text-white"
									>
										{ownCount}
									</Badge>
								{/if}
							</button>
						{/each}
					</div>
					<!-- 移动侧栏：极窄栏（第一行 图标 + 库存角标 + 型号总数，第二行 系列名小字） -->
					<div
						class="w-24 shrink-0 overflow-y-auto border-r border-gray-200 sm:hidden dark:border-gray-700"
					>
						{#each visibleSeries as [serie, paints]}
							{@const serieMeta = getSerieMeta(selectedBrand, serie)}
							{@const ownCount = ownedCountInSerie(paints)}
							<button
								type="button"
								onclick={() => selectSerie(serie)}
								title={serieMeta?.desc}
								class="flex w-full cursor-pointer flex-col items-center py-1 pr-2 pl-2 {serie ===
								selectedSerie
									? 'bg-primary-50 dark:bg-gray-700'
									: 'hover:bg-gray-50 dark:hover:bg-gray-800'}"
							>
								<div class="relative aspect-square w-full overflow-hidden rounded-md shadow-sm">
									<img
										src={serieThumb(selectedBrand, serie)}
										alt=""
										class="h-full w-full object-cover"
										onerror={(e) => {
											if (e.currentTarget instanceof HTMLElement) {
												e.currentTarget.style.visibility = 'hidden';
											}
										}}
									/>
									{#if ownCount > 0}
										<Badge
											class="absolute top-0.5 right-0.5 rounded-full bg-primary-500 px-1.5 text-[10px] text-white dark:bg-primary-500 dark:text-white"
										>
											{ownCount}
										</Badge>
									{/if}
									<div
										class="absolute inset-x-0 bottom-0 bg-black/65 px-0.5 py-0.5 backdrop-blur-[1px]"
									>
										<div class="truncate text-[9px] leading-tight font-semibold text-white">
											{serieMeta?.name ?? serie}
										</div>
										<div class="truncate text-[8px] leading-tight text-white/75">
											{t('search.paintsCount', { n: paints.length })}
										</div>
									</div>
								</div>
							</button>
						{/each}
					</div>
					{#snippet paintCard(paint: PaintInfo, showSerie: boolean)}
						{@const inStock = stock.has(paintId(paint))}
						<div
							role="button"
							tabindex="0"
							onclick={() => selectPaint(paint)}
							onkeydown={(e) => e.key === 'Enter' && selectPaint(paint)}
							class="group relative aspect-square w-full cursor-pointer overflow-hidden rounded-md shadow-sm transition-transform hover:scale-105 {inStock
								? 'ring-[3px] ring-primary-500'
								: 'ring-1 ring-black/10 hover:ring-black/30 dark:ring-white/10 dark:hover:ring-white/30'}"
							style="background-color: {rgbToHex(paint.rgb)}"
							title={paint.desc}
						>
							<button
								type="button"
								title={inStock ? t('stock.removeFromStock') : t('stock.addToStock')}
								onclick={(e) => {
									e.stopPropagation();
									stock.toggle(paintId(paint));
									e.currentTarget.blur();
								}}
								class="absolute top-0 right-0 h-6 w-6 scale-75 cursor-pointer text-white opacity-0 transition-all duration-150 group-hover:scale-100 group-hover:opacity-100 focus:scale-100 focus:opacity-100 {inStock
									? 'scale-100 opacity-100'
									: ''}"
							>
								<span
									class="absolute inset-0 [clip-path:polygon(100%_0,0_0,100%_100%)] {inStock
										? 'bg-primary-500'
										: 'bg-black/60 hover:bg-black/75'}"
								></span>
								<span class="absolute top-0.5 right-0.5">
									{#if inStock}
										<Check class="h-2.5 w-2.5" />
									{:else}
										<Plus class="h-2.5 w-2.5" />
									{/if}
								</span>
							</button>
							<div class="absolute inset-x-0 bottom-0 bg-black/55 px-1 py-0.5 backdrop-blur-[1px]">
								<div class="truncate text-[10px] leading-tight font-semibold text-white">
									{paint.code}
								</div>
								<div class="truncate text-[9px] leading-tight text-white/75">
									{showSerie
										? (getSerieMeta(paint.brand, paint.serie)?.name ?? paint.serie)
										: paint.desc}
								</div>
							</div>
						</div>
					{/snippet}
					<div
						class="grid flex-1 auto-rows-min grid-cols-[repeat(auto-fill,minmax(64px,1fr))] gap-2.5 overflow-y-auto p-2"
					>
						{#if searchSerieResults}
							{#each searchSerieResults as paint (paint.code)}
								{@render paintCard(paint, false)}
							{/each}
							{#if searchSerieResults.length === 0}
								<div class="col-span-full p-4 text-center text-xs text-gray-400">
									{t('stock.noResults')}
								</div>
							{/if}
						{:else}
							{#each sortedCurrentSerie as paint (paint.code)}
								{@render paintCard(paint, false)}
							{/each}
						{/if}
					</div>
				</div>
			{/key}
		{:else if level === 2 && selectedPaint}
			{@const paint = selectedPaint}
			{@const inStock = stock.has(paintId(paint))}
			{#key `${level}-${paint.brand}-${paint.code}`}
				<div class="h-full overflow-y-auto p-4" in:fly={{ x: 24, duration: 150 }}>
					<div class="mx-auto max-w-xl space-y-4">
						<div class="flex items-start justify-between gap-3">
							<div class="min-w-0">
								<div>
									<span class="text-4xl font-bold">{paint.code}</span>
								</div>
								<div class="font-bold text-gray-500 dark:text-gray-400">{paint.desc}</div>
							</div>
							<button
								type="button"
								aria-label={inStock ? t('stock.removeFromStock') : t('stock.addToStock')}
								onclick={() => stock.toggle(paintId(paint))}
								class="flex h-9 w-9 shrink-0 cursor-pointer items-center justify-center rounded-full transition-colors {inStock
									? 'bg-primary-500 text-white hover:bg-primary-600'
									: 'bg-gray-100 text-gray-500 hover:bg-gray-200 dark:bg-gray-700 dark:text-gray-300 dark:hover:bg-gray-600'}"
							>
								{#if inStock}
									<Check class="h-5 w-5" />
								{:else}
									<Plus class="h-5 w-5" />
								{/if}
							</button>
						</div>

						<div class="relative h-40 overflow-hidden rounded-lg shadow-inner">
							<div
								class="absolute inset-x-0 top-0 {comparePaint ? 'h-1/2' : 'h-full'}"
								style="background-color: {rgbToHex(paint.rgb)}"
							>
								<img
									src="/brands/{paint.brand}.png"
									alt=""
									class="absolute top-1.5 left-1.5 h-8 w-8 object-contain drop-shadow"
								/>
							</div>
							{#if comparePaint}
								<button
									type="button"
									onclick={() => selectPaint(comparePaint)}
									class="absolute inset-x-0 bottom-0 h-1/2 cursor-pointer"
									style="background-color: {rgbToHex(comparePaint.rgb)}"
								>
									<img
										src="/brands/{comparePaint.brand}.png"
										alt=""
										class="absolute right-1.5 bottom-1.5 h-8 w-8 object-contain drop-shadow"
									/>
								</button>
							{/if}
						</div>

						{#if comparePaint}
							<div class="flex w-full text-left">
								{#if compareDeltaE !== null}
									<div class="flex-1 text-xs text-gray-400">
										{t('stock.similarity', { n: similarity(compareDeltaE).toFixed(0) })}
									</div>
								{/if}
								<div class="text-right">
									<div class="font-bold text-gray-500 dark:text-gray-400">
										{comparePaint.desc}
									</div>
									<div>
										{@render labelPrimary(comparePaint)}
										<span class="text-4xl font-bold">{comparePaint.code}</span>
									</div>
								</div>
							</div>
						{/if}

						<div>
							<h3 class="mb-2 text-sm font-semibold text-gray-500 uppercase dark:text-gray-400">
								{t('stock.directEquiv')}
							</h3>
							<div class="flex flex-wrap gap-2">
								{#each directEquivalences as p (paintId(p))}
									<button
										type="button"
										onclick={() => toggleCompare(p)}
										class="flex items-center gap-2 rounded-lg border px-2 py-1 {compareCode ===
										paintId(p)
											? 'border-primary-500 bg-primary-50 dark:bg-gray-700'
											: 'border-gray-200 hover:bg-gray-50 dark:border-gray-700 dark:hover:bg-gray-800'}"
									>
										<div
											class="h-5 w-5 shrink-0 rounded"
											style="background-color: {rgbToHex(p.rgb)}"
										></div>
										<span class="text-xs uppercase">{p.brand}/{p.code}</span>
									</button>
								{:else}
									<div class="text-xs text-gray-400">{t('stock.noDirectEquiv')}</div>
								{/each}
							</div>
						</div>

						<div>
							<h3 class="mb-2 text-sm font-semibold text-gray-500 uppercase dark:text-gray-400">
								{t('stock.similarColors')}
							</h3>
							<div class="flex flex-wrap gap-2">
								{#each colorEquivalences as p (paintId(p))}
									<button
										type="button"
										onclick={() => toggleCompare(p)}
										class="flex cursor-pointer items-center gap-2 rounded-lg border px-2 py-1 {compareCode ===
										paintId(p)
											? 'border-primary-500 bg-primary-50 dark:bg-gray-700'
											: 'border-gray-200 hover:bg-gray-50 dark:border-gray-700 dark:hover:bg-gray-800'}"
									>
										<div
											class="h-5 w-5 shrink-0 rounded"
											style="background-color: {rgbToHex(p.rgb)}"
										></div>
										<span class="text-xs uppercase">{p.brand}/{p.code}</span>
									</button>
								{:else}
									<div class="text-xs text-gray-400">{t('stock.noSimilar')}</div>
								{/each}
							</div>
						</div>
					</div>
				</div>
			{/key}
		{/if}
	</div>
</div>

<style>
	:global(input[type='search'])::-webkit-search-cancel-button {
		-webkit-appearance: none;
		appearance: none;
	}
</style>
