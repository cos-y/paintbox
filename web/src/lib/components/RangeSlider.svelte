<script lang="ts">
	import { clamp } from '$lib/utils.svelte';
	import { useSlider } from './slider';
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

	const trackRange = $derived(max - min);

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

	// 指针拖拽：pointerdown 时按指针离哪个 thumb 近决定拖哪个，之后只动它
	let active: 'low' | 'high' = $state('low');
	let trackEl = $state<HTMLElement | null>(null);
	const slider = $derived(useSlider({
		el: () => trackEl,
		min,
		max,
		step,
		start: (t) => {
			const el = trackEl;
			if (!el) return null;
			const w = el.getBoundingClientRect().width;
			const x = t * w;
			const lowC = (lowPercent / 100) * w;
			const highC = (highPercent / 100) * w;
			active = Math.abs(x - lowC) < Math.abs(x - highC) ? 'low' : 'high';
			return min + t * (max - min);
		},
		move: (t) => min + t * (max - min),
		oninput: (v) => (active === 'low' ? setLow(v) : setHigh(v))
	}));
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

		<!-- 交互层：接收指针事件；内部原生 input 仅保留键盘可达性，thumb 均 pointer-events: none -->
		<div
			class="slider-track absolute inset-0"
			role="group"
			bind:this={trackEl}
			onpointerdown={slider.pointerdown}
			onpointermove={slider.pointermove}
			onpointerup={slider.pointerup}
			onpointercancel={slider.pointercancel}
		>
			<input
				type="range"
				class="slider-input bg-transparent disabled:opacity-50"
				{min}
				{max}
				{step}
				value={low}
				oninput={(e) => setLow(+(e.target as HTMLInputElement).value)}
			/>
			<input
				type="range"
				class="slider-input bg-transparent disabled:opacity-50"
				{min}
				{max}
				{step}
				value={high}
				oninput={(e) => setHigh(+(e.target as HTMLInputElement).value)}
			/>
			<div class="slider-thumb" style="left: {lowPercent}%"></div>
			<div class="slider-thumb" style="left: {highPercent}%"></div>
		</div>
	</div>
</div>
