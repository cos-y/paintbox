import { SURFACE_BITS, type PaintInfo } from '$lib/paints.svelte';
import { t } from '$lib/i18n.svelte';

const BASE_KEYS = ['base.lacquer', 'base.alcohol', 'base.enamel', 'base.water'] as const;
const MEDIUM_KEYS = ['medium.Airbrush', 'medium.Spray', 'medium.Brush', 'medium.Marker'] as const;
const SURFACE_KEYS = [
	['G', 'surface.G'],
	['SG', 'surface.SG'],
	['M', 'surface.M'],
	['ME', 'surface.ME'],
	['C', 'surface.C'],
	['PA', 'surface.PA'],
	['FL', 'surface.FL'],
	['W', 'surface.W'],
	['U', 'surface.U']
] as const;

/** 溶剂标签：base 位拆解（枚举位 → i18n 文案） */
export const baseLabels = (paint: PaintInfo): string[] => {
	const labels: string[] = [];
	for (let i = 0; i < BASE_KEYS.length; i++) {
		if (paint.bases & (1 << i)) labels.push(t(BASE_KEYS[i]));
	}
	return labels;
};

/** 漆面标签：prop 位拆解（surface 缩写 → i18n 文案） */
export const surfaceLabels = (paint: PaintInfo): string[] => {
	const labels: string[] = [];
	for (const [bit, key] of SURFACE_KEYS) {
		if (paint.surfaces & SURFACE_BITS[bit]) labels.push(t(key));
	}
	return labels;
};

/** 漆面标签：prop 位拆解（surface 缩写 → i18n 文案） */
export const mediumLabels = (paint: PaintInfo): string[] => {
	const labels: string[] = [];
	for (let i = 0; i < MEDIUM_KEYS.length; i++) {
		if (paint.mediums & (1 << i)) labels.push(t(MEDIUM_KEYS[i]));
	}
	return labels;
};
