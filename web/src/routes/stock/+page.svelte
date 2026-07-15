<script lang="ts">
	import { fly } from 'svelte/transition';
	import { ChevronLeft } from 'lucide-svelte';
	import { Card, Button, ButtonGroup } from 'flowbite-svelte';
	import {
		listPaints,
		groupPaints,
		paintId,
		rgbToHex,
		floatRgbToCss,
		searchNearest,
		type PaintInfo,
		type BrandGroup
	} from '$lib/paints';
	import { stock } from '$lib/stock.svelte';

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
					class="uppercase hover:underline {level === 1
						? 'font-semibold text-gray-900 dark:text-white'
						: 'text-gray-500 dark:text-gray-400'}"
				>
					{selectedBrand}
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
							class="cursor-pointer p-3 hover:bg-gray-50 dark:hover:bg-gray-700"
						>
							<div class="flex items-center gap-3">
								<div
									class="bg-primary-600 flex h-10 w-10 shrink-0 items-center justify-center rounded-full font-bold text-white uppercase"
								>
									{g.brand.slice(0, 2)}
								</div>
								<div class="min-w-0">
									<div class="truncate font-semibold uppercase">{g.brand}</div>
									<div class="text-xs text-gray-500 dark:text-gray-400">
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
						class="w-32 shrink-0 overflow-y-auto border-r border-gray-200 sm:w-44 dark:border-gray-700"
					>
						{#each currentBrandGroup?.series ?? [] as s (s.serie)}
							<button
								type="button"
								onclick={() => selectSerie(s.serie)}
								class="w-full px-3 py-2 text-left text-sm {s.serie === selectedSerie
									? 'bg-primary-50 text-primary-700 dark:bg-gray-700 dark:text-white font-medium'
									: 'text-gray-700 hover:bg-gray-50 dark:text-gray-300 dark:hover:bg-gray-800'}"
							>
								{s.serie}
								<span class="text-xs text-gray-400">({s.paints.length})</span>
							</button>
						{/each}
					</div>
					<div
						class="grid flex-1 auto-rows-min grid-cols-[repeat(auto-fill,minmax(96px,1fr))] gap-2 overflow-y-auto p-2"
					>
						{#each currentSerieGroup?.paints ?? [] as paint (paint.code)}
							<button
								type="button"
								onclick={() => selectPaint(paint)}
								class="group relative flex flex-col items-center rounded-lg p-1.5 hover:bg-gray-100 dark:hover:bg-gray-800"
								title={paint.desc}
							>
								{#if stock.get(paintId(paint)) > 0}
									<span
										class="absolute top-1 right-1 h-2.5 w-2.5 rounded-full bg-green-500 ring-2 ring-white dark:ring-gray-900"
									></span>
								{/if}
								<div
									class="aspect-square w-full rounded-md shadow-inner"
									style="background-color: {rgbToHex(paint.rgb)}"
								></div>
								<div class="mt-1 w-full truncate text-center text-xs font-medium">
									{paint.code}
								</div>
							</button>
						{/each}
					</div>
				</div>
			{/key}
		{:else if level === 2 && selectedPaint}
			{@const paint = selectedPaint}
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
							<div class="mt-1 text-xs text-gray-400 uppercase">
								{paint.brand} · {paint.serie}
							</div>
						</div>

						<div class="flex items-center gap-3">
							<span class="text-sm text-gray-500 dark:text-gray-400">库存</span>
							<ButtonGroup>
								<Button size="sm" onclick={() => stock.add(paintId(paint), -1)}>-</Button>
								<Button size="sm" color="light" class="pointer-events-none w-12 justify-center">
									{stock.get(paintId(paint))}
								</Button>
								<Button size="sm" onclick={() => stock.add(paintId(paint), 1)}>+</Button>
							</ButtonGroup>
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
