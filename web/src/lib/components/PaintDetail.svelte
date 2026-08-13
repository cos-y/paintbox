<script lang="ts">
	import { Check, Plus, Mail, Palette, FlaskConical, ArrowRight } from '@lucide/svelte';
	import { goto } from '$app/navigation';
	import {
		paintId,
		rgbToHex,
		searchNearest,
		colorDiff,
		type PaintInfo,
		getPaintById,
		getPaintByIndex
	} from '$lib/paints.svelte';
	import { baseLabels, mediumLabels, surfaceLabels } from '$lib/paintInfo';
	import { stock } from '$lib/stock.svelte';
	import { store } from '../../routes/search/search.svelte.ts';
	import { getBrandMeta, getSerieMeta } from '$lib/meta';
	import { openExternal, similarity } from '$lib/utils.svelte';
	import { paintDesc } from '$lib/i18ndyn.svelte';
	import { t } from '$lib/i18n.svelte';
	import PanText from '$lib/components/PanText.svelte';
	import { findEquivIndices } from '$lib/equivs.svelte';
	import { Button } from 'flowbite-svelte';
	import Tag from './Tag.svelte';

	const FEEDBACK_EMAIL = 'zack.studios.15@gmail.com';

	interface Props {
		paint: PaintInfo;
		isStockPage: boolean;
	}

	let { paint, isStockPage }: Props = $props();

	const inStock = $derived(stock.has(paint.id));

	// ---- 官标等价（数据源品牌对照表，如 Gunze H9 <-> Gunze C9） ----
	const directEquivalences = $derived(
		findEquivIndices(paint.index)
			.map((i) => getPaintByIndex(i))
			.filter((p): p is PaintInfo => !!p)
	);

	// ---- 相近同色漆：按颜色距离查询，名字不一定相关 ----
	const colorEquivalences = $derived.by(() =>
		searchNearest(paint.rgb, { mix: 0, limit: 8 })
			.map((r) => getPaintById(paintId(r.portions[0])))
			.filter((p): p is PaintInfo => !!p && p.id !== paint.id)
	);

	// ---- 对比：点击等价/相近色方块，在原色下方拼接对比条（单选，再点一次取消） ----
	let compareCode = $state<string | null>(null);
	const comparePaint = $derived(compareCode ? getPaintById(compareCode) : null);
	const compareDeltaE = $derived(comparePaint ? colorDiff(paint.rgb, comparePaint.rgb) : null);
	const toggleCompare = (p: PaintInfo) => {
		compareCode = compareCode === p.id ? null : p.id;
	};

	// ---- 操作 ----
	// 直接 goto：layout 的 beforeNavigate 会 clearAnimated，关闭动画与页面切换并行进行
	const mixFromStock = () => {
		// 纯 store 驱动：store 是跨路由模块单例，search 页组件重建时直接读取；
		// 同路由软导航组件不重建，store.source 响应式更新同样生效
		store.color = paint.rgb;
		store.searchScope = 1;
		store.mixingLimit = 2;
		store.paintKey = paint.id;
		store.source = 'paint';
		// 普通 goto（不 replaceState）：layout 的 beforeNavigate 会 closeAnimated，
		// 关闭动画与页面切换并行；历史新增一条 /search/ 记录，listener 失效时
		// 系统 back 的 WebView.goBack 也能回退到 /stock/ 并触发同样的关闭路径
		goto('/search/', { noScroll: true });
	};

	const viewAllSimilar = () => {
		store.color = paint.rgb;
		store.searchScope = 0;
		store.mixingLimit = 0;
		store.paintKey = paint.id;
		store.source = 'paint';
		goto('/search/', { noScroll: true });
	};

	const reportIssue = () => {
		const subject = encodeURIComponent(`[paintbox] 数据反馈: ${paint.brand} ${paint.code}`);
		const body = encodeURIComponent(
			`品牌: ${paint.brand}\n色号: ${paint.code}\n名称: ${paintDesc(paint)}\n色值: #${paint.rgb
				.toString(16)
				.padStart(
					6,
					'0'
				)}\n漆面: ${surfaceLabels(paint).join('/') || '-'}\n溶剂: ${baseLabels(paint).join('/') || '-'}\n\n问题描述:\n`
		);
		const url = `mailto:${FEEDBACK_EMAIL}?subject=${subject}&body=${body}`;
		openExternal(url);
	};
</script>

