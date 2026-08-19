<script lang="ts">
	import { Check, Plus } from '@lucide/svelte';
	import { SURFACE_BITS, type PaintInfo } from '$lib/paints.svelte';
	import { stock } from '$lib/stock.svelte';
	import { t } from '$lib/i18n.svelte';
	import { paintDesc } from '$lib/i18ndyn.svelte';
	import SwatchFx from './SwatchFx.svelte';

	interface Props {
		paint: PaintInfo;
		onSelect: (p: PaintInfo) => void;
	}

	let { paint, onSelect }: Props = $props();

	const inStock = $derived(stock.has(paint.id));
</script>

<div
	role="button"
	tabindex="0"
	onclick={() => onSelect(paint)}
	onkeydown={(e) => e.key === 'Enter' && onSelect(paint)}
	class="paint-card group relative aspect-square w-full cursor-pointer overflow-hidden rounded-md shadow-md transition-transform hover:scale-105 {inStock
		? 'ring-[3px] ring-primary-500'
		: 'ring-1 ring-black/10 hover:ring-black/30 dark:ring-white/10 dark:hover:ring-white/30'}"
	title={paintDesc(paint)}
>
	<SwatchFx {paint} />
	<button
		type="button"
		title={inStock ? t('stock.removeFromStock') : t('stock.addToStock')}
		onclick={(e) => {
			e.stopPropagation();
			stock.toggle(paint.id);
			e.currentTarget.blur();
		}}
		class="absolute top-0 right-0 z-200 h-6 w-6 scale-75 cursor-pointer text-white opacity-0 transition-all duration-150 group-hover:scale-100 group-hover:opacity-100 focus:scale-100 focus:opacity-100 {inStock
			? 'scale-100 opacity-100'
			: ''}"
	>
		<span
			class="absolute inset-0 [clip-path:polygon(100%_0,0_0,100%_100%)] {inStock
				? 'bg-primary-500'
				: 'bg-black/60 hover:bg-black/75'}"
		></span>
		<span class="absolute top-0.5 right-0.5">
			{#if inStock}
				<Check class="h-2.5 w-2.5" />
			{:else}
				<Plus class="h-2.5 w-2.5" />
			{/if}
		</span>
	</button>
	<div
		class="pointer-events-none absolute inset-x-0 bottom-0 z-200 bg-black/55 px-1 py-0.5 backdrop-blur-[1px]"
	>
		<div class="truncate text-[10px] leading-tight font-semibold text-white">
			{paint.code}
		</div>
		<div class="truncate text-[9px] leading-tight text-white/75">
			{paintDesc(paint)}
		</div>
	</div>
</div>
