<script lang="ts">
	import { useMode, modeHwb, modeRgb, modeOklch, modeHsl, type Oklch } from 'culori/fn';

	import Hsl from '$lib/components/Hsl.svelte';
	import Rgb from '$lib/components/Rgb.svelte';
	import { Box, ChevronDown, Cylinder, Pipette, Funnel } from '@lucide/svelte';
	import { Badge, Button, Dropdown } from 'flowbite-svelte';
	import { listPaints, getCatalog, paintId, floatRgbToCss, type SearchResult } from '$lib/paints';
	import { searchAsync } from '$lib/searchClient';
	import { stock } from '$lib/stock.svelte';
	import { getBrandMeta, getSerieMeta, serieThumb } from '$lib/meta';
	import { clamp, similarity } from '$lib/utils';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import MultiSelect from '$lib/components/MultiSelect.svelte';
	import Select from '$lib/components/Select.svelte';

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

	let model = $state(0);

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
	const catalog = getCatalog(allPaints);
	const paintByKey = new Map(allPaints.map((p) => [`${p.brand}:${p.code}`, p]));
	const stockLink = (brand: string, code: string) => {
		const paint = paintByKey.get(`${brand}:${code}`);
		const params = new URLSearchParams({ brand });
		if (paint) params.set('serie', paint.serie);
		params.set('code', code);
		return `/stock?${params.toString()}`;
	};

	// filter params
	let surfaceTypes = $state([]);
	let baseTypes = $state([]);
	let searchScope = $state(0);
	let mixingLimit = $state(0);

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
	let activeFilterBrand: string | null = $state(null);

	const toggleSerie = (brand: string, serie: string) => {
		const key = serieKey(brand, serie);
		const next = new Set(selectedSeries);
		if (next.has(key)) next.delete(key);
		else next.add(key);
		selectedSeries = next;
	};

	const isBrandFullySelected = (brand: string) =>
		Object.keys(catalog[brand]).every((s) => selectedSeries.has(serieKey(brand, s)));

	const selectedCountInBrand = (brand: string) =>
		Object.keys(catalog[brand]).filter((s) => selectedSeries.has(serieKey(brand, s))).length;

	const toggleBrandAll = (brand: string) => {
		const on = !isBrandFullySelected(brand);
		const next = new Set(selectedSeries);
		for (const s of Object.keys(catalog[brand])) {
			const key = serieKey(brand, s);
			if (on) next.add(key);
			else next.delete(key);
		}
		selectedSeries = next;
	};

	const isDefaultFilter = $derived(
		selectedSeries.size == 0 &&
			surfaceTypes.length == 0 &&
			baseTypes.length == 0 &&
			!searchScope &&
			mixingLimit == 0
	);

	const resetFilter = () => {
		selectedSeries = new Set();
		surfaceTypes = [];
		baseTypes = [];
		searchScope = 0;
		mixingLimit = 0;
	};

	let results: SearchResult[] = $state([]);
	let searching = $state(false);
	let searchSeq = 0;

	$effect(() => {
		const targetRgb = rgbInt;
		const seq = ++searchSeq;
		searching = true;

		const series = [...selectedSeries].map((key) => {
			const [brand, serie] = key.split('::');
			return [brand, serie];
		});
		const all =
			// FIXME: optimize performance
			searchScope == 0
				? undefined
				: allPaints.filter((p) => stock.has(paintId(p))).map((p) => p.index);
		const opts = {
			series,
			all,
			surfaces: [...surfaceTypes] as string[],
			bases: baseTypes.map((x) => +x),
			mix: mixingLimit,
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
</script>

{#snippet colorPicker()}
	{@const Picker = [Hsl, Rgb][model]}

	<div>
		<div
			class="relative overflow-hidden h-24 mb-3 rounded-xl border border-gray-700 bg-(--picker-color-srgb)"
		>
			<button
				type="button"
				class="absolute right-1.5 bottom-1.5 rounded-md bg-black/40 p-1.5 text-white backdrop-blur-sm transition-colors hover:bg-black/60"
				onclick={eyedrop}
			>
				<Pipette size="1rem" />
			</button>
		</div>

		{#snippet hsl()}
			<span class="inline-flex items-center gap-1"><Cylinder class="size-4" />HSL</span>
		{/snippet}

		{#snippet rgb()}
			<span class="inline-flex items-center gap-1"><Box class="size-4" />RGB</span>
		{/snippet}

		<Select class="w-full" options={[hsl, rgb]} bind:value={model} />
	</div>

	<div class="min-w-45 sm:max-w-135">
		<Picker bind:oklch />
	</div>
{/snippet}

{#snippet selectSeries()}
	<Button size="xs" color="alternative" class="cursor-pointer relative gap-1 justify-start w-32">
		Series:
		{#if selectedSeries.size > 0}
			<Badge
				class="absolute pl-1.5 pr-1.5 text-xs top-1.5 right-7 rounded-full bg-primary-500 dark:bg-primary-500 dark:text-white"
			>
				{selectedSeries.size}
			</Badge>
		{:else}
			Any
		{/if}
		<ChevronDown class="h-3 w-3 ms-auto" />
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
						class="cursor-pointer flex w-full items-center gap-2 px-2.5 py-2 text-left text-sm text-gray-700 dark:text-gray-200 {activeFilterBrand ===
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
							<span class="text-xs text-gray-400">{Object.keys(series).length} series</span>
							<button
								type="button"
								class="text-primary-500 dark:text-primary-400 text-xs hover:underline"
								onclick={() => toggleBrandAll(brand)}
							>
								{isBrandFullySelected(brand) ? 'Cancel All' : 'Select All'}
							</button>
						</div>
						<div class="grid grid-cols-4 gap-2.5">
							{#each Object.entries(series) as [serie, paints]}
								{@const serieMeta = getSerieMeta(brand, serie)}
								{@const selected = selectedSeries.has(serieKey(brand, serie))}
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
											{paints.length} models
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
{/snippet}

<div class="flex h-full flex-col overflow-y-auto p-4">
	<div
		class="color-picker-root grid gap-3 sm:grid-flow-col sm:auto-cols-[125px_1fr]"
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
		{@render colorPicker()}
	</div>

	<div
		class="mt-4 flex flex-wrap items-center gap-2 border-y border-gray-200 py-2 dark:border-gray-700"
	>
		<span
			class="flex items-center gap-1 text-xs whitespace-nowrap text-gray-500 dark:text-gray-400"
		>
			<Funnel class="h-4 w-4" />
		</span>

		{@render selectSeries()}

		<MultiSelect
			tooltip="surface type"
			class="w-36 text-xs"
			options={{
				G: 'Gloss',
				SG: 'Semi-Gloss',
				M: 'Flat',
				ME: 'Metallic',
				C: 'Clear',
				PA: 'Mica',
				FL: 'Fluorescence',
				W: 'Weathering'
			}}
			title="Surface"
			bind:value={surfaceTypes}
		/>

		<MultiSelect
			tooltip="solvent base type"
			class="w-28 text-xs"
			options={{
				0: 'Lacquer',
				1: 'Alcohol',
				2: 'Enamel',
				3: 'Water'
			}}
			title="Base"
			bind:value={baseTypes}
		/>

		<Select
			tooltip="search scope"
			class="w-28 text-xs"
			options={['Market', 'My Stock']}
			bind:value={searchScope}
		/>

		<Select
			tooltip="mixing"
			class="w-28 text-xs"
			options={['Mix Off', 'Mix-1', 'Mix-2']}
			bind:value={mixingLimit}
			disabled={searchScope != 1}
			disabledValue={0}
			disabledTooltip={'mixing requires search scope: my stock'}
		/>

		{#if !isDefaultFilter}
			<button
				type="button"
				class="text-primary-500 dark:text-primary-400 text-xs whitespace-nowrap hover:underline"
				onclick={() => {
					resetFilter();
				}}
			>
				Reset Filter
			</button>
		{/if}
	</div>

	<div class="mt-4 pb-4">
		<h3 class="mb-2 text-sm font-semibold">{results.length} Results</h3>
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
					<!-- no result -->
				{/each}
			{/if}
		</div>
	</div>
</div>
