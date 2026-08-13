<script lang="ts">
	import { type Oklch } from 'culori/fn';

	import Hsl from '$lib/components/Hsl.svelte';
	import Rgb from '$lib/components/Rgb.svelte';
	import {
		Box,
		Camera,
		ChevronDown,
		Cylinder,
		FlaskConical,
		Palette,
		Droplet,
		Pipette,
		ArrowLeftRight,
		RotateCcwClock,
		Paintbrush,
		Eclipse
	} from '@lucide/svelte';
	import { Button, Dropdown, Tooltip } from 'flowbite-svelte';
	import CameraPicker from '$lib/components/CameraPicker.svelte';
	import PanText from '$lib/components/PanText.svelte';
	import { getCatalog, floatRgbToCss, type PaintInfo, getPaintById } from '$lib/paints.svelte';
	import { baseLabels, mediumLabels, surfaceLabels } from '$lib/paintInfo';
	import PaintSearch from '$lib/components/PaintSearch.svelte';
	import ColorCode from '$lib/components/ColorCode.svelte';
	import PaintDetail from '$lib/components/PaintDetail.svelte';
	import DetailEmpty from '$lib/components/DetailEmpty.svelte';
	import { drawer } from '$lib/drawer.svelte';
	import { getBrandMeta, getSerieMeta, serieThumb } from '$lib/meta';
	import { clamp, similarity, toRgb, toOklch, toHwb, isMedia } from '$lib/utils.svelte';
	import Select from '$lib/components/Select.svelte';
	import TagSelect from '$lib/components/TagSelect.svelte';
	import TagButtonGroup from '$lib/components/TagButtonGroup.svelte';
	import SeriesPicker from '$lib/components/SeriesPicker.svelte';
	import { store, rt } from './search.svelte';
	import { t } from '$lib/i18n.svelte';
	import { untrack } from 'svelte';
	import { paintDesc } from '$lib/i18ndyn.svelte';
	import { isTauri } from '@tauri-apps/api/core';
	import Tag from '$lib/components/Tag.svelte';

	/** store.color（rgb int）与色板 oklch 互转 */
	const oklchToInt = (c: Oklch): number => {
		const { r, g, b } = toRgb(c);
		return (
			(clamp(Math.round(r * 255), 0, 255) << 16) |
			(clamp(Math.round(g * 255), 0, 255) << 8) |
			clamp(Math.round(b * 255), 0, 255)
		);
	};
	const intToOklch = (int: number): Oklch =>
		toOklch({
			mode: 'rgb',
			r: ((int >> 16) & 0xff) / 255,
			g: ((int >> 8) & 0xff) / 255,
			b: (int & 0xff) / 255
		});

	// 取色板初始颜色：store.color（localStorage 持久化 / 详情页调配设置），唯一事实源，不再读 URL
	let oklch: Oklch = $state(intToOklch(store.color));
	const rgb = $derived(toRgb(oklch));
	const hwb = $derived(toHwb(oklch));

	const hasEyeDropper = $derived('EyeDropper' in window);

	// 展示面板模式（store.source 持久化，刷新/重开按模式直接进入对应面板）：
	// 调色板 / 摄像机（仅 Tauri 应用内可用）/ 油漆（详情页「调配/查看全部」经 store.paintKey 带入）
	const hasCamera = $derived(
		isTauri() && typeof navigator !== 'undefined' && !!navigator.mediaDevices?.getUserMedia
	);
	// 摄像机模式仅 Tauri 内可用：持久化的 camera 模式在无相机环境降级为调色板
	$effect(() => {
		if (store.source === 'camera' && !untrack(() => hasCamera)) {
			store.source = 'palette';
		}
	});

	// 设备横屏（横屏时摄像机全屏覆盖右侧内容区）
	let isLandscape = $state(false);
	$effect(() => {
		const mq = window.matchMedia('(orientation: landscape)');
		isLandscape = mq.matches;
		const onChange = () => (isLandscape = mq.matches);
		mq.addEventListener('change', onChange);
		return () => mq.removeEventListener('change', onChange);
	});

	const eyedrop = () => {
		let EyeDropper = (window as any).EyeDropper;
		const eyeDropper = new EyeDropper();
		eyeDropper.open().then((result: any) => {
			const hex = parseInt(result.sRGBHex.slice(1), 16);
			const b = (hex & 0xff) / 255;
			const g = ((hex >> 8) & 0xff) / 255;
			const r = ((hex >> 16) & 0xff) / 255;
			oklch = toOklch({ mode: 'rgb', r, g, b });
		});
	};

	const catalog = getCatalog();

	// 油漆来源：store.paintKey（详情页「调配/查看全部」设置），唯一指定当前油漆
	const paintColor = $derived.by(() => {
		const key = store.paintKey;
		return key ? getPaintById(key) : null;
	});

	// 油漆信息标签（溶剂/漆面位拆解，与 PaintDetail 共享逻辑）
	// 油漆模式只读色值展示（跟随 store.color，切换 palette 后返回 paint 仍显示油漆色）
	const hexText = $derived(`#${store.color.toString(16).padStart(6, '0').toLowerCase()}`);
	// paint 面板搜索态：无油漆锚点（待选择）时直接显示搜索框
	let searching = $state(!store.paintKey);

	// 切回油漆模式时颜色复位为油漆色（palette 调色后再切回，颜色回到油漆原始色）
	$effect(() => {
		if (store.source === 'paint' && paintColor) {
			oklch = intToOklch(paintColor.rgb);
		}
	});

	// 桌面端右侧详情栏的选中油漆（手机端走 Drawer 底部卡片）
	let selectedPaint = $state<PaintInfo | null>(null);

	// 打开油漆详情：桌面填充右侧常驻详情栏（不离开列表），手机打开底部卡片
	const openDetail = (brand: string, code: string) => {
		const paint = getPaintById(`${brand}:${code}`);
		if (!paint) return;
		if (isMedia().sm) {
			selectedPaint = paint;
		} else {
			drawer.open({
				key: paint.id,
				component: PaintDetail,
				props: { paint, isStockPage: false }
			});
		}
	};

	/** 混合比例渐变条：按各组分比例生成横向 linear-gradient（色卡下方细细的一条） */
	const mixGradient = (portions: { t: number; rgb: [number, number, number] }[]): string => {
		const sum = portions.reduce((s, p) => s + p.t, 0) || 1;
		let acc = 0;
		const stops: string[] = [];
		for (const p of portions) {
			const start = (acc / sum) * 100;
			acc += p.t;
			const end = (acc / sum) * 100;
			const c = floatRgbToCss(p.rgb);
			stops.push(`${c} ${start}%`, `${c} ${end}%`);
		}
		return `linear-gradient(to right, ${stops.join(', ')})`;
	};

	let seriesOpen = $state(false);

	const isDefaultFilter = $derived(
		store.selectedSeries.size == 0 &&
			store.surfaceTypes.length == 0 &&
			store.baseTypes.length == 0 &&
			!store.searchScope &&
			store.mixingLimit == 0
	);

	const resetFilter = () => {
		store.reset();
	};

	// search 的薄封装，保持原有接口。底层走通用 wasm RPC 客户端：
	// 新请求会取消（terminate）仍在执行的旧请求。被取消的请求这里静默返回空数组，
	// 把取色板当前颜色写入模块级运行时状态并触发搜索；maybeSearch 内部比较
	// (color, filter, stock) 签名，无变化时跳过（切页回来不会重新搜索）。
	// ---- 数据流：store.color 是唯一事实源 ----
	// 色板 oklch → store.color（用户拖色板/取色）：单向。
	// Picker 拖拽期间绝不反向写 oklch（否则浮点往返造成滑条往复跳变），
	// 用 lastColor 记录已同步值，避免对 store.color 建立依赖（外部写入不被拉回）。
	let lastColor = store.color;
	// Picker 引起的 store.color 更新标记：当帧内 effect B 不反向同步 oklch
	let fromPicker = false;
	$effect(() => {
		oklch;
		const int = oklchToInt(oklch);
		if (Math.abs(int - lastColor) > 1) {
			lastColor = int;
			fromPicker = true;
			store.color = int;
			queueMicrotask(() => (fromPicker = false));
		}
	});

	// store.color → 持久化 + 触发搜索 + 色板跟随（仅外部来源更新时同步 oklch）
	let lastSyncedColor = store.color;
	$effect(() => {
		const c = store.color;
		if (!fromPicker) oklch = intToOklch(c);
		// 非油漆渠道（调色板/相机/滴管）更新了颜色 → 解除油漆锚点：
		// paint 面板退回「仅搜索栏」状态（油漆卡片不显示）；untrack 使本 effect 仍只依赖 color，
		// 且仅在颜色实际变化时执行（组件首次运行不清理）。
		if (c !== lastSyncedColor) {
			lastSyncedColor = c;
			if (untrack(() => store.source) !== 'paint') store.paintKey = null;
		}
		store.persist();
		rt.search();
	});

	// 展示面板模式（source/paintKey）变化 → 持久化，刷新/重开后按模式直接进入对应面板
	$effect(() => {
		store.source;
		store.paintKey;
		store.persist();
	});

	// 筛选条件（系列/漆面/溶剂/范围/混色/显示模式）变化 → 持久化 + 立即重搜
	$effect(() => {
		store.selectedSeries;
		store.surfaceTypes;
		store.baseTypes;
		store.searchScope;
		store.mixingLimit;
		store.model;
		store.persist();
		rt.search();
	});

	// 搜索范围不是「我的库存」时混色不可用，强制复位（替代原 Select 的 disabledValue 联动）
	$effect(() => {
		if (store.searchScope != 1 && store.mixingLimit != 0) {
			store.mixingLimit = 0;
		}
	});
