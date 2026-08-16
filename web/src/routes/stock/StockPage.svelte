<script lang="ts">
	import { fade } from 'svelte/transition';
	import { ChevronLeft, Check, Plus, Search, X, Funnel, ExternalLink } from '@lucide/svelte';
	import { Card, Badge } from 'flowbite-svelte';
	import {
		getCatalog,
		rgbToHex,
		SURFACE_BITS,
		MEDIUM_BITS,
		type PaintInfo
	} from '$lib/paints.svelte';
	import { stock } from '$lib/stock.svelte';
	import { stockNav, goBackOneLevel } from '$lib/stocknav.svelte';
	import { isMedia, openExternal } from '$lib/utils.svelte';
	import { drawer } from '$lib/drawer.svelte';
	import { registerBackHandler, unregisterBackHandler } from '$lib/back.svelte';
	import { getBrandMeta, getSerieMeta, serieThumb } from '$lib/meta';
	import PaintDetail from '$lib/components/PaintDetail.svelte';
	import DetailEmpty from '$lib/components/DetailEmpty.svelte';
	import { paintDesc } from '$lib/i18ndyn.svelte';
	import { t } from '$lib/i18n.svelte';
	import StockFilterPanel from '$lib/components/StockFilterPanel.svelte';

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

	// 导航层级完全由运行时 store（stockNav）驱动：不写 URL、不进 history，
	// 返回按钮/返回手势只改内存状态（见 goBack / goBackOneLevel）。
	const navigateTo = (params: {
		brand?: string | null;
		serie?: string | null;
		code?: string | null;
	}) => {
		const brand = params.brand ?? '';
		const serie = params.serie ?? '';
		const code = params.code ?? '';
		if (brand === stockNav.brand && serie === stockNav.serie && code === stockNav.code) return;
		stockNav.brand = brand;
		stockNav.serie = serie;
		stockNav.code = code;
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
		if (!isMedia().sm) {
			// 手机端：底部卡片弹出详情，scrim 单击退回列表
			drawer.open({
				key: paint.id,
				component: PaintDetail,
				props: { paint, isStockPage: true }
			});
		} else {
			navigateTo({ brand: paint.brand, serie: paint.serie, code: paint.code });
		}
	};

	// 返回按钮 = 后退一档（纯内存状态，与 Tauri 返回手势共用 goBackOneLevel）
	const goBack = () => goBackOneLevel();

	// 系列名面包屑：清空 code 回系列层（覆盖 PaintDetail 档，不新增返回层级）
	const goToLevel1 = () => {
		if (!selectedBrand) return;
		navigateTo({ brand: selectedBrand, serie: selectedSerie });
	};

	// Tauri Android 返回手势：本页挂载期间注册逐层回退（型号→系列→品牌），
	// 抽屉打开时由 back 分发器优先消费，页面无感知；卸载即注销。
	// Tauri Android 返回手势：仅在有可退层级时登记（根级注销，系统接管 back → predictive 退出）；
	// 抽屉打开时由 back 分发器优先消费（dispatchBack 先查 drawer.isOpen），页面无感知。
	$effect(() => {
		if (level <= 0) return;
		const handler = () => goBackOneLevel();
		registerBackHandler(handler);
		return () => unregisterBackHandler(handler);
	});

	// ---- 排序 / 搜索（品牌内跨系列）与筛选：状态放运行时 store，跨路由切回保持 ----
	const sortKey = $derived(stockNav.sortKey);
	const searchOpen = $derived(stockNav.searchOpen);
	const query = $derived(stockNav.query);
	const surfSel = $derived(stockNav.surfSel);
	const baseSel = $derived(stockNav.baseSel);
	const mediumSel = $derived(stockNav.mediumSel);
	const baseFilter = $derived(
		baseSel.length == 0 ? 0x7fffffff : baseSel.reduce((a, b) => a | (1 << +b), 0)
	);
	const surfFilter = $derived(
		surfSel.length == 0 ? 0x7fffffff : surfSel.reduce((a, b) => a | SURFACE_BITS[b], 0)
	);
	const mediumFilter = $derived(
		mediumSel.length == 0 ? 0x7fffffff : mediumSel.reduce((a, b) => a | MEDIUM_BITS[b], 0)
	);
	const filterCount = $derived(surfSel.length + baseSel.length + mediumSel.length);
	const passFilter = (p: PaintInfo): boolean => {
		return (
			(surfFilter & p.surfaces) != 0 &&
			(baseFilter & p.bases) != 0 &&
			(mediumFilter & p.mediums) != 0
		);
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

{#snippet filterBar()}
	<div
		class="border-theme dark:border-theme max-h-[50dvh] shrink-0 overflow-y-auto border-b px-4 py-3"
	>
		<StockFilterPanel />
	</div>
{/snippet}

{#snippet filterBtn()}
	<div class="relative shrink-0">
		<button
			type="button"
			title={t('stock.filterTitle')}
			onclick={() => (stockNav.filterOpen = !stockNav.filterOpen)}
			class="relative cursor-pointer rounded-full p-1.5 text-gray-500 hover:bg-gray-100 hover:text-gray-700 dark:hover:bg-gray-700 dark:hover:text-gray-200 {stockNav.filterOpen
				? 'bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-white'
				: filterCount > 0
					? 'text-primary-500'
					: ''}"
		>
			<Funnel class="h-4 w-4" />
			{#if filterCount > 0}
				<!-- 小蓝点：非默认筛选设置时提示有改动 -->
				<span class="absolute -top-0.5 -right-0.5 h-2 w-2 rounded-full bg-primary-500"></span>
			{/if}
		</button>
	</div>
{/snippet}

{#snippet seriesNav()}
	{#if isMedia().xl}
		<!-- 桌面侧栏：宽栏（图标 + 系列名/描述 + 库存） -->
		<div class="border-theme dark:border-theme w-56 shrink-0 overflow-y-auto border-r">
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
	{:else}
		<!-- 移动侧栏：极窄栏（第一行 图标 + 库存角标 + 型号总数，第二行 系列名小字） -->
		<div class="border-theme dark:border-theme w-24 shrink-0 overflow-y-auto border-r">
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
					<div class="relative aspect-square w-full overflow-hidden rounded-md shadow-md">
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
						<div class="absolute inset-x-0 bottom-0 bg-black/65 px-0.5 py-0.5 backdrop-blur-[1px]">
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
	{/if}
{/snippet}

{#snippet brandCard(brand: string)}
	{@const series = catalog[brand]}
	<Card
		onclick={() => selectBrand(brand)}
		role="button"
		tabindex={0}
		size="md"
		shadow="xs"
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
				<div class="text-[11px] text-gray-500 dark:text-gray-400">
					{t('stock.brandStats', {
						series: Object.keys(series).length,
						paints: totalModels(series)
					})}
				</div>
			</div>
		</div>
	</Card>
{/snippet}

{#snippet paintCard(paint: PaintInfo)}
	{@const inStock = stock.has(paint.id)}
	<div
		role="button"
		tabindex="0"
		onclick={() => selectPaint(paint)}
		onkeydown={(e) => e.key === 'Enter' && selectPaint(paint)}
		class="group relative aspect-square w-full cursor-pointer overflow-hidden rounded-md shadow-md transition-transform hover:scale-105 {inStock
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
				{paintDesc(paint)}
			</div>
		</div>
	</div>
{/snippet}

<div class="flex h-full flex-col">
	<div class="border-theme dark:border-theme shrink-0 border-b px-4 py-2">
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
					class="transition-opacity duration-150 {searchOpen && level > 0
						? 'max-sm:pointer-events-none max-sm:opacity-0'
						: 'opacity-100'}"
				>
					<div class="flex min-w-0 items-center gap-1.5">
						<span class="truncate text-xl font-semibold text-gray-900 dark:text-white">
							{level > 0 ? (getBrandMeta(selectedBrand)?.name ?? selectedBrand) : t('stock.brands')}
						</span>
						{#if level > 0}
							{@const bmeta = getBrandMeta(selectedBrand)}
							{#if bmeta?.url}
								<button
									type="button"
									onclick={() => openExternal(bmeta.url)}
									title={bmeta?.name}
									class="flex h-5 w-5 shrink-0 cursor-pointer items-center justify-center rounded-md bg-gray-100 text-gray-500 transition-colors hover:bg-gray-200 dark:bg-gray-700 dark:text-gray-300 dark:hover:bg-gray-600"
								>
									<ExternalLink class="h-3 w-3" />
								</button>
							{/if}
						{/if}
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
				{#if searchOpen && level > 0}
					<input
						type="search"
						autofocus
						in:fade={{ duration: 150 }}
						out:fade={{ duration: 150 }}
						bind:value={stockNav.query}
						placeholder={t('stock.searchPlaceholder')}
						class="border-theme dark:border-theme absolute top-1/2 left-0 h-9
							w-full -translate-y-1/2 rounded-md border bg-gray-100 px-2 py-1.5
							text-sm placeholder:text-gray-500 focus:border-primary-500
							focus:outline-none sm:right-0 sm:left-auto
							sm:w-2/3 dark:bg-gray-800 dark:text-white dark:placeholder:text-gray-400"
					/>
				{/if}
			</div>
			{#if level > 0}
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
			{/if}
		</div>
	</div>

	<div class="flex-1 overflow-hidden">
		<div class="flex h-full">
			<!-- 左列：品牌列表 或 系列+油漆（详情在右侧面板时列表保持，可继续点选） -->
			<div class="min-w-0 flex-1 overflow-hidden">
				{#if level === 0}
					{#key level}
						<div
							class="grid h-full auto-rows-min grid-cols-[repeat(auto-fill,minmax(240px,1fr))] gap-3 overflow-y-auto px-6 py-4"
						>
							{#each Object.keys(catalog) as brand}
								{@render brandCard(brand)}
							{/each}
						</div>
					{/key}
				{:else}
					{#key selectedBrand}
						<!-- 手机：筛选条横跨整页（第 1、2 列上方）；PC：位于第 2 列（油漆区）上方 -->
						<div class="flex h-full flex-col sm:flex-row">
							{#if !isMedia().sm && stockNav.filterOpen}
								{@render filterBar()}
							{/if}
							<div class="flex min-h-0 flex-1">
								{@render seriesNav()}
								<div class="flex min-w-0 flex-1 flex-col">
									<div
										class="grid min-h-0 flex-1 auto-rows-min grid-cols-[repeat(auto-fill,minmax(64px,1fr))] gap-2.5 overflow-y-auto p-2"
									>
										{#if searchSerieResults}
											{#each searchSerieResults as paint (paint.code)}
												{@render paintCard(paint)}
											{/each}
											{#if searchSerieResults.length === 0}
												<div class="col-span-full p-4 text-center text-xs text-gray-400">
													{t('stock.noResults')}
												</div>
											{/if}
										{:else}
											{#each sortedCurrentSerie as paint (paint.code)}
												{@render paintCard(paint)}
											{/each}
										{/if}
									</div>
								</div>
							</div>
						</div>
					{/key}
				{/if}
			</div>

			<!-- 右栏：详情面板（品牌层及以上常驻；根路由纯列表不加分栏） -->
			{#if isMedia().sm && level > 0}
				<aside
					class="flex w-[clamp(18rem,28vw,26rem)] shrink-0 flex-col border-l border-theme dark:border-theme"
				>
					{#if stockNav.filterOpen}
						{@render filterBar()}
					{/if}
					<div class="min-h-0 flex-1 overflow-y-auto">
						{#if selectedPaint}
							{@const paint = selectedPaint}
							{#key paint.id}
								<div class="p-4">
									<PaintDetail {paint} isStockPage />
								</div>
							{/key}
						{:else}
							<DetailEmpty hint={t('stock.selectPaintHint')} />
						{/if}
					</div>
				</aside>
			{/if}
		</div>
	</div>
</div>
