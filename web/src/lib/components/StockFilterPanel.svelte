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
	<div class="min-w-0 scrollbar-none overflow-x-auto [&::-webkit-scrollbar]:hidden">
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
		{t('surface.title')}
	</span>
	<TagSelect
		options={{
			G: t('surface.G'),
			SG: t('surface.SG'),
			M: t('surface.M'),
			ME: t('surface.ME'),
			C: t('surface.C'),
			PA: t('surface.PA'),
			FL: t('surface.FL'),
			W: t('surface.W')
		}}
		bind:value={stockNav.surfSel}
	/>

	<span class="shrink-0 py-0.5 text-xs text-black dark:text-white">
		<FlaskConical class="inline-block size-3" />
		{t('base.title')}
	</span>
	<TagSelect
		options={{
			0: t('base.lacquer'),
			1: t('base.alcohol'),
			2: t('base.enamel'),
			3: t('base.water')
		}}
		bind:value={stockNav.baseSel}
	/>

	<span class="shrink-0 py-0.5 text-xs text-black dark:text-white">
		<Paintbrush class="inline-block size-3" />
		{t('medium.title')}
	</span>
	<TagSelect
		options={{
			Airbrush: t('medium.Airbrush'),
			Spray: t('medium.Spray'),
			Brush: t('medium.Brush'),
			Marker: t('medium.Marker')
			// Other: t('medium.Other')
		}}
		bind:value={stockNav.mediumSel}
	/>
</div>
