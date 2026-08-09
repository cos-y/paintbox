<script lang="ts">
	import { t } from '$lib/i18n.svelte';
	import { Check } from '@lucide/svelte';
	import { type Snippet } from 'svelte';

	interface Props {
		values: string[];
		options: { [_: string]: string };
		children: Snippet;
	}

	let { values = $bindable(), options, children }: Props = $props();
</script>

<div class="flex items-center justify-between gap-2 px-2 pt-1 pb-1.5">
	<span class="text-[11px] font-semibold text-gray-400 uppercase">{@render children()}</span>
	<button
		type="button"
		onclick={() => (values = [...Object.keys(options)])}
		class="cursor-pointer text-[10px] text-primary-500 hover:underline"
	>
		{values.length === 0 ? t('search.selectAll') : t('search.cancelAll')}
	</button>
</div>
{#each Object.entries(options) as [key, label]}
	<button
		type="button"
		onclick={() => {
			values = values.includes(key) ? values.filter((x) => x !== key) : [...values, key];
		}}
		class="flex w-full cursor-pointer items-center gap-2 rounded-md px-2 py-1 text-xs whitespace-nowrap text-gray-700 hover:bg-gray-50 dark:text-gray-200 dark:hover:bg-gray-700"
	>
		<span
			class="inline-flex h-3.5 w-3.5 shrink-0 items-center justify-center rounded border border-gray-300 {values.includes(
				key
			)
				? 'border-primary-500 bg-primary-500 text-white'
				: ''} dark:border-gray-600"
		>
			{#if values.includes(key)}
				<Check class="h-3 w-3" />
			{/if}
		</span>
		<span class={values.includes(key) ? 'font-semibold' : 'font-normal'}>{label}</span>
	</button>
{/each}
