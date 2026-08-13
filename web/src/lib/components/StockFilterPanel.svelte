<script lang="ts">
	import TagSelect from './TagSelect.svelte';
	import { stockNav } from '$lib/stocknav.svelte';
	import { t } from '$lib/i18n.svelte';
	import { ArrowDownWideNarrow, Eclipse, FlaskConical, Paintbrush } from '@lucide/svelte';
	import TagButtonGroup from './TagButtonGroup.svelte';
</script>

<!-- 库存页筛选面板：PC 固定显示在标题栏正下方（第 1、2 列上方），手机放进全局 Drawer -->
<div class="grid grid-cols-[max-content_1fr] gap-x-3 gap-y-2">
	<span class="shrink-0 py-0.5 text-xs text-black dark:text-white">
		<ArrowDownWideNarrow class="inline-block size-3" />
		{t('stock.sortTitle')}
	</span>
	<div class="min-w-0 overflow-x-auto [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
		<TagButtonGroup
			options={{
				0: t('stock.sortCode'),
				1: t('stock.sortHue'),
				2: t('stock.sortSat'),
				3: t('stock.sortLight'),
				4: t('stock.sortStock')
			}}
			value={String(stockNav.sortKey)}
			onchange={(v) => (stockNav.sortKey = +v)}
		/>
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
		bind:value={stockNav.surfSel}
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
		bind:value={stockNav.baseSel}
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
		bind:value={stockNav.mediumSel}
	/>
</div>
