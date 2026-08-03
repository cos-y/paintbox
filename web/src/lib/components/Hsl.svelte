<script lang="ts">
	import { useMode, modeHsl, modeRgb, modeOklch, type Oklch } from 'culori/fn';
	import SliderTrack from './ColorSlider.svelte';
	import ColorCode from './ColorCode.svelte';
	import { clamp, hexToRgb } from '$lib/utils';

	interface Props {
		oklch: Oklch;
	}

	let { oklch = $bindable() }: Props = $props();

	const eps = 1e-12;

	const toHsl = useMode(modeHsl);
	const toOklch = useMode(modeOklch);
	const toRgb = useMode(modeRgb);

	const { h, s, l } = toHsl(oklch);
	// [0, 360)
	let localHue = $state(h ?? 0);
	// [0, 1]
	let localSaturation = $state(s);
	// [0, 1]
	let localLuminosity = $state(l);

	$effect(() => {
		const { l, c, h } = oklch;
		const hsl = toHsl(oklch);

		// hue
		if (h !== undefined) {
			if (c > eps) {
				localHue = hsl.h ?? 0;
			}
		}

		// saturation
		if (!((l < eps || 1 - l < eps) && c < eps)) {
			localSaturation = hsl.s;
		}

		// luminosity
		localLuminosity = hsl.l;
	});

	const update = (h: number, s: number, l: number) => {
		localHue = h;
		localSaturation = s;
		localLuminosity = l;
		oklch = toOklch({ mode: 'hsl', h, s, l });
	};

	const hueStyle = $derived.by(() => {
		let s = 'background: linear-gradient(to right';
		for (let i = 0; i <= 360; ++i) {
			s += `, hsl(${i} calc(var(--picker-saturation) * 1%) calc(var(--picker-luminosity) * 1%))`;
		}
		s += ')';
		return s;
	});

	const saturaionStyle = $derived.by(() => {
		let s = 'background: linear-gradient(to right';
		for (let i = 0; i <= 100; i += 50) {
			s += `, hsl(var(--picker-hue) ${i}% calc(var(--picker-luminosity) * 1%))`;
		}
		s += ')';
		return s;
	});

	const luminosityStyle = $derived.by(() => {
		let s = 'background: linear-gradient(to right';
		for (let i = 0; i <= 100; i += 10) {
			s += `, hsl(var(--picker-hue) calc(var(--picker-saturation) * 1%) ${i}%)`;
		}
		s += ')';
		return s;
	});

	const toText = (h: number, s: number, l: number) => {
		h = clamp(h, 0, 360);
		s = clamp(s, 0, 1);
		l = clamp(l, 0, 1);
		return `hsl(${h.toFixed(0)} ${(s * 100).toFixed(1)}% ${(l * 100).toFixed(1)}%)`;
	};

	const toHex = (h: number, s: number, l: number) => {
		h = clamp(h, 0, 360);
		s = clamp(s, 0, 1);
		l = clamp(l, 0, 1);
		const rgb = toRgb({ mode: 'hsl', h, s, l });
		const [r, g, b] = [rgb.r, rgb.g, rgb.b].map((x) => clamp(Math.round(x * 255), 0, 255));
		return `#${r.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${b.toString(16).padStart(2, '0')}`;
	};
</script>

<div
	class="color-picker"
	style="--picker-hue: {localHue};
		--picker-saturation: {localSaturation * 100}; 
		--picker-luminosity: {localLuminosity * 100};"
>
	<SliderTrack
		min={0}
		max={360}
		value={localHue}
		oninput={(v) => update(v, localSaturation, localLuminosity)}
		style={hueStyle}
	/>
	<SliderTrack
		min={0}
		max={100}
		precision={1}
		value={localSaturation * 100}
		oninput={(v) => update(localHue, v / 100, localLuminosity)}
		style={saturaionStyle}
	/>
	<SliderTrack
		min={0}
		max={100}
		precision={1}
		value={localLuminosity * 100}
		oninput={(v) => update(localHue, localSaturation, v / 100)}
		style={luminosityStyle}
	/>

	<div class="flex h-9 items-center gap-2">
		<ColorCode
			re="^hsl\(([\d.]+)\s*(?:,|\s)\s*([\d.]+)%\s*(?:,|\s)\s*([\d.]+)%\)$"
			text={toText(localHue, localSaturation, localLuminosity)}
			validate={([h, s, l]) => +h >= 0 && +h <= 360 && +s >= 0 && +s <= 100 && +l >= 0 && +l <= 100}
			oninput={(h, s, l) => update(+h, +s / 100, +l / 100)}
			class="flex-3"
		/>

		<ColorCode
			re={`^#([0-9a-fA-F]{3}|[0-9a-fA-F]{6})$`}
			text={toHex(localHue, localSaturation, localLuminosity)}
			class="h-full not-sm:flex-1 sm:w-24"
			textAlign="left"
			oninput={(hex) => {
				const rgb = hexToRgb(hex);
				if (rgb) {
					const [r, g, b] = rgb;
					const hsl = toHsl({ mode: 'rgb', r, g, b });
					update(hsl.h ?? 0, hsl.s, hsl.l);
					console.log('hex input', hex, rgb, hsl);
				}
			}}
		/>
	</div>
</div>
