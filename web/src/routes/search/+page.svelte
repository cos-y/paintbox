<script lang="ts">
	import { useMode, modeHwb, modeRgb, modeOklch, type Oklch } from 'culori/fn';

	import Hsl from '$lib/components/Hsl.svelte';
	import Rgb from '$lib/components/Rgb.svelte';
	import { Box, ChevronUp, Cylinder, Pipette } from 'lucide-svelte';
	import { Button, Dropdown, DropdownItem } from 'flowbite-svelte';

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

	let modelDropdownOpen = $state(false);

	const handleSelectModel = (newModel: number) => {
		model = newModel;
		modelDropdownOpen = false;
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
				<div class="flex right-0">
					<button type="button" class="flex-1" onclick={eyedrop}>
						<Pipette size="1rem" color="#666" />
					</button>
				</div>
			</div>

			<div>
				<Button size="xs" class="w-24" aria-label="Choose color model">
					<Icon class="size-4" />
					{name}
					<ChevronUp size="12" />
				</Button>
				<Dropdown placement="top" class="w-30 text-xs" bind:isOpen={modelDropdownOpen}>
					{#each models as { name, icon: Icon }, i}
						<DropdownItem
							class={i === model ? 'bg-gray-100 dark:bg-gray-600' : ''}
							onclick={() => handleSelectModel(i)}
						>
							<span class="flex items-center gap-1"><Icon class="size-4" />{name}</span>
						</DropdownItem>
					{/each}
				</Dropdown>
			</div>
		</div>

		<Picker bind:oklch />
	{/snippet}

	{@render picker()}
</div>
