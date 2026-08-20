<script lang="ts">
	import type { PaintInfo } from '$lib/paints.svelte';
	import SwatchFx from './SwatchFx.svelte';

	interface Props {
		paint: PaintInfo;
		class?: string;
	}

	const { paint, class: clz }: Props = $props();
</script>

<span
	class="relative h-4 w-4 shrink-0 overflow-hidden rounded-sm ring-1 ring-black/10 dark:ring-white/10 {clz}"
>
	<SwatchFx {paint} disablePaging />
</span>
