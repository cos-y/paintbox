<script lang="ts">
	import { SURFACE_BITS, rgbToHex, type PaintInfo } from '$lib/paints.svelte';

	interface Props {
		paint: PaintInfo;
	}

	let { paint }: Props = $props();

	const hex = $derived(rgbToHex(paint.rgb));

	// 特种色示意性渲染模式：Clear > Metallic > Pearl > Fluo，其余平涂
	// 视觉语言刻意保持"美术示意"而非拟真（数据只有 hex，不能假装对实际观感负责）：
	// 金属=全局跟随光带 / 透明=叠层+网格 / 珠光=全局跟随光晕 / 荧光=呼吸
	const mode = $derived(
		paint.surfaces & SURFACE_BITS.C
			? 'clear'
			: paint.surfaces & SURFACE_BITS.ME
				? 'metallic'
				: paint.surfaces & SURFACE_BITS.PA
					? 'pearl'
					: paint.surfaces & SURFACE_BITS.FL
						? 'fluo'
						: 'flat'
	);
</script>

<div class="swatch-fx base rounded-md {mode}" style="--c: {hex};">
	{#if mode === 'metallic' || mode === 'pearl'}
		<div class="fx-band"></div>
	{/if}
	{#if mode === 'clear'}
		<div class="fx-tint"></div>
		<div class="fx-grid"></div>
	{/if}
</div>

<style>
	.swatch-fx {
		position: absolute;
		inset: 0;
		overflow: hidden;
		pointer-events: none;
	}

	.swatch-fx.base {
		background-color: var(--c);
	}

	.fx-band {
		position: absolute;
		left: 20%;
		top: 0%;
		top: -25%;
		bottom: -25%;
	}

	/* ---- 金属：平涂基色 + 全局跟随光带（高光即立体感，无静态渐变） ---- */
	.swatch-fx.metallic {
		--c0: oklch(from var(--c) 0.9 c h);
		--c1: color-mix(in srgb, var(--c0) 40%, transparent);
		--c2: color-mix(in srgb, var(--c0) 10%, transparent);
	}
	.swatch-fx.metallic .fx-band {
		width: 50%;
		rotate: 45deg;
		translate: 40% 0;
		background: linear-gradient(to right, transparent, var(--c2), transparent);
	}
	.swatch-fx.metallic .fx-band::after {
		content: '';
		position: absolute;
		top: 0;
		bottom: 0;
		left: 50%;
		width: 3px;
		margin-left: -1.5px;
		background: linear-gradient(to bottom, var(--c2), var(--c1), var(--c2));
		/* background: var(--c2); */
		box-shadow: 0 0 8px rgba(255, 255, 255, 0.35);
	}

	/* ---- 珠光：光带几何与金属一致（斜向高光），两端偏色相反色相（双色性），
	   表面叠加微量闪粉。无 flip 数据，示意性近似 ---- */
	.swatch-fx.pearl {
		--c0: oklch(from var(--c) 0.8 c calc(h + 60));
		--c1: color-mix(in srgb, var(--c0) 75%, transparent);
		--c2: color-mix(in srgb, var(--c0) 10%, transparent);
		--c3: oklch(from var(--c) l calc(c * 1.5) calc(h - 60) / 80%);
	}
	.swatch-fx.pearl .fx-band {
		width: 200%;
		rotate: 45deg;
		translate: -30% 0%;
		background: linear-gradient(to right, var(--c3), var(--c2), var(--c));
	}
	.swatch-fx.pearl .fx-band::after {
		content: '';
		position: absolute;
		top: 0;
		bottom: 0;
		left: 50%;
		width: 8px;
		margin-left: -1.5px;
		background: linear-gradient(to bottom, var(--c2), var(--c1), var(--c2));
	}
	/* 微量闪粉：单一大 tile 内 18 个错位点（位置/大小/透明度各异），平铺后呈伪随机分布，
	   避免规律网格的机械感 */
	.swatch-fx.pearl::after {
		content: '';
		position: absolute;
		inset: 0;
		background-image:
			radial-gradient(circle 1.4px at 2px 4px, rgba(255, 255, 255, 0.75) 50%, transparent 51%),
			radial-gradient(circle 1px at 12px 2px, rgba(255, 255, 255, 0.4) 50%, transparent 51%),
			radial-gradient(circle 1.2px at 22px 5px, rgba(255, 255, 255, 0.55) 50%, transparent 51%),
			radial-gradient(circle 0.9px at 28px 1px, rgba(255, 255, 255, 0.35) 50%, transparent 51%),
			radial-gradient(circle 1px at 7px 10px, rgba(255, 255, 255, 0.45) 50%, transparent 51%),
			radial-gradient(circle 1.3px at 17px 12px, rgba(255, 255, 255, 0.65) 50%, transparent 51%),
			radial-gradient(circle 0.8px at 26px 9px, rgba(255, 255, 255, 0.3) 50%, transparent 51%),
			radial-gradient(circle 1.1px at 3px 17px, rgba(255, 255, 255, 0.5) 50%, transparent 51%),
			radial-gradient(circle 0.9px at 14px 20px, rgba(255, 255, 255, 0.42) 50%, transparent 51%),
			radial-gradient(circle 1.4px at 24px 16px, rgba(255, 255, 255, 0.7) 50%, transparent 51%),
			radial-gradient(circle 1px at 29px 21px, rgba(255, 255, 255, 0.45) 50%, transparent 51%),
			radial-gradient(circle 0.8px at 9px 25px, rgba(255, 255, 255, 0.33) 50%, transparent 51%),
			radial-gradient(circle 1.2px at 18px 27px, rgba(255, 255, 255, 0.58) 50%, transparent 51%),
			radial-gradient(circle 0.9px at 26px 29px, rgba(255, 255, 255, 0.38) 50%, transparent 51%),
			radial-gradient(circle 1.1px at 5px 30px, rgba(255, 255, 255, 0.52) 50%, transparent 51%),
			radial-gradient(circle 1.3px at 21px 24px, rgba(255, 255, 255, 0.6) 50%, transparent 51%),
			radial-gradient(circle 0.7px at 30px 13px, rgba(255, 255, 255, 0.3) 50%, transparent 51%),
			radial-gradient(circle 1px at 11px 15px, rgba(255, 255, 255, 0.47) 50%, transparent 51%);
		background-size: 32px 33px;
		opacity: 0.25;
		pointer-events: none;
	}

	/* ---- 透明：金属灰底 + 半透色 + 网格（图层隐喻，示意而非拟真） ---- */
	.swatch-fx.clear {
		background-image: linear-gradient(to bottom, #e8e8e8, #bcbcbc 45%, #8c8c8c);
	}
	.swatch-fx.clear .fx-tint {
		position: absolute;
		inset: 0;
		background-color: var(--c);
		opacity: 0.72;
	}
	.swatch-fx.clear .fx-grid {
		position: absolute;
		inset: 0;
		background-image:
			linear-gradient(rgba(255, 255, 255, 0.24) 1px, transparent 1px),
			linear-gradient(90deg, rgba(255, 255, 255, 0.24) 1px, transparent 1px);
		background-size: 16px 16px;
	}

	/* ---- 荧光：平涂基色 + 圆角矩形辉光。
	   亮度分布用 radial（等值线圆、自然衰减无硬边），外形由 border-radius 裁成圆角矩形，
	   角部是圆的边界而非两个正交渐变的乘积塌陷（即"radial 画法 + 矩形外形"） ---- */
	.swatch-fx.fluo::after {
		content: '';
		position: absolute;
		width: 100%;
		height: 100%;
		left: 0;
		top: 0;
		border-radius: 12%;
		background: radial-gradient(
			circle,
			oklch(from var(--c) calc(l * 1.2) c h),
			oklch(from var(--c) calc(l * 1.1) c h) 40%,
			oklch(from var(--c) calc(l * 1) c h) 60%,
			oklch(from var(--c) calc(l * 0.8) c h) 100%
		);
		pointer-events: none;
	}
</style>
