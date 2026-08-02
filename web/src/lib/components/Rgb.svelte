<script lang="ts">
	import { useMode, modeRgb, modeOklch, type Oklch } from 'culori/fn';
	import ColorSlider from './ColorSlider.svelte';
	import { clamp, hexToRgb } from '$lib/utils';
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

	const toHex = (r: number, g: number, b: number) => {
		r = clamp(r, 0, 1);
		g = clamp(g, 0, 1);
		b = clamp(b, 0, 1);
		const [R, G, B] = [r, g, b].map((x) => clamp(Math.round(x * 255), 0, 255));
		return `#${R.toString(16).padStart(2, '0')}${G.toString(16).padStart(2, '0')}${B.toString(16).padStart(2, '0')}`;
	};
</script>

<div
	class="color-picker"
	style="--picker-red: {r * 255}; 
		--picker-green: {g * 255}; 
		--picker-blue: {b * 255};"
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

	<div class="flex items-center gap-2 h-9">
		<ColorCode
			re="^rgb\(([\d.]+)\s*(?:,|\s)\s*([\d.]+)\s*(?:,|\s)\s*([\d.]+)\)$"
			text={toText(r, g, b)}
			oninput={(r, g, b) => update(+r / 255, +g / 255, +b / 255)}
			class="flex-3"
		/>

		<ColorCode
			re={`^#([0-9a-fA-F]{3}|[0-9a-fA-F]{6})$`}
			text={toHex(r, g, b)}
			class="not-sm:flex-1 sm:w-24 h-full"
			textAlign="left"
			oninput={(hex) => {
				const rgb = hexToRgb(hex);
				if (rgb) {
					update(rgb[0], rgb[1], rgb[2]);
				}
			}}
		/>
	</div>
</div>
