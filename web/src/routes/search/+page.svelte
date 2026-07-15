<script lang="ts">
	import { useMode, modeHwb, modeRgb, modeOklch, type Oklch } from 'culori/fn';

	import Hsl from '$lib/components/Hsl.svelte';
	import Rgb from '$lib/components/Rgb.svelte';
	import { Box, ChevronUp, Cylinder, Pipette } from 'lucide-svelte';
	import {
		Button,
		Dropdown,
		DropdownItem,
		Accordion,
		AccordionItem,
		Checkbox,
		Toggle
	} from 'flowbite-svelte';
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

	<div class="mt-6 space-y-3 border-y border-gray-200 py-4 dark:border-gray-700">
		<div class="flex items-center justify-between">
			<h3 class="text-sm font-semibold">过滤器</h3>
			<button
				type="button"
				class="text-primary-600 dark:text-primary-400 text-xs hover:underline"
				onclick={clearFilters}
			>
				清除筛选
			</button>
		</div>

		<div class="flex flex-wrap items-center gap-6">
			<Toggle bind:checked={ownedOnly}>仅查询我拥有的油漆</Toggle>
			<span class="text-xs text-gray-400">
				混色查询暂时禁用（性能优化中），当前仅返回单一油漆的匹配结果
			</span>
		</div>

		<div>
			<div class="mb-1 text-xs text-gray-500 dark:text-gray-400">
				按系列筛选（不选则不限制）{selectedSeries.size > 0 ? `· 已选 ${selectedSeries.size}` : ''}
			</div>
			<Accordion class="max-h-64 overflow-y-auto">
				{#each groups as g (g.brand)}
					<AccordionItem>
						{#snippet header()}
							<div class="flex w-full items-center justify-between pr-2">
								<span class="uppercase">{g.brand}</span>
								<span class="text-xs text-gray-400">{g.series.length} 系列</span>
							</div>
						{/snippet}
						<div class="mb-2 flex justify-end">
							<button
								type="button"
								class="text-primary-600 dark:text-primary-400 text-xs hover:underline"
								onclick={() => toggleBrandAll(g)}
							>
								{isBrandFullySelected(g) ? '取消全选' : '全选'}
							</button>
						</div>
						<div class="grid grid-cols-2 gap-1 sm:grid-cols-3">
							{#each g.series as s (s.serie)}
								<Checkbox
									checked={selectedSeries.has(serieKey(g.brand, s.serie))}
									onchange={() => toggleSerie(g.brand, s.serie)}
								>
									{s.serie} <span class="text-gray-400">({s.paints.length})</span>
								</Checkbox>
							{/each}
						</div>
					</AccordionItem>
				{/each}
			</Accordion>
		</div>
	</div>

	<div class="mt-4 space-y-2 pb-4">
		<h3 class="text-sm font-semibold">查询结果</h3>
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
