<script lang="ts">
	import { clamp } from '$lib/utils';
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
		return +v.toFixed(precision);
	};

	const handleInput = (e: Event & { currentTarget: HTMLInputElement }) => {
		oninput(+e.currentTarget.value);
	};
</script>

<div class="color-slider">
	<div class="slider-track relative">
		<input tabindex="-1" type="range" {min} {max} {step} {value} {style} oninput={handleInput} />
		<div class="custom-slider-handle" style="left: {left}%"></div>
	</div>
	<input
		class="text-sm! font-mono"
		type="number"
		{min}
		{max}
		{step}
		value={validate(value)}
		oninput={handleInput}
	/>
</div>
