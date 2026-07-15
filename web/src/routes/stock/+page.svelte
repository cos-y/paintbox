<script lang="ts">
	import { fly } from 'svelte/transition';
	import { ChevronLeft, Check, Plus } from 'lucide-svelte';
	import { Card, Button, Badge } from 'flowbite-svelte';
	import {
		listPaints,
		groupPaints,
		paintId,
		rgbToHex,
		floatRgbToCss,
		searchNearest,
		type PaintInfo,
		type BrandGroup,
		type SerieGroup
	} from '$lib/paints';
	import { stock } from '$lib/stock.svelte';
	import { getBrandMeta, getSerieMeta, serieThumb } from '$lib/meta';

	const groups: BrandGroup[] = groupPaints(listPaints());

	type Level = 0 | 1 | 2;
	let level: Level = $state(0);
	let selectedBrand: string | null = $state(null);
	let selectedSerie: string | null = $state(null);
	let selectedPaint: PaintInfo | null = $state(null);

	const currentBrandGroup = $derived(groups.find((g) => g.brand === selectedBrand) ?? null);
	const currentSerieGroup = $derived(
		currentBrandGroup?.series.find((s) => s.serie === selectedSerie) ?? null
	);

	const totalModels = (g: BrandGroup) => g.series.reduce((n, s) => n + s.paints.length, 0);

	const ownedCountInBrand = (g: BrandGroup) =>
		g.series.reduce((n, s) => n + ownedCountInSerie(s), 0);

	const ownedCountInSerie = (s: SerieGroup) =>
		s.paints.reduce((n, p) => n + (stock.has(paintId(p)) ? 1 : 0), 0);

	const selectBrand = (brand: string) => {
		selectedBrand = brand;
		selectedSerie = groups.find((g) => g.brand === brand)?.series[0]?.serie ?? null;
		level = 1;
	};

	const selectSerie = (serie: string) => {
		selectedSerie = serie;
	};

	const selectPaint = (paint: PaintInfo) => {
		selectedPaint = paint;
		level = 2;
	};

	const goToLevel0 = () => {
		level = 0;
		selectedBrand = null;
		selectedSerie = null;
		selectedPaint = null;
	};

	const goToLevel1 = () => {
		level = 1;
		selectedPaint = null;
	};

	const goBack = () => (level === 2 ? goToLevel1() : goToLevel0());

	const equivalences = $derived.by(() => {
		if (!selectedPaint) return [];
		const paint = selectedPaint;
		return searchNearest(paint.rgb, 0, 8)
			.map((r) => r.portions[0])
			.filter((p) => p && !(p.brand === paint.brand && p.code === paint.code));
	});
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
				class="rounded-full p-1 hover:bg-gray-100 dark:hover:bg-gray-700"
			>
				<ChevronLeft />
			</button>
		{/if}
		<nav class="flex items-center gap-1 text-sm">
			<button
				type="button"
				onclick={goToLevel0}
				class="hover:underline {level === 0
					? 'font-semibold text-gray-900 dark:text-white'
					: 'text-gray-500 dark:text-gray-400'}"
			>
				全部
			</button>
			{#if selectedBrand}
				<span class="text-gray-400">/</span>
				<button
					type="button"
					onclick={goToLevel1}
					class="hover:underline {level === 1
						? 'font-semibold text-gray-900 dark:text-white'
						: 'text-gray-500 dark:text-gray-400'}"
				>
					{getBrandMeta(selectedBrand)?.name ?? selectedBrand}
				</button>
			{/if}
			{#if selectedPaint}
				<span class="text-gray-400">/</span>
				<span class="font-semibold text-gray-900 dark:text-white">{selectedPaint.code}</span>
			{/if}
		</nav>
	</div>

	<div class="flex-1 overflow-hidden">
		{#if level === 0}
			{#key level}
				<div
					class="grid h-full auto-rows-min grid-cols-[repeat(auto-fill,minmax(160px,1fr))] gap-3 overflow-y-auto p-4"
					in:fly={{ x: -24, duration: 150 }}
				>
					{#each groups as g (g.brand)}
						<Card
							onclick={() => selectBrand(g.brand)}
							role="button"
							tabindex={0}
							size="sm"
							class="relative cursor-pointer p-3 hover:bg-gray-50 dark:hover:bg-gray-700"
						>
							{#if ownedCountInBrand(g) > 0}
								<Badge
									color="green"
									class="absolute top-2 right-2 bg-green-500 text-white dark:bg-green-500 dark:text-white"
								>
									{ownedCountInBrand(g)}
								</Badge>
							{/if}
							{@const meta = getBrandMeta(g.brand)}
							<div class="flex items-center gap-3">
								<img
									src="/brands/{g.brand}.png"
									alt={g.brand}
									class="h-10 w-10 shrink-0 rounded-full bg-white object-cover ring-1 ring-black/10"
								/>
								<div class="min-w-0">
									<div class="truncate font-semibold">{meta?.name ?? g.brand}</div>
									{#if meta?.desc}
										<div class="truncate text-xs text-gray-500 dark:text-gray-400">{meta.desc}</div>
									{/if}
									<div class="text-[11px] text-gray-400">
										{g.series.length} 系列 · {totalModels(g)} 型号
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
						{#each currentBrandGroup?.series ?? [] as s (s.serie)}
							{@const serieMeta = getSerieMeta(selectedBrand ?? '', s.serie)}
							<button
								type="button"
								onclick={() => selectSerie(s.serie)}
								title={serieMeta?.desc}
								class="flex w-full items-center gap-2 px-2 py-1.5 text-left text-xs {s.serie ===
								selectedSerie
									? 'bg-primary-50 text-primary-700 dark:bg-gray-700 dark:text-white font-medium'
									: 'text-gray-700 hover:bg-gray-50 dark:text-gray-300 dark:hover:bg-gray-800'}"
							>
								<img
									src={serieThumb(selectedBrand ?? '', s.serie)}
									alt=""
									class="h-7 w-7 shrink-0 rounded bg-white object-cover ring-1 ring-black/10"
									onerror={(e) => {
									if (e.currentTarget instanceof HTMLElement) {
										e.currentTarget.style.visibility = 'hidden';
									}
								}}
								/>
								<span class="min-w-0 flex-1">
									<span class="block truncate">{serieMeta?.name ?? s.serie}</span>
									<span class="block truncate text-[10px] text-gray-400"
										>{s.serie} · {s.paints.length}型号</span
									>
								</span>
								{#if ownedCountInSerie(s) > 0}
									<Badge color="green" class="bg-green-500 text-white dark:bg-green-500 dark:text-white">
										{ownedCountInSerie(s)}
									</Badge>
								{/if}
							</button>
						{/each}
					</div>
					<div
						class="grid flex-1 auto-rows-min grid-cols-[repeat(auto-fill,minmax(64px,1fr))] gap-2.5 overflow-y-auto p-2"
					>
						{#each currentSerieGroup?.paints ?? [] as paint (paint.code)}
							{@const owned = stock.has(paintId(paint))}
							<div
								role="button"
								tabindex="0"
								onclick={() => selectPaint(paint)}
								onkeydown={(e) => e.key === 'Enter' && selectPaint(paint)}
								class="group relative aspect-square w-full cursor-pointer overflow-hidden rounded-md shadow-sm transition-transform hover:scale-105 {owned
									? 'ring-[3px] ring-green-500'
									: 'ring-1 ring-black/10 hover:ring-black/30 dark:ring-white/10 dark:hover:ring-white/30'}"
								style="background-color: {rgbToHex(paint.rgb)}"
								title={paint.desc}
							>
								<button
									type="button"
									aria-label={owned ? '移出油漆库' : '加入油漆库'}
									onclick={(e) => {
										e.stopPropagation();
										stock.toggle(paintId(paint));
										e.currentTarget.blur();
									}}
									class="absolute top-0 right-0 h-6 w-6 scale-75 text-white opacity-0 transition-all duration-150 group-hover:scale-100 group-hover:opacity-100 focus:scale-100 focus:opacity-100 {owned
										? 'scale-100 opacity-100'
										: ''}"
								>
									<span
										class="absolute inset-0 [clip-path:polygon(100%_0,0_0,100%_100%)] {owned
											? 'bg-green-500'
											: 'bg-black/60 hover:bg-black/75'}"
									></span>
									<span class="absolute top-0.5 right-0.5">
										{#if owned}
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
			{@const brandMeta = getBrandMeta(paint.brand)}
			{@const serieMeta = getSerieMeta(paint.brand, paint.serie)}
			{#key `${level}-${paint.brand}-${paint.code}`}
				<div class="h-full overflow-y-auto p-4" in:fly={{ x: 24, duration: 150 }}>
					<div class="mx-auto max-w-xl space-y-4">
						<div
							class="h-32 rounded-lg shadow-inner"
							style="background-color: {rgbToHex(paint.rgb)}"
						></div>

						<div>
							<div class="text-2xl font-bold">{paint.code}</div>
							<div class="text-gray-600 dark:text-gray-300">{paint.desc}</div>
							<div class="mt-1 flex items-center gap-2">
								<img
									src="/brands/{paint.brand}.png"
									alt=""
									class="h-4 w-4 rounded-full bg-white object-cover ring-1 ring-black/10"
								/>
								<span class="text-xs text-gray-400">
									{brandMeta?.name ?? paint.brand} · {serieMeta?.name ?? paint.serie}
								</span>
							</div>
							{#if serieMeta?.desc}
								<div class="mt-0.5 text-xs text-gray-400">{serieMeta.desc}</div>
							{/if}
						</div>

						<div class="flex items-center gap-3">
							<span class="text-sm text-gray-500 dark:text-gray-400">库存</span>
							<Button
								size="sm"
								color={stock.has(paintId(paint)) ? 'red' : 'primary'}
								onclick={() => stock.toggle(paintId(paint))}
							>
								{stock.has(paintId(paint)) ? '移出油漆库' : '加入油漆库'}
							</Button>
						</div>

						<div>
							<h3 class="mb-2 text-sm font-semibold text-gray-500 uppercase dark:text-gray-400">
								相近同色漆
							</h3>
							<div class="flex flex-wrap gap-2">
								{#each equivalences as p}
									<div
										class="flex items-center gap-2 rounded-lg border border-gray-200 px-2 py-1 dark:border-gray-700"
									>
										<div class="h-5 w-5 rounded" style="background-color: {floatRgbToCss(p.rgb)}"></div>
										<span class="text-xs uppercase">{p.brand}/{p.code}</span>
									</div>
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
