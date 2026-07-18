<script lang="ts">
	import { clamp } from '$lib/utils';
	import { Input } from 'flowbite-svelte';
	import './style.css';

	interface Props {
		min: number;
		max: number;
		precision?: number;
		value: number;
		oninput: (newValue: number) => void;
		style: string;
	}

	let { min, max, precision = 0, value, oninput, style }: Props = $props();

	const step = $derived(Math.pow(0.1, precision));

	const left = $derived.by(() => {
		return clamp(value / (max - min), 0, 1) * 100;
	});

	const validate = (v: number) => {
		v = clamp(v, min, max);
		v = +v.toFixed(precision);
		if (isNaN(v)) {
			v = 0;
		}
		return v;
	};

	const handleInput = (e: Event) => {
		const el = e.currentTarget! as HTMLInputElement;
		oninput(+el.value);
	};
</script>

<div class="flex items-center gap-2 h-9 sm:h-6">
	<div class="relative w-full h-full flex-4">
		<input
			class="no-handle m-0 w-full h-full
				outline-0 rounded-lg border border-gray-600
				select-none appearance-none pointer-auto
				touch-pan-y touch-pinch-zoom disabled:opacity-50"
			tabindex="-1"
			type="range"
			{min}
			{max}
			{step}
			{value}
			{style}
			oninput={handleInput}
		/>
		<div class="color-slider-handle" style="left: {left}%"></div>
	</div>
	<div class="not-sm:flex-1 sm:w-16 h-full">
		<Input
			class="text-xs! font-mono p-1 text-right h-full"
			type="number"
			{min}
			{max}
			{step}
			value={validate(value)}
			oninput={handleInput}
		/>
	</div>
</div>
