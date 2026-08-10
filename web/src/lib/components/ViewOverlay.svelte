<script lang="ts">
	import { slide } from 'svelte/transition';
	import { viewStack } from '$lib/viewstack.svelte';
</script>

{#each viewStack.stack as v, i (v.key)}
	<div
		class="absolute inset-0 z-50 {i === viewStack.size - 1
			? 'bg-white dark:bg-gray-900'
			: 'pointer-events-none invisible'}"
		transition:slide={{ axis: 'x', duration: 250 }}
	>
		<svelte:component this={v.component} {...v.props} />
	</div>
{/each}
