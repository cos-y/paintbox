<script lang="ts">
	import { ChevronDown } from '@lucide/svelte';
	import type { Snippet } from 'svelte';

	interface Props {
		isOpen?: boolean;
		title: string;
		children: Snippet<[]>;
		class?: string;
	}

	let { isOpen = $bindable(true), title, children, class: clz }: Props = $props();
</script>

<button
	type="button"
	class="border-theme flex w-full cursor-pointer items-center
		justify-between border-y px-3 py-2.5
		hover:bg-gray-50 dark:hover:bg-gray-800"
	onclick={() => (isOpen = !isOpen)}
>
	<span class="text-sm font-semibold text-gray-700 dark:text-gray-200">
		{title}
	</span>
	<ChevronDown
		class="h-4 w-4 text-gray-500 transition-transform {isOpen
			? 'rotate-180'
			: ''} dark:text-gray-400"
	/>
</button>

{#if isOpen}
	<div class={clz}>
		{@render children()}
	</div>
{/if}
