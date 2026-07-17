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

<div class="color-slider">
	<div class="slider-track relative">
		<input tabindex="-1" type="range" {min} {max} {step} {value} {style} oninput={handleInput} />
		<div class="custom-slider-handle" style="left: {left}%"></div>
	</div>
	<Input
		class="text-xs! font-mono p-1 text-right"
		type="number"
		{min}
		{max}
		{step}
		value={validate(value)}
		oninput={handleInput}
	/>
</div>