</script>

{#snippet sourceSwitcher()}
	<div
		class="absolute top-1.5 right-1.5 z-10 flex flex-row overflow-hidden rounded-md bg-black/40 backdrop-blur-sm"
	>
		{#if isTauri()}
			<button
				type="button"
				class="cursor-pointer p-1.5 text-white transition-colors hover:bg-white/15 disabled:cursor-not-allowed disabled:opacity-40 {store.source ===
				'camera'
					? 'bg-white/25'
					: ''}"
				title={t('search.sourceCamera')}
				onclick={() => (store.source = 'camera')}
				disabled={!hasCamera}
			>
				<Camera class="size-4" />
			</button>
			<div class="w-px bg-white/25"></div>
		{/if}
		<button
			type="button"
			class="cursor-pointer p-1.5 text-white transition-colors hover:bg-white/15 {store.source ===
			'palette'
				? 'bg-white/25'
				: ''}"
			title={t('search.sourcePalette')}
			onclick={() => (store.source = 'palette')}
		>
			<Palette class="size-4" />
		</button>
		<div class="w-px bg-white/25"></div>
		<button
			type="button"
			class="cursor-pointer p-1.5 text-white transition-colors hover:bg-white/15 {store.source ===
			'paint'
				? 'bg-white/25'
				: ''}"
			title={t('search.sourcePaint')}
			onclick={() => (store.source = 'paint')}
		>
			<Droplet class="size-4" />
		</button>
	</div>
{/snippet}

