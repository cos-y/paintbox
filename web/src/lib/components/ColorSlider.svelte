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

	// ---- number text box ----
	// While the box is focused (editing) its text is never overwritten by prop
	// changes: `localText` mirrors the DOM value on every keystroke, so the prop
	// always equals what the user typed. On blur it is re-synced to the canonical
	// `value` (reverts illegal input, normalizes valid input).
	const format = (v: number) => String(+v.toFixed(precision));

	let localText = $state(format(value));
	let focused = $state(false);
	let error = $state(false);

	const handleSliderInput = (e: Event) => {
		const el = e.currentTarget! as HTMLInputElement;
		oninput(+el.value);
	};

	const handleNumberInput = (e: Event) => {
		const el = e.currentTarget! as HTMLInputElement;
		localText = el.value;
		const raw = el.value.trim();
		if (raw === '' || !isFinite(+raw) || +raw < min || +raw > max) {
			error = true;
			return;
		}
		error = false;
		oninput(+raw);
	};

	const handleBlur = () => {
		focused = false;
		error = false;
		localText = format(value);
	};

	$effect(() => {
		if (!focused && !error) {
			localText = format(value);
		}
	});
</script>

<div class="flex items-center gap-2 h-9 sm:h-6">
	<div class="relative w-full h-full flex-3">
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
			oninput={handleSliderInput}
		/>
		<div class="slider-thumb" style="left: {left}%"></div>
	</div>
	<div class="not-sm:flex-1 sm:w-24 h-full">
		<Input
			class={`text-xs! font-mono p-1 text-right h-full${
				error ? ' ring-2! ring-red-500! border-red-500!' : ''
			}`}
			type="number"
			{min}
			{max}
			{step}
			value={localText}
			oninput={handleNumberInput}
			onfocus={() => (focused = true)}
			onblur={handleBlur}
		/>
	</div>
</div>
