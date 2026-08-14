<script lang="ts">
	import { Badge } from 'flowbite-svelte';
	import { getCatalog } from '$lib/paints.svelte';
	import { getBrandMeta, getSerieMeta, serieThumb } from '$lib/meta';
	import { store } from '../../routes/search/search.svelte';
	import { t } from '$lib/i18n.svelte';

	const catalog = getCatalog();

	const serieKey = (brand: string, serie: string) => `${brand}::${serie}`;

	// 打开即默认选中第一个品牌，避免面板空白（原逻辑：seriesOpen 时未选品牌则取第一个）
	let activeFilterBrand = $state<string | null>(Object.keys(catalog)[0] ?? null);

	// 品牌 chip 条横向滚动的渐隐遮罩状态（仅手机端显示）
	let brandStrip = $state<HTMLElement | null>(null);

	// 两端渐隐宽度（滚动到头的一侧归零）。mask-image 渐变本身不可插值，
	// 用 @property 注册的长度变量驱动，CSS transition 平滑过渡（与 PanText 同款）。
	const FADE = '1.6em';

	const updateStrip = () => {
		const el = brandStrip;
		if (!el) return;
		const atStart = el.scrollLeft <= 0;
		const atEnd = el.scrollLeft + el.clientWidth >= el.scrollWidth - 1;
		el.style.setProperty('--fade-l', atStart ? '0px' : FADE);
		el.style.setProperty('--fade-r', atEnd ? '0px' : FADE);
	};

	$effect(() => {
		const el = brandStrip;
		if (!el) return;
		// 打开时 DOM 可能尚未完成布局（scrollWidth 为 0），用 rAF 延迟到下一帧再算
		const tick = () => requestAnimationFrame(updateStrip);
		tick();
		// chip 尺寸/容器尺寸变化（如字体加载）时重算遮罩状态
		const ro = new ResizeObserver(tick);
		ro.observe(el);
		for (const child of el.children) ro.observe(child);
		el.addEventListener('scroll', updateStrip, { passive: true });
		window.addEventListener('resize', tick);
		return () => {
			el.removeEventListener('scroll', updateStrip);
			ro.disconnect();
			window.removeEventListener('resize', tick);
		};
	});

	const toggleSerie = (brand: string, serie: string) => {
		const key = serieKey(brand, serie);
		const next = new Set(store.selectedSeries);
		if (next.has(key)) next.delete(key);
		else next.add(key);
		store.selectedSeries = next;
	};

	const isBrandFullySelected = (brand: string) =>
		Object.keys(catalog[brand]).every((s) => store.selectedSeries.has(serieKey(brand, s)));

	const selectedCountInBrand = (brand: string) =>
		Object.keys(catalog[brand]).filter((s) => store.selectedSeries.has(serieKey(brand, s))).length;

	const toggleBrandAll = (brand: string) => {
		const on = !isBrandFullySelected(brand);
		const next = new Set(store.selectedSeries);
		for (const s of Object.keys(catalog[brand])) {
			const key = serieKey(brand, s);
			if (on) next.add(key);
			else next.delete(key);
		}
		store.selectedSeries = next;
	};
</script>

