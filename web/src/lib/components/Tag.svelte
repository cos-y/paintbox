<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		children: Snippet;
		intereactive?: boolean;
		active?: boolean;
		class?: string;
	}

	const { children, intereactive, active, class: clz }: Props = $props();

	const activeClasses = $derived(
		`bg-primary-500 dark:bg-primary-500 dark:text-white ${
			intereactive ? 'hover:bg-primary-500 dark:hover:bg-primary-500' : ''
		}`
	);

	const inactiveClasses = $derived(
		`bg-gray-800 dark:bg-gray-800 dark:text-gray-300 ${
			intereactive ? 'hover:bg-gray-100 dark:hover:bg-gray-700' : ''
		}`
	);
</script>

<span
	class="flex items-center gap-1 rounded-md bg-gray-100 px-2 py-0.5
    text-xs text-gray-600 {[
		intereactive ? 'cursor-pointer transition-colors' : '',
		active ? activeClasses : inactiveClasses,
		clz
	].join(' ')}"
>
	{@render children()}
</span>
