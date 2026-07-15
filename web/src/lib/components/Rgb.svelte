<script lang="ts">
	import { useMode, modeRgb, modeOklch, type Oklch } from 'culori/fn';
	import ColorSlider from './ColorSlider.svelte';
	import { Copy } from 'lucide-svelte';
	import { clamp } from '$lib/utils';
	import { tick } from 'svelte';
	import ColorCode from './ColorCode.svelte';

	interface Props {
		oklch: Oklch;
	}

	let { oklch = $bindable() }: Props = $props();

	const toRgb = useMode(modeRgb);
	const toOklch = useMode(modeOklch);

	const { r, g, b } = $derived(toRgb(oklch));

	const update = (r: number, g: number, b: number) => {
		oklch = toOklch({ mode: 'rgb', r, g, b });
	};

	const redStyle = $derived.by(() => {
		let s = 'background: linear-gradient(to right';
		for (let i = 0; i <= 255; ++i) {
			s += `, rgb(${i} var(--picker-green) var(--picker-blue))`;
		}
		s += ')';
		return s;
	});

	const greenStyle = $derived.by(() => {
		let s = 'background: linear-gradient(to right';
		for (let i = 0; i <= 255; ++i) {
			s += `, rgb(var(--picker-red) ${i} var(--picker-blue))`;
		}
		s += ')';
		return s;
	});

	const blueStyle = $derived.by(() => {
		let s = 'background: linear-gradient(to right';
		for (let i = 0; i <= 255; ++i) {
			s += `, rgb(var(--picker-red) var(--picker-green) ${i})`;
		}
		s += ')';
		return s;
	});

	const toText = (r: number, g: number, b: number) => {
		r = clamp(r, 0, 1);
		g = clamp(g, 0, 1);
		b = clamp(b, 0, 1);
		return `rgb(${(r * 255).toFixed(0)} ${(g * 255).toFixed(0)} ${(b * 255).toFixed(0)})`;
	};

	const handleInput = (r: number, g: number, b: number) => {
		update(r / 255, g / 255, b / 255);
	};
</script>

<div
	class="color-picker"
	style="--picker-red: {r * 255}; --picker-green: {g * 255}; --picker-blue: {b * 255}"
>
	<ColorSlider
		min={0}
		max={255}
		value={r * 255}
		oninput={(v) => update(v / 255, g, b)}
		style={redStyle}
	/>
	<ColorSlider
		min={0}
		max={255}
		value={g * 255}
		oninput={(v) => update(r, v / 255, b)}
		style={greenStyle}
	/>
	<ColorSlider
		min={0}
		max={255}
		value={b * 255}
		oninput={(v) => update(r, g, v / 255)}
		style={blueStyle}
	/>

	<ColorCode
		re="^rgb\(([\d.]+)\s*(?:,|\s)\s*([\d.]+)\s*(?:,|\s)\s*([\d.]+)\)$"
		text={toText(r, g, b)}
		oninput={handleInput}
	/>
</div>