{#snippet metaRow(paint: PaintInfo)}
	{@const bases = baseLabels(paint)}
	{@const surfaces = surfaceLabels(paint)}
	{@const mediums = mediumLabels(paint)}
	<div class="flex flex-wrap gap-1.5">
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
		<Button
			onclick={reportIssue}
			color="secondary"
			class="inline-flex cursor-pointer items-center gap-1 rounded-md px-1.5 py-0.5 text-[11px]! 
						text-gray-600 dark:text-gray-400"
		>
			<Mail class="size-3" />
			{t('stock.reportIssue')}
		</Button>
	</div>
{/snippet}

<div class="mx-auto max-w-xl space-y-4">
	<!-- 头部：色号 + 名称 + 库存开关 -->
	<div class="flex items-start justify-between gap-3">
		<div class="min-w-0">
			<div class="flex min-w-0 items-baseline gap-2">
				<span class="shrink-0 text-4xl font-bold">{paint.code}</span>
				<PanText class="text-xl font-bold text-gray-500 dark:text-gray-400">
					{paintDesc(paint)}
				</PanText>
			</div>
			<PanText class="mt-0.5 text-xs text-gray-400">
				{getSerieMeta(paint.brand, paint.serie)?.name ?? paint.serie} /
				{getBrandMeta(paint.brand)?.name ?? paint.brand}
			</PanText>
		</div>
		<button
			type="button"
			onclick={mixFromStock}
			title={t('stock.mixFromStock')}
			class="flex h-9 w-9 shrink-0 cursor-pointer items-center justify-center rounded-full bg-primary-500 text-white transition-colors hover:bg-primary-600"
		>
			<Palette class="h-5 w-5" />
		</button>
	</div>

	<!-- 色卡 + 对比条 -->
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
				onclick={() => toggleCompare(comparePaint)}
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
				<div class="flex-1 text-xs text-gray-400">
					{t('stock.similarity', { n: similarity(compareDeltaE).toFixed(0) })}
				</div>
			{/if}
			<div class="text-right">
				<div class="flex justify-end">
					<PanText class="font-bold text-gray-500 dark:text-gray-400">
						{paintDesc(comparePaint)}
					</PanText>
				</div>
				<div class="text-4xl font-bold">{comparePaint.code}</div>
			</div>
		</div>
	{/if}

	<!-- 元信息：溶剂 + 漆面 -->
	{@render metaRow(paint)}

	<!-- 操作：库存（仅 stock 页召唤时显示） -->
	{#if isStockPage}
		<div class="flex flex-wrap gap-2">
			<button
				type="button"
				onclick={() => stock.toggle(paint.id)}
				class="flex cursor-pointer items-center gap-1.5 rounded-lg px-3 py-1.5 text-sm font-medium transition-colors {inStock
					? 'bg-primary-500 text-white hover:bg-primary-600'
					: 'bg-gray-100 text-gray-500 hover:bg-gray-200 dark:bg-gray-700 dark:text-gray-300 dark:hover:bg-gray-600'}"
			>
				{#if inStock}
					<Check class="size-4" />
				{:else}
					<Plus class="size-4" />
				{/if}
				{inStock ? t('stock.removeFromStock') : t('stock.addToStock')}
			</button>
		</div>
	{/if}

	<!-- 官标等价 -->
	<div>
		<h3 class="mb-2 text-sm font-semibold text-gray-500 uppercase dark:text-gray-400">
			{t('stock.directEquiv')}
		</h3>
		<div class="flex flex-wrap gap-2">
			{#each directEquivalences as p (p.id)}
				<button
					type="button"
					onclick={() => toggleCompare(p)}
					class="flex items-center gap-2 rounded-lg border px-2 py-1 {compareCode === p.id
						? 'border-primary-500 bg-primary-50 dark:bg-gray-700'
						: 'border-gray-200 hover:bg-gray-50 dark:border-gray-700 dark:hover:bg-gray-800'}"
				>
					<div class="h-5 w-5 shrink-0 rounded" style="background-color: {rgbToHex(p.rgb)}"></div>
					<span class="text-xs uppercase">{p.brand}/{p.code}</span>
				</button>
			{:else}
				<div class="text-xs text-gray-400">{t('stock.noDirectEquiv')}</div>
			{/each}
		</div>
	</div>

	<!-- 相近同色漆 -->
	<div>
		<div class="mb-2 flex items-center justify-between">
			<h3 class="text-sm font-semibold text-gray-500 uppercase dark:text-gray-400">
				{t('stock.similarColors')}
			</h3>
			<button
				type="button"
				onclick={viewAllSimilar}
				class="flex cursor-pointer items-center gap-0.5 text-xs font-medium text-primary-600 hover:underline dark:text-primary-400"
			>
				{t('stock.viewAllSimilar')}
				<ArrowRight class="size-3" />
			</button>
		</div>
		<div class="flex flex-wrap gap-2">
			{#each colorEquivalences as p (p.id)}
				<button
					type="button"
					onclick={() => toggleCompare(p)}
					class="flex cursor-pointer items-center gap-2 rounded-lg border px-2 py-1 {compareCode ===
					p.id
						? 'border-primary-500 bg-primary-50 dark:bg-gray-700'
						: 'border-gray-200 hover:bg-gray-50 dark:border-gray-700 dark:hover:bg-gray-800'}"
				>
					<div class="h-5 w-5 shrink-0 rounded" style="background-color: {rgbToHex(p.rgb)}"></div>
					<span class="text-xs uppercase">{p.brand}/{p.code}</span>
				</button>
			{:else}
				<div class="text-xs text-gray-400">{t('stock.noSimilar')}</div>
			{/each}
		</div>
	</div>
</div>