<div class="flex h-full flex-col overflow-hidden sm:h-96 sm:max-h-[55vh] sm:flex-row">
	<!-- 品牌列表：手机为横向滚动 chip 条，桌面为纵向列表 -->
	<div class="border-theme w-full shrink-0 sm:w-40 sm:border-r">
		<div
			bind:this={brandStrip}
			onscroll={updateStrip}
			class="fade-edge mx-1 flex scrollbar-none gap-1.5 overflow-x-auto px-2 py-2 sm:mx-0 sm:flex-col sm:gap-0 sm:overflow-y-auto sm:p-0"
		>
			{#each Object.entries(catalog) as [brand, series]}
				{@const selectedCount = selectedCountInBrand(brand)}
				{@const name = getBrandMeta(brand)?.name ?? brand}
				<!-- 手机 chip -->
				<button
					type="button"
					onclick={() => (activeFilterBrand = brand)}
					title={name}
					class="flex shrink-0 cursor-pointer items-center gap-1.5 rounded-full border px-2.5 py-1.5 text-xs whitespace-nowrap sm:hidden {activeFilterBrand ===
					brand
						? 'border-primary-500 bg-primary-50 text-primary-700 ring-1 ring-primary-500 dark:border-primary-600 dark:bg-primary-900/50 dark:text-primary-200'
						: 'border-theme bg-transparent text-gray-900 hover:bg-gray-100 dark:bg-gray-800 dark:text-gray-400 dark:hover:bg-gray-700'}"
				>
					<img
						src="/brands/{brand}.png"
						alt=""
						class="h-5 w-5 shrink-0 rounded-full bg-white object-cover ring-1 ring-black/10"
					/>
					<span class="max-w-28 truncate">{name}</span>
					{#if selectedCount > 0}
						<Badge
							class="rounded-full bg-primary-500 text-white dark:bg-primary-500 dark:text-white"
						>
							{selectedCount}
						</Badge>
					{/if}
				</button>
				<!-- 桌面行 -->
				<button
					type="button"
					onmouseenter={() => (activeFilterBrand = brand)}
					onclick={() => (activeFilterBrand = brand)}
					title={name}
					class="hidden w-full cursor-pointer items-center gap-2 px-2.5 py-2 text-left text-sm text-gray-700 sm:flex dark:text-gray-200 {activeFilterBrand ===
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
	</div>
	<div class="min-h-0 flex-1 overflow-y-auto p-3">
		{#if activeFilterBrand}
			{@const series = catalog[activeFilterBrand]}
			{#if series}
				{@const brand = activeFilterBrand}
				<div class="mb-2 flex items-center justify-between">
					<span class="text-xs text-gray-500 dark:text-gray-400"
						>{t('search.seriesCount', { n: Object.keys(series).length })}</span
					>
					<button
						type="button"
						class="text-xs text-primary-500 hover:underline dark:text-primary-400"
						onclick={() => toggleBrandAll(brand)}
					>
						{isBrandFullySelected(brand) ? t('search.resetFilter') : t('search.selectAll')}
					</button>
				</div>
				<div class="grid grid-cols-4 gap-2.5">
					{#each Object.entries(series) as [serie, paints]}
						{@const serieMeta = getSerieMeta(brand, serie)}
						{@const selected = store.selectedSeries.has(serieKey(brand, serie))}
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
							<div class="absolute inset-x-0 bottom-0 bg-black/55 px-1 py-0.5 backdrop-blur-[1px]">
								<div class="truncate text-[10px] leading-tight font-semibold text-white">
									{serieMeta?.name ?? serie}
								</div>
								<div class="truncate text-[9px] leading-tight text-white/75">
									{t('search.paintsCount', { n: paints.length })}
								</div>
							</div>
						</div>
					{/each}
				</div>
			{/if}
		{:else}
			<div
				class="flex h-full items-center justify-center text-center text-xs text-gray-500 dark:text-gray-400"
			>
				{@html t('search.hoverBrandHint')}
			</div>
		{/if}
	</div>
</div>

<style>
	/* 两端渐隐驱动变量：长度可插值，transition 平滑（@property 需要全局注册，
	   Svelte scoped 编译会原样保留 at-rule） */
	@property --fade-l {
		syntax: '<length>';
		inherits: false;
		initial-value: 0px;
	}
	@property --fade-r {
		syntax: '<length>';
		inherits: false;
		initial-value: 0px;
	}
	/* 隐藏横向滚动条（Firefox + WebKit） */
	.scrollbar-none {
		scrollbar-width: none;
	}
	.scrollbar-none::-webkit-scrollbar {
		display: none;
	}
	/* 左右两端渐隐：没到头的一侧淡出内容，指示还有内容可拖/可滚；
	   桌面纵向列表用媒体查询关闭 mask（原实现也是手机专用） */
	.fade-edge {
		transition:
			--fade-l 0.25s ease,
			--fade-r 0.25s ease;
		mask-image: linear-gradient(
			to right,
			transparent 0,
			black var(--fade-l),
			black calc(100% - var(--fade-r)),
			transparent 100%
		);
		-webkit-mask-image: linear-gradient(
			to right,
			transparent 0,
			black var(--fade-l),
			black calc(100% - var(--fade-r)),
			transparent 100%
		);
	}
	@media (min-width: 640px) {
		.fade-edge {
			mask-image: none;
			-webkit-mask-image: none;
		}
	}
</style>
