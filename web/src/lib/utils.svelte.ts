import { isTauri } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import { useMode, modeHsl, modeHwb, modeRgb, modeOklch } from 'culori/fn';

export const clamp = (val: number, min: number, max: number) => Math.max(min, Math.min(val, max));

/** sRGB transfer function: linear [0,1] -> sRGB [0,1] */
export const linearToSrgb = (c: number) =>
	c <= 0.0031308 ? 12.92 * c : 1.055 * Math.pow(c, 1 / 2.4) - 0.055;

export const similarity = (deltaE: number) => clamp(1 - deltaE * 3, 0, 1) * 100;

export const hexToRgb = (hex: string | number): number[] => {
	let rgb;
	if (typeof hex === 'string') {
		const raw = hex.replace(/^#/, '');
		let rgbs;
		if (/^[0-9a-fA-F]{3}$/.test(raw)) {
			const [r, g, b] = raw;
			rgbs = [r + r, g + g, b + b];
		} else {
			const m = raw.match(/^([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})$/);
			if (!m) return [];
			rgbs = [m[1], m[2], m[3]];
		}
		rgb = rgbs.map((s) => parseInt(s, 16));
	} else {
		const r = ((hex >> 16) & 0xff) / 255;
		const g = ((hex >> 8) & 0xff) / 255;
		const b = (hex & 0xff) / 255;
		rgb = [r, g, b];
	}
	return rgb.map((c) => clamp(c / 255, 0, 1));
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

export const toHsl = useMode(modeHsl);
export const toOklch = useMode(modeOklch);
export const toRgb = useMode(modeRgb);
export const toHwb = useMode(modeHwb);
