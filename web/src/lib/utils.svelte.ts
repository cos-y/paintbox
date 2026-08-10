import { isTauri } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';

export const clamp = (val: number, min: number, max: number) => Math.max(min, Math.min(val, max));

/** sRGB transfer function: linear [0,1] -> sRGB [0,1] */
export const linearToSrgb = (c: number) =>
	c <= 0.0031308 ? 12.92 * c : 1.055 * Math.pow(c, 1 / 2.4) - 0.055;

export const similarity = (deltaE: number) => clamp(1 - deltaE * 3, 0, 1) * 100;

export const hexToRgb = (hex: string): number[] => {
	const raw = hex.replace(/^#/, '');
	let rgb;
	if (/^[0-9a-fA-F]{3}$/.test(raw)) {
		const [r, g, b] = raw;
		rgb = [r + r, g + g, b + b];
	} else {
		const m = raw.match(/^([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})$/);
		if (!m) return [];
		rgb = [m[1], m[2], m[3]];
	}
	return rgb.map((c) => clamp(parseInt(c, 16) / 255, 0, 1));
};

let isSm_ = $state(false);
if (typeof window !== 'undefined') {
	const mq = window.matchMedia('(min-width: 640px)');
	isSm_ = mq.matches;
	mq.addEventListener('change', (e) => (isSm_ = e.matches));
}

let isCoarse_ = $state(false);
if (typeof window !== 'undefined') {
	const mq = window.matchMedia('(pointer: coarse)');
	isCoarse_ = mq.matches;
	mq.addEventListener('change', (e) => (isCoarse_ = e.matches));
}

export const isSm = () => isSm_;
export const isCoarse = () => isCoarse_;

export const openExternal = (url: string) => {
	if (isTauri()) openUrl(url);
	else window.open(url, '_blank', 'noopener');
};
