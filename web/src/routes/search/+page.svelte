<script lang="ts">
	import { useMode, modeHwb, modeRgb, modeOklch, modeHsl, type Oklch } from 'culori/fn';

	import Hsl from '$lib/components/Hsl.svelte';
	import Rgb from '$lib/components/Rgb.svelte';
	import { Box, ChevronUp, ChevronDown, Cylinder, Pipette, Check, Plus, Filter } from 'lucide-svelte';
	import { Button, Dropdown, DropdownItem } from 'flowbite-svelte';
	import {
		listPaints,
		groupPaints,
		paintId,
		floatRgbToCss,
		type BrandGroup,
		type FilterOptions,
		type SearchResult
	} from '$lib/paints';
	import { searchAsync } from '$lib/searchClient';
	import { stock } from '$lib/stock.svelte';
	import { getBrandMeta, getSerieMeta, serieThumb } from '$lib/meta';
	import { clamp, similarity } from '$lib/utils';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';

	useMode(modeHsl);
	const toHwb = useMode(modeHwb);
	const toRgb = useMode(modeRgb);
	const toOklch = useMode(modeOklch);

	let oklch: Oklch = $state(toOklch({ mode: 'hsl', h: 220, s: 0.714, l: 0.439 }));
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

	let selectedSeries: Set<string> = $state(new Set());
	let ownedOnly = $state(false);
	let ownedDropdownOpen = $state(false);
	let activeFilterBrand: string | null = $state(null);
	let maxMix = $state(0);
	let maxMixDropdownOpen = $state(false);

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
	let searching = $state(false);
	let searchSeq = 0;

	$effect(() => {
		const targetRgb = rgbInt;
		const filter = filterOptions;
		const mix = maxMix;
		const seq = ++searchSeq;
		searching = true;
		const handle = setTimeout(async () => {
			const r = await searchAsync(targetRgb, mix, 12, filter);
			if (seq === searchSeq) {
				results = r;
				searching = false;
			}
		}, 150);
		return () => clearTimeout(handle);
	});
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
					<button
						type="button"
						aria-label="取色器"
						class="absolute right-1.5 bottom-1.5 rounded-md bg-black/40 p-1.5 text-white backdrop-blur-sm transition-colors hover:bg-black/60"
						onclick={eyedrop}
					>
						<Pipette size="1rem" />
					</button>
				</div>

				<div>
					<Button size="xs" color="alternative" class="w-full" aria-label="Choose color model">
						<Icon class="size-4" />
						{name}
						<ChevronUp class="ms-auto h-3 w-3" />
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

	<div
		class="mt-4 flex flex-wrap items-center gap-2 border-y border-gray-200 py-2 dark:border-gray-700"
	>
		<span
			class="flex items-center gap-1 text-xs whitespace-nowrap text-gray-500 dark:text-gray-400"
		>
			<Filter class="h-3.5 w-3.5" />
			{results.length} 个结果
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

		<Button size="xs" color="alternative" class="gap-1">
			最大混色：{maxMix}
			<ChevronDown class="h-3 w-3" />
		</Button>
		<Dropdown placement="bottom-start" class="w-28 text-xs" bind:isOpen={maxMixDropdownOpen}>
			{#each [0, 1, 2] as n}
				<DropdownItem
					class={maxMix === n ? 'bg-gray-100 dark:bg-gray-600' : ''}
					onclick={() => {
						maxMix = n;
						maxMixDropdownOpen = false;
					}}
				>
					{n}
				</DropdownItem>
			{/each}
		</Dropdown>

		{#if selectedSeries.size > 0 || ownedOnly || maxMix > 0}
			<button
				type="button"
				class="text-primary-600 dark:text-primary-400 text-xs whitespace-nowrap hover:underline"
				onclick={() => {
					clearFilters();
					maxMix = 0;
				}}
			>
				清除筛选
			</button>
		{/if}
	</div>

	<div class="mt-4 pb-4">
		<h3 class="mb-2 text-sm font-semibold">查询结果</h3>
		<div class="grid grid-cols-[repeat(auto-fill,minmax(170px,1fr))] gap-3">
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
												class="text-primary-700 dark:text-primary-300 shrink-0 rounded-sm bg-gray-100 px-1.5 py-0.5 font-medium dark:bg-gray-700"
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
								<span>{similarity(r.delta_e).toFixed(0)}% 相似</span>
							</div>
						</div>
					</div>
				{:else}
					<div class="text-sm text-gray-400">没有匹配结果</div>
				{/each}
			{/if}
		</div>
	</div>
</div>
