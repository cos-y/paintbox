<script lang="ts">
	import { fade, fly } from 'svelte/transition';
	import { ChevronLeft, Check, Plus, Search, X, Funnel, ArrowDownWideNarrow } from '@lucide/svelte';
	import { Card, Badge, Dropdown } from 'flowbite-svelte';
	import { getCatalog, rgbToHex, SURFACE_BITS, type PaintInfo } from '$lib/paints.svelte';
	import { stock } from '$lib/stock.svelte';
	import { stockNav } from '$lib/stocknav.svelte';
	import { isSm } from '$lib/utils.svelte';
	import { viewStack } from '$lib/viewstack.svelte';
	import { getBrandMeta, getSerieMeta, serieThumb } from '$lib/meta';
	import PaintDetail from '$lib/components/PaintDetail.svelte';
	import { paintDesc } from '$lib/i18ndyn.svelte';
	import { t } from '$lib/i18n.svelte';
	import Select from '$lib/components/Select.svelte';
	import StockFilterRow from '$lib/components/StockFilterRow.svelte';

	const catalog = getCatalog();

	// 导航层级完全由运行时 store（stockNav）驱动：不写 URL、不进 history，
	// 返回按钮/后退手势不会经过 stock 导航的 history entry；刷新回品牌层（store 重置）。
	const selectedBrand = $derived(stockNav.brand);

	const currentBrandGroup = $derived(catalog[selectedBrand] ?? {});
	const selectedSerie = $derived(stockNav.serie || Object.keys(currentBrandGroup)[0] || null);
	const currentSerieGroup = $derived(
		selectedSerie != null ? (currentBrandGroup[selectedSerie] ?? []) : []
	);
	const selectedPaint = $derived(
		stockNav.code ? (currentSerieGroup.find((p) => p.code === stockNav.code) ?? null) : null
	);

	const level = $derived(selectedPaint ? 2 : selectedBrand ? 1 : 0);

	const totalModels = (series: { [key: string]: PaintInfo[] }) =>
		Object.values(series).reduce((n, s) => n + s.length, 0);

	const ownedCountInBrand = (series: { [key: string]: PaintInfo[] }) =>
		Object.values(series).reduce((n, s) => n + ownedCountInSerie(s), 0);

	const ownedCountInSerie = (s: PaintInfo[]) =>
		s.reduce((n, p) => n + (stock.has(p.id) ? 1 : 0), 0);

	// 导航层级由运行时 store（stockNav）驱动，同时用同 URL history entry（state 标记）
	// 记录导航栈：返回按钮/返回手势走 popstate 恢复 state（不依赖 searchParams）。
	// push 新增一档（品牌选择、桌面 PaintDetail）；replace 覆盖当前档（系列切换），
	// 保证系列层只占一档——切换系列后手势回退直接回到品牌层而不是上一系列。
	const navigateTo = (
		params: {
			brand?: string | null;
			serie?: string | null;
			code?: string | null;
		},
		opts?: { replace?: boolean }
	) => {
		const brand = params.brand ?? '';
		const serie = params.serie ?? '';
		const code = params.code ?? '';
		if (brand === stockNav.brand && serie === stockNav.serie && code === stockNav.code) return;
		stockNav.brand = brand;
		stockNav.serie = serie;
		stockNav.code = code;
		const state = { paintboxNav: { brand, serie, code } };
		if (opts?.replace) history.replaceState(state, '');
		else history.pushState(state, '');
	};

	const selectBrand = (brand: string) => {
		const serie = Object.keys(catalog[brand] ?? {})[0] ?? null;
		navigateTo({ brand, serie });
	};

	const selectSerie = (serie: string) => {
		if (!selectedBrand) return;
		navigateTo({ brand: selectedBrand, serie }, { replace: true });
	};

	const selectPaint = (paint: PaintInfo) => {
		if (!isSm()) {
			// 手机端：底部卡片弹出详情，scrim 单击退回列表
			viewStack.push({
				key: paint.id,
				component: PaintDetail,
				props: { paint, inStack: true, isStockPage: true }
			});
		} else {
			navigateTo({ brand: paint.brand, serie: paint.serie, code: paint.code });
		}
	};

	// 返回按钮 = 后退一档：popstate 恢复 history state（含手势共用同一路径）
	const goBack = () => history.back();

	// 系列名面包屑：清空 code 回系列层（覆盖 PaintDetail 档，不新增返回层级）
	const goToLevel1 = () => {
		if (!selectedBrand) return;
		navigateTo({ brand: selectedBrand, serie: selectedSerie }, { replace: true });
	};

	// popstate（返回按钮/手势/tauri 返回键）：恢复 stockNav 到对应导航层级。
	// 不依赖 event.state（个别 webview 下可能丢失）：sheet 打开时由 viewStack 播关闭
	// 动画（栈空前的 popstate 一律不动 stockNav）；栈空时用 history.state 区分
	// sheet 撤销 entry（拖拽/遮罩关闭的 back()）与 stock 导航 entry。
	$effect(() => {
		const onPop = () => {
			if (viewStack.size > 0) return;
			const st = history.state;
			if (st?.paintboxView) return;
			const nav = st?.paintboxNav as { brand?: string; serie?: string; code?: string } | undefined;
			stockNav.brand = nav?.brand ?? '';
			stockNav.serie = nav?.serie ?? '';
			stockNav.code = nav?.code ?? '';
		};
		window.addEventListener('popstate', onPop);
		return () => window.removeEventListener('popstate', onPop);
	});

	// ---- 排序 / 搜索（品牌内跨系列）与筛选：状态放运行时 store，跨路由切回保持 ----
	const sortKey = $derived(stockNav.sortKey);
	const searchOpen = $derived(stockNav.searchOpen);
	const query = $derived(stockNav.query);
	const filterOpen = $derived(stockNav.filterOpen);
	const surfSel = $derived(stockNav.surfSel);
	const baseSel = $derived(stockNav.baseSel);
	const baseFilter = $derived(
		baseSel.length == 0 ? 0x7fffffff : baseSel.reduce((a, b) => a | (1 << +b), 0)
	);
	const surfFilter = $derived(new Set(surfSel.map((k) => SURFACE_BITS[k])));
	const filterCount = $derived(surfSel.length + baseSel.length);
	const passFilter = (p: PaintInfo): boolean => {
		if (surfSel.length > 0 && !surfFilter.has(p.prop)) return false;
		return (baseFilter & p.base) != 0;
	};

	const sortPaints = (list: PaintInfo[]): PaintInfo[] => {
		const arr = [...list];
		switch (sortKey) {
			case 1:
				return arr.sort((a, b) => a.hsl[0] - b.hsl[0]);
			case 2:
				return arr.sort((a, b) => b.hsl[1] - a.hsl[1]);
			case 3:
				return arr.sort((a, b) => b.hsl[2] - a.hsl[2]);
			case 4:
				return arr.sort((a, b) => Number(stock.has(b.id)) - Number(stock.has(a.id)));
			default:
				return arr.sort((a, b) => a.code.localeCompare(b.code, undefined, { numeric: true }));
		}
	};

	const matches = (p: PaintInfo): boolean => {
		const q = query.trim().toLowerCase();
		if (!q) return true;
		return p.code.toLowerCase().includes(q) || paintDesc(p).toLowerCase().includes(q);
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
						bind:value={stockNav.query}
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
							stockNav.searchOpen = false;
							stockNav.query = '';
						}}
						title={t('stock.closeSearch')}
						class="cursor-pointer rounded-full p-1.5 text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-700"
					>
						<X class="h-4 w-4" />
					</button>
				{:else}
					<button
						type="button"
						onclick={() => (stockNav.searchOpen = true)}
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
					bind:value={stockNav.sortKey}
					class="rounded-full border-0 p-1.5
					text-gray-500 ring-0! dark:bg-gray-900 dark:hover:bg-gray-700"
					activeClass="dark:bg-gray-700 dark:text-white"
					lockWidth={false}
					placement="bottom-end"
				>
					<ArrowDownWideNarrow class="h-4 w-4" />
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
			<Dropdown
				class="list-none overflow-hidden!"
				placement="bottom-end"
				bind:isOpen={stockNav.filterOpen}
			>
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
							bind:values={stockNav.surfSel}
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
							bind:values={stockNav.baseSel}
						>
							{t('stock.baseTitle')}
						</StockFilterRow>
					</div>
				</div>
			</Dropdown>
		</div>
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
						{@const inStock = stock.has(paint.id)}
						<div
							role="button"
							tabindex="0"
							onclick={() => selectPaint(paint)}
							onkeydown={(e) => e.key === 'Enter' && selectPaint(paint)}
							class="group relative aspect-square w-full cursor-pointer overflow-hidden rounded-md shadow-sm transition-transform hover:scale-105 {inStock
								? 'ring-[3px] ring-primary-500'
								: 'ring-1 ring-black/10 hover:ring-black/30 dark:ring-white/10 dark:hover:ring-white/30'}"
							style="background-color: {rgbToHex(paint.rgb)}"
							title={paintDesc(paint)}
						>
							<button
								type="button"
								title={inStock ? t('stock.removeFromStock') : t('stock.addToStock')}
								onclick={(e) => {
									e.stopPropagation();
									stock.toggle(paint.id);
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
										: paintDesc(paint)}
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
			{#key `${level}-${paint.brand}-${paint.code}`}
				<div class="h-full" in:fly={{ x: 24, duration: 150 }}>
					<PaintDetail {paint} isStockPage />
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
