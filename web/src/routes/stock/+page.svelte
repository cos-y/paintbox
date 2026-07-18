<script lang="ts">
	import { fly } from 'svelte/transition';
	import { ChevronLeft, Check, Plus } from 'lucide-svelte';
	import { Card, Button, Badge } from 'flowbite-svelte';
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
		type PaintInfo
	} from '$lib/paints';
	import { stock } from '$lib/stock.svelte';
	import { getBrandMeta, getSerieMeta, serieThumb } from '$lib/meta';
	import { similarity } from '$lib/utils';

	const allPaints = listPaints();
	const paintByKey = new Map(allPaints.map((p) => [paintId(p), p]));
	const catalog = getCatalog(allPaints);

	// 用查询参数（?brand=&serie=&code=）驱动状态，而不是纯内部state：
	// 这样浏览时地址栏会实时更新，且任意页面都能通过完整URL直接分享/刷新进入，
	// 同时因为只有查询参数在变、路径始终是同一个/stock，静态文件服务器不需要
	// 为每个品牌/系列/型号单独生成页面文件。
	const selectedBrand = $derived(page.url.searchParams.get('brand'));
	const selectedSerieParam = $derived(page.url.searchParams.get('serie'));
	const selectedCode = $derived(page.url.searchParams.get('code'));

	const currentBrandGroup = $derived(selectedBrand ? (catalog[selectedBrand] ?? null) : null);
	const selectedSerie = $derived(
		selectedSerieParam ?? Object.keys(currentBrandGroup ?? {})[0] ?? null
	);
	const currentSerieGroup = $derived((currentBrandGroup ?? {})[selectedSerie] ?? null);
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
</script>

<div class="flex h-full flex-col">
	<div
		class="flex shrink-0 items-center gap-2 border-b border-gray-200 px-4 py-3 dark:border-gray-700"
	>
		{#if level > 0}
			<button
				type="button"
				aria-label="返回"
				onclick={goBack}
				class="cursor-pointer rounded-full p-1 hover:bg-gray-100 dark:hover:bg-gray-700"
			>
				<ChevronLeft />
			</button>
		{/if}
		<nav class="flex items-center gap-1 text-sm">
			<button
				type="button"
				onclick={goToLevel0}
				class="cursor-pointer hover:underline {level === 0
					? 'font-semibold text-gray-900 dark:text-white'
					: 'text-gray-500 dark:text-gray-400'}"
			>
				Paints
			</button>
			{#if selectedBrand}
				<span class="text-gray-400">/</span>
				<button
					type="button"
					onclick={goToLevel1}
					class="cursor-pointer hover:underline {level === 1
						? 'font-semibold text-gray-900 dark:text-white'
						: 'text-gray-500 dark:text-gray-400'}"
				>
					{getBrandMeta(selectedBrand)?.name ?? selectedBrand}
				</button>
			{/if}
			{#if selectedPaint}
				<span class="text-gray-400">/</span>
				<span class="cursor-pointer font-semibold text-gray-900 dark:text-white"
					>{selectedPaint.code}
				</span>
			{/if}
		</nav>
	</div>

	{#snippet labelPrimary(paint: PaintInfo)}
		{@const brandMeta = getBrandMeta(paint.brand)}
		{@const serieMeta = getSerieMeta(paint.brand, paint.serie)}
		<span class="text-gray-600 dark:text-gray-300">
			<Button
				color="secondary"
				class="inline-block cursor-pointer p-0 text-gray-600 dark:text-gray-300"
				onclick={() => navigateTo({ brand: paint.brand, serie: paint.serie })}
			>
				{serieMeta?.name ?? paint.serie}
			</Button>
			/
			<Button
				color="secondary"
				class="inline-block cursor-pointer p-0 text-gray-600 dark:text-gray-300"
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
					class="grid h-full auto-rows-min grid-cols-[repeat(auto-fill,minmax(240px,1fr))] gap-3 overflow-y-auto p-4"
					in:fly={{ x: -24, duration: 150 }}
				>
					{#each Object.entries(catalog) as [brand, series]}
						<Card
							onclick={() => selectBrand(brand)}
							role="button"
							tabindex={0}
							size="sm"
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
										{Object.keys(series).length} series · {totalModels(series)} models
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
					<div
						class="w-40 shrink-0 overflow-y-auto border-r border-gray-200 sm:w-56 dark:border-gray-700"
					>
						{#each Object.entries(currentBrandGroup ?? {}) as [serie, paints]}
							{@const serieMeta = getSerieMeta(selectedBrand ?? '', serie)}
							{@const ownCount = ownedCountInSerie(paints)}
							<button
								type="button"
								onclick={() => selectSerie(serie)}
								title={serieMeta?.desc}
								class="cursor-pointer flex w-full items-center gap-2 px-2 py-1.5 text-left text-xs {serie ===
								selectedSerie
									? 'bg-primary-50 text-primary-700 dark:bg-gray-700 dark:text-white font-medium'
									: 'text-gray-700 hover:bg-gray-50 dark:text-gray-300 dark:hover:bg-gray-800'}"
							>
								<img
									src={serieThumb(selectedBrand ?? '', serie)}
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
					<div
						class="grid flex-1 auto-rows-min grid-cols-[repeat(auto-fill,minmax(64px,1fr))] gap-2.5 overflow-y-auto p-2"
					>
						{#each currentSerieGroup ?? [] as paint (paint.code)}
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
									title={inStock ? 'remove from stock' : 'add to stock'}
									onclick={(e) => {
										e.stopPropagation();
										stock.toggle(paintId(paint));
										e.currentTarget.blur();
									}}
									class="cursor-pointer absolute top-0 right-0 h-6 w-6 scale-75 text-white opacity-0 transition-all duration-150 group-hover:scale-100 group-hover:opacity-100 focus:scale-100 focus:opacity-100 {inStock
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
								<div
									class="absolute inset-x-0 bottom-0 bg-black/55 px-1 py-0.5 backdrop-blur-[1px]"
								>
									<div class="truncate text-[10px] leading-tight font-semibold text-white">
										{paint.code}
									</div>
									<div class="truncate text-[9px] leading-tight text-white/75">
										{paint.desc}
									</div>
								</div>
							</div>
						{/each}
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
									{@render labelPrimary(paint)}
								</div>
								<div class="font-bold text-gray-500 dark:text-gray-400">{paint.desc}</div>
							</div>
							<button
								type="button"
								aria-label={inStock ? '移出油漆库' : '加入油漆库'}
								onclick={() => stock.toggle(paintId(paint))}
								class="cursor-pointer flex h-9 w-9 shrink-0 items-center justify-center rounded-full transition-colors {inStock
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
									<div class="text-xs text-gray-400 flex-1">
										{similarity(compareDeltaE).toFixed(0)}% 相似
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
								直接等价
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
									<div class="text-xs text-gray-400">暂无同名的其他油漆</div>
								{/each}
							</div>
						</div>

						<div>
							<h3 class="mb-2 text-sm font-semibold text-gray-500 uppercase dark:text-gray-400">
								相近同色漆
							</h3>
							<div class="flex flex-wrap gap-2">
								{#each colorEquivalences as p (paintId(p))}
									<button
										type="button"
										onclick={() => toggleCompare(p)}
										class="cursor-pointer flex items-center gap-2 rounded-lg border px-2 py-1 {compareCode ===
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
									<div class="text-xs text-gray-400">暂无相近的其他油漆</div>
								{/each}
							</div>
						</div>
					</div>
				</div>
			{/key}
		{/if}
	</div>
</div>
