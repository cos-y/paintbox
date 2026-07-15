<script lang="ts">
	import { useMode, modeHwb, modeRgb, modeOklch, type Oklch } from 'culori/fn';

	import Hsl from '$lib/components/Hsl.svelte';
	import Rgb from '$lib/components/Rgb.svelte';
	import { Box, ChevronUp, ChevronDown, Cylinder, Pipette, Check, Plus } from 'lucide-svelte';
	import { Button, Dropdown, DropdownItem } from 'flowbite-svelte';
	import {
		listPaints,
		groupPaints,
		paintId,
		floatRgbToCss,
		searchNearest,
		type BrandGroup,
		type FilterOptions,
		type SearchResult
	} from '$lib/paints';
	import { stock } from '$lib/stock.svelte';
	import { getBrandMeta, getSerieMeta, serieThumb } from '$lib/meta';
	import { clamp } from '$lib/utils';

	let oklch: Oklch = $state({ mode: 'oklch', l: 0, c: 0, h: 0 });

	const toHwb = useMode(modeHwb);
	const toRgb = useMode(modeRgb);
	const toOklch = useMode(modeOklch);
	const hwb = $derived(toHwb(oklch));
	const rgb = $derived(toRgb(oklch));

	let model = $state(0);
	const models = [
		{ name: 'HSL', icon: Cylinder, picker: Hsl },
		{ name: 'RGB', icon: Box, picker: Rgb }
	];

	let modelDropdownOpen = $state(false);

	const handleSelectModel = (newModel: number) => {
		model = newModel;
		modelDropdownOpen = false;
	};

	const eyedrop = () => {
		if ('EyeDropper' in window) {
			let EyeDropper = (window as any).EyeDropper;
			const eyeDropper = new EyeDropper();
			eyeDropper.open().then((result: any) => {
				const hex = parseInt(result.sRGBHex.slice(1), 16);
				const b = (hex & 0xff) / 255;
				const g = ((hex >> 8) & 0xff) / 255;
				const r = ((hex >> 16) & 0xff) / 255;
				oklch = toOklch({ mode: 'rgb', r, g, b });
			});
		}
	};

	const rgbInt = $derived.by(() => {
		const r = clamp(Math.round(rgb.r * 255), 0, 255);
		const g = clamp(Math.round(rgb.g * 255), 0, 255);
		const b = clamp(Math.round(rgb.b * 255), 0, 255);
		return (r << 16) | (g << 8) | b;
	});

	const allPaints = listPaints();
	const groups: BrandGroup[] = groupPaints(allPaints);

	const serieKey = (brand: string, serie: string) => `${brand}::${serie}`;

	let selectedSeries: Set<string> = $state(new Set());
	let ownedOnly = $state(false);
	let ownedDropdownOpen = $state(false);
	let activeFilterBrand: string | null = $state(null);
	// TODO: 混合搜索(max_mix>0)在候选集较大时会长时间阻塞主线程，
	// 在wasm端做出让步（分批/worker）之前先固定为0，禁止混色查询。
	const maxMix = 0;

	const toggleSerie = (brand: string, serie: string) => {
		const key = serieKey(brand, serie);
		const next = new Set(selectedSeries);
		if (next.has(key)) next.delete(key);
		else next.add(key);
		selectedSeries = next;
	};

	const isBrandFullySelected = (g: BrandGroup) =>
		g.series.every((s) => selectedSeries.has(serieKey(g.brand, s.serie)));

	const selectedCountInBrand = (g: BrandGroup) =>
		g.series.filter((s) => selectedSeries.has(serieKey(g.brand, s.serie))).length;

	const toggleBrandAll = (g: BrandGroup) => {
		const on = !isBrandFullySelected(g);
		const next = new Set(selectedSeries);
		for (const s of g.series) {
			const key = serieKey(g.brand, s.serie);
			if (on) next.add(key);
			else next.delete(key);
		}
		selectedSeries = next;
	};

	const clearFilters = () => {
		selectedSeries = new Set();
		ownedOnly = false;
	};

	const filterOptions: FilterOptions = $derived.by(() => {
		const series = [...selectedSeries].map((key) => {
			const [brand, serie] = key.split('::');
			return { brand, serie };
		});
		const owned = ownedOnly
			? allPaints.filter((p) => stock.has(paintId(p))).map((p) => p.index)
			: undefined;
		return { series, owned };
	});

	let results: SearchResult[] = $state([]);

	$effect(() => {
		const targetRgb = rgbInt;
		const filter = filterOptions;
		const mix = maxMix;
		const handle = setTimeout(() => {
			results = searchNearest(targetRgb, mix, 12, filter);
		}, 150);
		return () => clearTimeout(handle);
	});

	const similarity = (deltaE: number) => clamp(100 - deltaE * 4, 0, 100);
