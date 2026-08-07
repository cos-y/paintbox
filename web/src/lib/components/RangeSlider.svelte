<script lang="ts">
	import { clamp } from '$lib/utils.svelte';
	import './style.css';

	interface Props {
		min: number;
		max: number;
		step?: number;
		minDist?: number;
		/** [low, high] */
		value: [number, number];
		/** CSS gradient string for the active track segment, e.g. "black, white" for L axis */
		gradient: [string, string];
	}

	let { min, max, step = 1, minDist = 1, value = $bindable(), gradient }: Props = $props();

	let low = $derived(value[0]);
	let high = $derived(value[1]);

	const trackRange = max - min;

	const lowPercent = $derived(((low - min) / trackRange) * 100);
	const highPercent = $derived(((high - min) / trackRange) * 100);

	function setLow(v: number) {
		if (v > high - minDist) {
			// push the right thumb along, capped at max
			const newHigh = Math.min(v + minDist, max);
			value = [Math.min(v, newHigh - minDist), newHigh];
		} else {
			value = [clamp(v, min, high - minDist), high];
		}
	}

	function setHigh(v: number) {
		if (v < low + minDist) {
			// push the left thumb along, capped at min
			const newLow = Math.max(v - minDist, min);
			value = [newLow, Math.max(v, newLow + minDist)];
		} else {
			value = [low, clamp(v, low + minDist, max)];
		}
	}

	// track background: gradient only visible between the two thumbs
	const trackBg = $derived(``);
</script>

<div
	class="range-slider flex items-center gap-2"
	style="
	--picker-track-box-shadow: inset 0 1px 2px rgba(0,0,0,.35);
	--picker-text-shadow: none;
	--a: {lowPercent}%;
	--b: {highPercent}%;
	--color-a: {gradient[0]};
	--color-b: {gradient[1]};"
>
	<div class="relative h-9 w-full flex-1 sm:h-6">
		<!-- track background -->
		<div class="range-slider-track absolute inset-0 rounded-lg"></div>

		<!-- low thumb -->
		<div class="range-slider-input absolute inset-0">
			<input
				type="range"
				class="no-handle pointer-auto m-0 h-full
					w-full touch-pan-y
					touch-pinch-zoom appearance-none bg-transparent
					outline-0 select-none disabled:opacity-50"
				{min}
				{max}
				{step}
				value={low}
				oninput={(e) => setLow(+(e.target as HTMLInputElement).value)}
			/>
			<div class="slider-thumb" style="left: {lowPercent}%"></div>
		</div>

		<!-- high thumb -->
		<div class="range-slider-input absolute inset-0">
			<input
				type="range"
				class="no-handle pointer-auto m-0 h-full
					w-full touch-pan-y
					touch-pinch-zoom appearance-none bg-transparent
					outline-0 select-none disabled:opacity-50"
				{min}
				{max}
				{step}
				value={high}
				oninput={(e) => setHigh(+(e.target as HTMLInputElement).value)}
			/>
			<div class="slider-thumb" style="left: {highPercent}%"></div>
		</div>
	</div>
</div>