{#snippet colorSwatch()}
	<div
		class="relative h-24 overflow-hidden rounded-xl border border-gray-700 bg-(--picker-color-srgb)"
	>
		{#if store.source === 'paint' && paintColor}
			<img
				src="/brands/{paintColor.brand}.png"
				alt=""
				class="absolute top-1.5 left-1.5 h-8 w-8 object-contain drop-shadow"
			/>
		{/if}
		{@render sourceSwitcher()}
		{#if store.source === 'palette'}
			{#if !isTauri() && hasEyeDropper}
				<button
					type="button"
					class="absolute right-1.5 bottom-1.5 cursor-pointer rounded-md bg-black/40 p-1.5 text-white backdrop-blur-sm transition-colors hover:bg-black/60"
					onclick={eyedrop}
				>
					<Pipette size="1rem" />
				</button>
			{/if}
		{/if}
	</div>
{/snippet}

{#snippet paintPanel(p: PaintInfo | null)}
	{#if p}
		{@const bases = baseLabels(p)}
		{@const surfaces = surfaceLabels(p)}
		{@const mediums = mediumLabels(p)}
		<div class="relative">
			<div class="rounded-xl border border-gray-200 p-3 sm:p-2 dark:border-gray-700">
				<div class="flex items-start gap-3">
					<div class="min-w-0 flex-1">
						<div class="flex min-w-0 items-baseline gap-2">
							<span class="shrink-0 text-2xl font-bold">{p.code}</span>
							<PanText class="text-lg font-bold text-gray-500 dark:text-gray-400">
								{paintDesc(p)}
							</PanText>
						</div>
						<PanText class="mt-0.5 text-xs text-gray-400">
							{getSerieMeta(p.brand, p.serie)?.name ?? p.serie} /
							{getBrandMeta(p.brand)?.name ?? p.brand}
						</PanText>
						{#if bases.length > 0 || surfaces.length > 0}
							<div class="mt-2 flex flex-wrap gap-1.5">
								{#each bases as l}
									<Tag>
										<FlaskConical class="size-3" />
										{l}
									</Tag>
								{/each}
								{#each surfaces as l}
									<Tag>
										{l}
									</Tag>
								{/each}
								{#each mediums as l}
									<Tag>
										{l}
									</Tag>
								{/each}
							</div>
						{/if}
					</div>

					<!-- 右侧：只读 hex + 切换按钮 -->
					<div class="flex shrink-0 flex-col items-end gap-2">
						<ColorCode
							re="^#([0-9a-fA-F]{6})$"
							text={hexText}
							readonly
							class="w-24"
							inputClass="text-left dark:bg-gray-800"
						/>
						<button
							type="button"
							class="flex h-9 w-9 cursor-pointer items-center justify-center rounded-full text-gray-500 transition-colors hover:bg-gray-100 hover:text-gray-700 dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-gray-200 {searching
								? 'bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-white'
								: ''}"
							title={t('gamut.change')}
							onclick={() => (searching = !searching)}
						>
							<ArrowLeftRight class="h-4 w-4" />
						</button>
					</div>
				</div>
			</div>

			{#if searching}
				<!-- 从卡片下方弹出的搜索框（文档流插入，选中/取消自动收回） -->
				<div class="mt-2">
					<PaintSearch
						onselect={(p2) => {
							store.paintKey = p2.id;
							searching = false;
						}}
						oncancel={() => (searching = false)}
					/>
				</div>
			{/if}
		</div>
	{:else}
		<!-- 无油漆锚点：直接显示搜索框 -->
		<PaintSearch
			onselect={(p2) => {
				store.paintKey = p2.id;
				searching = false;
			}}
		/>
	{/if}
{/snippet}

{#snippet colorPicker()}
	{#if store.source === 'paint'}
		<div class="grid gap-0 sm:auto-cols-[150px_1fr] sm:grid-flow-col sm:gap-3">
			{#if isMedia().sm}
				{@render colorSwatch()}
			{/if}
			<div class="min-w-45 sm:max-w-135">
				{@render paintPanel(paintColor)}
			</div>
		</div>
	{:else}
		{@const Picker = [Hsl, Rgb][store.model]}

		<div class="grid gap-3 sm:auto-cols-[150px_1fr] sm:grid-flow-col">
			<div>
				{#if isMedia().sm}
					<div class="mb-3">
						{@render colorSwatch()}
					</div>
				{/if}

				{#snippet hsl()}
					<span class="inline-flex items-center gap-1"><Cylinder class="size-4" />HSL</span>
				{/snippet}

				{#snippet rgb()}
					<span class="inline-flex items-center gap-1"><Box class="size-4" />RGB</span>
				{/snippet}

				<Select class="w-full" options={[hsl, rgb]} bind:value={store.model} />
			</div>

			<div class="min-w-45 sm:max-w-135">
				<Picker bind:oklch />
			</div>
		</div>
	{/if}
{/snippet}

{#snippet selectSeries()}
	<button
		type="button"
		onclick={() => {
			if (isMedia().sm) {
				// sm+：开关由 Dropdown(Popper) 的 mousedown toggle 接管，这里不做动作
			} else {
				// 手机端：用全局底部 Drawer（拖拽/遮罩/返回键关闭）
				if (drawer.view?.key === 'series') drawer.close();
				else drawer.open({ key: 'series', component: SeriesPicker, height: '100dvw' });
			}
		}}
		class="flex shrink-0 cursor-pointer items-center gap-1.5 rounded-md px-1.5 py-1 text-xs text-gray-600 transition-colors hover:bg-gray-200 dark:text-gray-300 dark:hover:bg-gray-700 {(
			isMedia().sm ? seriesOpen : drawer.view?.key === 'series'
		)
			? 'bg-gray-200 dark:bg-gray-700'
			: 'bg-gray-100 dark:bg-gray-800'}"
	>
		{t('search.series')}:
		{#if store.selectedSeries.size > 0}
			<span
				class="rounded-full bg-primary-500 px-1.5 text-[10px] leading-4 text-white dark:bg-primary-500 dark:text-white"
			>
				{store.selectedSeries.size}
			</span>
		{:else}
			<span class="text-gray-400 dark:text-gray-500">{t('search.any')}</span>
		{/if}
		<ChevronDown class="h-3 w-3" />
	</button>
	{#if isMedia().sm}
		<Dropdown
			bind:isOpen={seriesOpen}
			class="w-136 max-w-[calc(100vw-3rem)] overflow-hidden! p-0"
			placement="bottom-start"
		>
			<SeriesPicker />
		</Dropdown>
	{/if}
{/snippet}

<div
	class="color-provider relative flex h-full flex-col {isMedia().sm
		? 'overflow-hidden'
		: store.source === 'camera' && isLandscape
			? 'overflow-hidden'
			: 'overflow-y-auto'}"
	style="--slider-thumb-l: {oklch.l};
    --slider-thumb-c: {oklch.c};
    --slider-thumb-h: {oklch.h ?? 0};
    --slider-thumb-hue: {hwb.h ?? 0};
    --picker-color-srgb: rgb({rgb.r * 255} {rgb.g * 255} {rgb.b * 255});"
>
	{#if store.source === 'camera'}
		{#if isLandscape}
			<!-- 横屏：摄像机无圆角全屏覆盖右侧内容区，锁定滚动，挡住所有内容 -->
			<div class="fixed inset-y-0 right-0 left-0 z-40 flex flex-col bg-black sm:left-16">
				<CameraPicker
					fill
					onsample={(r, g, b) => {
						oklch = toOklch({ mode: 'rgb', r, g, b });
						store.source = 'palette';
					}}
				/>
				{@render sourceSwitcher()}
			</div>
		{:else}
			<div
				class="sticky top-0 z-20 shrink-0 bg-white px-6 pt-4 pb-4 shadow-sm dark:bg-gray-900 dark:shadow-black/30"
			>
				<div class="relative overflow-hidden rounded-xl border border-gray-700">
					<CameraPicker
						onsample={(r, g, b) => {
							oklch = toOklch({ mode: 'rgb', r, g, b });
							store.source = 'palette';
						}}
					/>
					{@render sourceSwitcher()}
				</div>
			</div>
		{/if}
	{:else}
		{#if !isMedia().sm}
			<div
				class="sticky top-0 z-20 shrink-0 bg-white px-6 pt-4 pb-2 shadow-sm dark:bg-gray-900 dark:shadow-black/30"
			>
				{@render colorSwatch()}
			</div>
			<div class="px-6 pt-2 pb-4">
				{@render colorPicker()}
			</div>
		{:else}
			<div class="w-full max-w-360 shrink-0 px-6 py-4">
				{@render colorPicker()}
			</div>
		{/if}
	{/if}

	<!-- sm+：容器为 flex-col 占满剩余高度（筛选行固定，结果区 flex-1 接管剩余）；<sm：px-6 原样 -->
	<div class={isMedia().sm ? 'mx-auto flex min-h-0 w-full flex-1 flex-col px-6' : 'px-6'}>
		<div class={isMedia().sm ? 'flex min-h-0 flex-1 overflow-hidden pb-4' : 'flex gap-6 pb-4'}>
			<div class={isMedia().sm ? 'min-w-0 flex-1 overflow-y-auto' : 'min-w-0 flex-1'}>
				<div class="mb-4 border-y border-gray-200 py-2 dark:border-gray-700">
					<div class="grid grid-flow-row gap-2">
						<div class="flex flex-auto flex-wrap items-center gap-2">
							<TagButtonGroup
								options={{ '0': t('search.market'), '1': t('search.myStock') }}
								value={String(store.searchScope)}
								onchange={(v) => (store.searchScope = Number(v))}
								class="py-1!"
							/>

							{@render selectSeries()}

							<Button
								color="secondary"
								disabled={isDefaultFilter}
								class="cursor-pointer p-0"
								onclick={resetFilter}
							>
								<Tag intereactive class="py-1!">
									<RotateCcwClock class="size-3" />
									{t('search.resetFilter')}
								</Tag>
							</Button>
						</div>

						<div class="grid grid-cols-[max-content_1fr] gap-x-3 gap-y-2">
							<span class="shrink-0 py-0.5 text-xs text-black dark:text-white">
								<Palette class="inline-block size-3" />
								{t('search.mixTitle')}
							</span>
							<div class="flex items-center gap-1">
								<TagButtonGroup
									disabled={store.searchScope != 1}
									options={{
										'0': t('search.mixOff'),
										'1': t('search.mix1'),
										'2': t('search.mix2')
									}}
									value={String(store.mixingLimit)}
									onchange={(v) => (store.mixingLimit = Number(v))}
								/>
								{#if store.searchScope != 1}
									<Tooltip placement="right" class="p-1 text-xs"
										>{t('search.mixScopeRequired')}</Tooltip
									>
								{/if}
							</div>

							<span class="shrink-0 py-0.5 text-xs text-black dark:text-white">
								<Eclipse class="inline-block size-3" />
								{t('search.surfaceTitle')}
							</span>
							<TagSelect
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
								bind:value={store.surfaceTypes}
							/>

							<span class="shrink-0 py-0.5 text-xs text-black dark:text-white">
								<FlaskConical class="inline-block size-3" />
								{t('search.baseTitle')}
							</span>
							<TagSelect
								options={{
									0: t('search.lacquer'),
									1: t('search.alcohol'),
									2: t('search.enamel'),
									3: t('search.water')
								}}
								bind:value={store.baseTypes}
							/>

							<span class="shrink-0 py-0.5 text-xs text-black dark:text-white">
								<Paintbrush class="inline-block size-3" />
								{t('search.mediumTitle')}
							</span>
							<TagSelect
								options={{
									Airbrush: t('search.medium.Airbrush'),
									Spray: t('search.medium.Spray'),
									Brush: t('search.medium.Brush'),
									Marker: t('search.medium.Marker'),
									Other: t('search.medium.Other')
								}}
								bind:value={store.mediumTypes}
							/>
						</div>
					</div>
				</div>

				<h3 class="mb-2 text-sm font-semibold">
					{t('search.results', { n: rt.results.length })}
				</h3>
				<div class="grid grid-cols-[repeat(auto-fill,minmax(150px,1fr))] gap-3 sm:pr-4">
					{#if rt.searching}
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
						{#each rt.results as r, i (i)}
							{@const isMix = r.portions.length > 1}
							<div
								class="flex flex-col overflow-hidden rounded-lg border border-gray-200 dark:border-gray-700"
							>
								<div class="h-16 w-full" style="background-color: {floatRgbToCss(r.rgb)}"></div>
								{#if isMix}
									<div
										class="h-1.5 w-full border-t border-gray-200 dark:border-gray-700/50"
										style="background: {mixGradient(r.portions)}"
									></div>
								{/if}
								<div class="flex flex-1 flex-col p-2">
									<div class="flex flex-col gap-1">
										{#each r.portions as p}
											<button
												type="button"
												onclick={() => openDetail(p.brand, p.code)}
												class="flex w-full cursor-pointer items-center gap-1.5 rounded-sm text-left text-[11px] hover:bg-gray-50 dark:hover:bg-gray-800"
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
														class="shrink-0 rounded-sm bg-gray-100 px-1.5 py-0.5 font-medium text-primary-700 dark:bg-gray-700 dark:text-primary-300"
													>
														{(p.t * 100).toFixed(0)}%
													</span>
												{/if}
											</button>
										{/each}
									</div>
									{#if !isMix}
										<div class="mt-0.5 pl-5.5 text-[10px] text-gray-500 dark:text-gray-400">
											<PanText>
												{paintDesc(r.portions[0])}
											</PanText>
										</div>
									{/if}
									<div
										class="mt-auto flex items-center justify-between pt-1.5 text-[10px] text-gray-400"
									>
										<span>ΔE {r.delta_e.toFixed(2)}</span>
										<span>{t('search.similarity', { n: similarity(r.delta_e).toFixed(0) })}</span>
									</div>
								</div>
							</div>
						{:else}
							<!-- no result -->
						{/each}
					{/if}
				</div>
			</div>

			{#if isMedia().sm}
				<!-- 右栏：详情面板（flex 交叉轴 stretch 撑满结果区高度，独立滚动） -->
				<aside
					class="w-[clamp(18rem,28vw,26rem)] shrink-0 overflow-y-auto border-t border-l border-gray-200 dark:border-gray-700"
				>
					{#if selectedPaint}
						{@const paint = selectedPaint}
						{#key paint.id}
							<div class="p-4">
								<PaintDetail {paint} isStockPage={false} />
							</div>
						{/key}
					{:else}
						<DetailEmpty hint={t('search.selectPaintHint')} />
					{/if}
				</aside>
			{/if}
		</div>
	</div>
</div>