</script>

<div class="flex h-full flex-col overflow-y-auto p-4">
	<div
		class="color-picker-root"
		style="
    --picker-oklchLightness: {oklch.l};
    --picker-oklchChroma: {oklch.c};
    --picker-oklchHue: {oklch.h ?? 0};
    --picker-hue: {hwb.h ?? 0};
    --picker-whiteness: {hwb.w};
    --picker-blackness: {hwb.b};
    --picker-color-srgb: rgb({rgb.r * 255} {rgb.g * 255} {rgb.b * 255});
    --picker-oklch: oklch({oklch.l} {oklch.c} {oklch.h})"
	>
		{#snippet picker()}
			{@const { name, icon: Icon, picker: Picker } = models[model]}

			<div>
				<div class="color-swatch relative">
					<div class="flex right-0">
						<button type="button" class="flex-1" onclick={eyedrop}>
							<Pipette size="1rem" color="#666" />
						</button>
					</div>
				</div>

				<div>
					<Button size="xs" class="w-24" aria-label="Choose color model">
						<Icon class="size-4" />
						{name}
						<ChevronUp size="12" />
					</Button>
					<Dropdown placement="top" class="w-30 text-xs" bind:isOpen={modelDropdownOpen}>
						{#each models as { name, icon: Icon }, i}
							<DropdownItem
								class={i === model ? 'bg-gray-100 dark:bg-gray-600' : ''}
								onclick={() => handleSelectModel(i)}
							>
								<span class="flex items-center gap-1"><Icon class="size-4" />{name}</span>
							</DropdownItem>
						{/each}
					</Dropdown>
				</div>
			</div>

			<Picker bind:oklch />
		{/snippet}

		{@render picker()}
	</div>

	<div class="mt-4 flex flex-wrap items-center gap-2 border-y border-gray-200 py-2 dark:border-gray-700">
		<span class="text-xs whitespace-nowrap text-gray-500 dark:text-gray-400">
			过滤器 · {results.length} 个结果
		</span>

		<Button size="xs" color="alternative" class="gap-1">
			按系列筛选{selectedSeries.size > 0 ? ` (${selectedSeries.size})` : ''}
			<ChevronDown class="h-3 w-3" />
		</Button>
		<Dropdown class="w-136 p-0" placement="bottom-start">
			<div class="flex h-96">
				<div
					class="w-40 shrink-0 overflow-y-auto border-r border-gray-200 py-1 dark:border-gray-700"
				>
					{#each groups as g (g.brand)}
						{@const selectedCount = selectedCountInBrand(g)}
						<button
							type="button"
							onmouseenter={() => (activeFilterBrand = g.brand)}
							onclick={() => (activeFilterBrand = g.brand)}
							class="flex w-full items-center gap-2 px-2.5 py-2 text-left text-sm text-gray-700 dark:text-gray-200 {activeFilterBrand ===
							g.brand
								? 'bg-gray-100 dark:bg-gray-600'
								: 'hover:bg-gray-50 dark:hover:bg-gray-800'}"
						>
							<img
								src="/brands/{g.brand}.png"
								alt=""
								class="h-7 w-7 shrink-0 rounded-full bg-white object-cover ring-1 ring-black/10"
							/>
							<span class="min-w-0 flex-1 truncate">{getBrandMeta(g.brand)?.name ?? g.brand}</span>
							{#if selectedCount > 0}
								<span
									class="flex h-4 min-w-4 shrink-0 items-center justify-center rounded-full bg-green-500 px-1 text-[10px] font-medium text-white"
								>
									{selectedCount}
								</span>
							{/if}
						</button>
					{/each}
				</div>
				<div class="flex-1 overflow-y-auto p-3">
					{#if activeFilterBrand}
						{@const g = groups.find((x) => x.brand === activeFilterBrand)}
						{#if g}
							<div class="mb-2 flex items-center justify-between">
								<span class="text-xs text-gray-400">{g.series.length} 系列</span>
								<button
									type="button"
									class="text-primary-600 dark:text-primary-400 text-xs hover:underline"
									onclick={() => toggleBrandAll(g)}
								>
									{isBrandFullySelected(g) ? '取消全选' : '全选'}
								</button>
							</div>
							<div class="grid grid-cols-4 gap-1.5">
								{#each g.series as s (s.serie)}
									{@const serieMeta = getSerieMeta(g.brand, s.serie)}
									{@const selected = selectedSeries.has(serieKey(g.brand, s.serie))}
									<div
										role="button"
										tabindex="0"
										onclick={() => toggleSerie(g.brand, s.serie)}
										onkeydown={(e) => e.key === 'Enter' && toggleSerie(g.brand, s.serie)}
										title={serieMeta?.desc}
										class="group relative aspect-square w-full cursor-pointer overflow-hidden rounded-md bg-gray-100 shadow-sm transition-transform hover:scale-105 dark:bg-gray-800 {selected
											? 'ring-[3px] ring-green-500'
											: 'ring-1 ring-black/10 hover:ring-black/30 dark:ring-white/10 dark:hover:ring-white/30'}"
									>
										<img
											src={serieThumb(g.brand, s.serie)}
											alt=""
											class="h-full w-full object-cover"
											onerror={(e) => {
												if (e.currentTarget instanceof HTMLElement) {
													e.currentTarget.style.visibility = 'hidden';
												}
											}}
										/>
										<button
											type="button"
											aria-label={selected ? '取消选择系列' : '选择系列'}
											onclick={(e) => {
												e.stopPropagation();
												toggleSerie(g.brand, s.serie);
												e.currentTarget.blur();
											}}
											class="absolute top-0 right-0 h-6 w-6 scale-75 text-white opacity-0 transition-all duration-150 group-hover:scale-100 group-hover:opacity-100 focus:scale-100 focus:opacity-100 {selected
												? 'scale-100 opacity-100'
												: ''}"
										>
											<span
												class="absolute inset-0 [clip-path:polygon(100%_0,0_0,100%_100%)] {selected
													? 'bg-green-500'
													: 'bg-black/60 hover:bg-black/75'}"
											></span>
											<span class="absolute top-0.5 right-0.5">
												{#if selected}
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
												{serieMeta?.name ?? s.serie}
											</div>
											<div class="truncate text-[9px] leading-tight text-white/75">
												{s.paints.length} 型号
											</div>
										</div>
									</div>
								{/each}
							</div>
						{/if}
					{:else}
						<div class="flex h-full items-center justify-center text-center text-xs text-gray-400">
							将鼠标悬停在品牌上<br />查看系列
						</div>
					{/if}
				</div>
			</div>
		</Dropdown>

		<Button size="xs" color="alternative" class="gap-1">
			{ownedOnly ? '仅我拥有的' : '库存：全部'}
			<ChevronDown class="h-3 w-3" />
		</Button>
		<Dropdown placement="bottom-start" class="w-32 text-xs" bind:isOpen={ownedDropdownOpen}>
			<DropdownItem
				class={!ownedOnly ? 'bg-gray-100 dark:bg-gray-600' : ''}
				onclick={() => {
					ownedOnly = false;
					ownedDropdownOpen = false;
				}}
			>
				全部
			</DropdownItem>
			<DropdownItem
				class={ownedOnly ? 'bg-gray-100 dark:bg-gray-600' : ''}
				onclick={() => {
					ownedOnly = true;
					ownedDropdownOpen = false;
				}}
			>
				仅我拥有的
			</DropdownItem>
		</Dropdown>

		{#if selectedSeries.size > 0 || ownedOnly}
			<button
				type="button"
				class="text-primary-600 dark:text-primary-400 text-xs whitespace-nowrap hover:underline"
				onclick={clearFilters}
			>
				清除筛选
			</button>
		{/if}
	</div>

	<div class="mt-4 space-y-2 pb-4">
		<div class="flex items-center justify-between">
			<h3 class="text-sm font-semibold">查询结果</h3>
			<span class="text-xs text-gray-400">混色查询暂时禁用（性能优化中）</span>
		</div>
		{#each results as r, i (i)}
			<div
				class="flex items-center gap-3 rounded-lg border border-gray-200 p-3 dark:border-gray-700"
			>
				<div
					class="h-12 w-12 shrink-0 rounded-md shadow-inner"
					style="background-color: {floatRgbToCss(r.rgb)}"
				></div>
				<div class="min-w-0 flex-1">
					<div class="flex items-center gap-2">
						<span class="text-sm font-semibold">{similarity(r.delta_e).toFixed(0)}% 相似</span>
						<span class="text-xs text-gray-400">ΔE {r.delta_e.toFixed(2)}</span>
					</div>
					<div class="mt-1 flex flex-wrap gap-2">
						{#each r.portions as p}
							<div
								class="flex items-center gap-1.5 rounded border border-gray-200 px-1.5 py-0.5 dark:border-gray-700"
							>
								<div class="h-3.5 w-3.5 rounded-sm" style="background-color: {floatRgbToCss(p.rgb)}"
								></div>
								<span class="text-xs uppercase">{p.brand}/{p.code} {(p.t * 100).toFixed(0)}%</span>
							</div>
						{/each}
					</div>
				</div>
			</div>
		{:else}
			<div class="text-sm text-gray-400">没有匹配结果</div>
		{/each}
	</div>
</div>
