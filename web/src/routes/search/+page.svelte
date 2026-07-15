<script lang="ts">
	import { useMode, modeHwb, modeRgb, modeOklch, type Oklch } from 'culori/fn';

	import Hsl from '$lib/components/Hsl.svelte';
	import Rgb from '$lib/components/Rgb.svelte';
	import { Box, ChevronUp, Cylinder, icons, Pipette } from 'lucide-svelte';

	let oklch: Oklch = $state({ mode: 'oklch', l: 0, c: 0, h: 0 });

	const toHwb = useMode(modeHwb);
	const toRgb = useMode(modeRgb);
	const toOklch = useMode(modeOklch);
	const hwb = $derived(toHwb(oklch));
	const rgb = $derived(toRgb(oklch));

	let model = $state(0);
	const models = [
		{ name: 'HSL', icon: Cylinder, picker: Hsl },
		{ name: 'RGB', icon: Box, picker: Rgb }
	];

	const handleSelectModel = (newModel: number) => {
		model = newModel;
		if (document.activeElement instanceof HTMLElement) {
			document.activeElement.blur();
		}
	};

	const eyedrop = () => {
		if ('EyeDropper' in window) {
			let EyeDropper = (window as any).EyeDropper;
			const eyeDropper = new EyeDropper();
			eyeDropper.open().then((result: any) => {
				const hex = parseInt(result.sRGBHex.slice(1), 16);
				const b = (hex & 0xff) / 255;
				const g = ((hex >> 8) & 0xff) / 255;
				const r = ((hex >> 16) & 0xff) / 255;
				oklch = toOklch({ mode: 'rgb', r, g, b });
			});
		}
	};
</script>

<div
	class="color-picker-root"
	style="
    --picker-oklchLightness: {oklch.l}; 
    --picker-oklchChroma: {oklch.c}; 
    --picker-oklchHue: {oklch.h ?? 0}; 
    --picker-hue: {hwb.h ?? 0}; 
    --picker-whiteness: {hwb.w}; 
    --picker-blackness: {hwb.b}; 
    --picker-color-srgb: rgb({rgb.r * 255} {rgb.g * 255} {rgb.b * 255}); 
    --picker-oklch: oklch({oklch.l} {oklch.c} {oklch.h})"
>
	{#snippet picker()}
		{@const { name, icon: Icon, picker: Picker } = models[model]}

		<div>
			<div class="color-swatch relative">
				<div class="dx-tooltip dx-tooltip-bottom flex right-0">
					<button type="button" class="flex-1" onclick={eyedrop}>
						<Pipette size="1rem" color="#666" />
					</button>
				</div>
			</div>

			<div class="dx-dropdown dx-dropdown-top">
				<button
					type="button"
					tabindex="0"
					class="w-24 dx-btn dx-btn-sm"
					// border-base-content/10 outline-base-content inline-flex h-10 cursor-pointer rounded-4xl
					// border-1 outline-offset-2 focus:outline-2 text-sm
					// text-primary-content bg-primary
					// w-24 place-items-center
					aria-label="Choose --color-base-content: oklch(83.768% 0.001 17.911)"
					title="--color-base-content: oklch(83.768% 0.001 17.911)"
				>
					<Icon class="size-4" />
					{name}
					<ChevronUp size="12" />
				</button>
				<ul class="dx-dropdown-content dx-menu rounded-box bg-base-200 w-30 text-xs">
					{#each models as { name, icon: Icon }, i}
						<li>
							<button
								type="button"
								class={i === model ? 'menu-active' : ''}
								onclick={() => handleSelectModel(i)}><Icon class="size-4" />{name}</button
							>
						</li>
					{/each}
				</ul>
			</div>
		</div>

		<Picker bind:oklch />
	{/snippet}

	{@render picker()}
</div>
