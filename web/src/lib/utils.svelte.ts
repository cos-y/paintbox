import { z, ZodType } from 'zod';
import { isTauri } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import { useMode, modeHsl, modeHwb, modeRgb, modeOklch, modeOklab } from 'culori/fn';

export const clamp = (val: number, min: number, max: number) => Math.max(min, Math.min(val, max));

/** sRGB transfer function: linear [0,1] -> sRGB [0,1] */
export const linearToSrgb = (c: number) =>
	c <= 0.0031308 ? 12.92 * c : 1.055 * Math.pow(c, 1 / 2.4) - 0.055;

export const similarity = (deltaE: number) => clamp(1 - deltaE * 0.05, 0, 1) * 100;

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
		const r = (hex >> 16) & 0xff;
		const g = (hex >> 8) & 0xff;
		const b = hex & 0xff;
		rgb = [r, g, b];
	}
	return rgb.map((c) => clamp(c / 255, 0, 1));
};

const breakpoints = {
	sm: '(min-width: 640px)',
	md: '(min-width: 768px)',
	lg: '(min-width: 1024px)',
	xl: '(min-width: 1280px)',
	'2xl': '(min-width: 1536px)',
	coarse: '(pointer: coarse)'
};

let media = $state({
	sm: false,
	md: false,
	lg: false,
	xl: false,
	'2xl': false,
	coarse: false
});

Object.entries(breakpoints).forEach(([key, query]) => {
	const mq = window.matchMedia(query);
	let dict = media as any;
	dict[key] = mq.matches;
	mq.addEventListener('change', (e) => (dict[key] = e.matches));
});

export const isMedia = () => media;

export const openExternal = (url: string) => {
	if (isTauri()) openUrl(url);
	else window.open(url, '_blank', 'noopener');
};

export const toHsl = useMode(modeHsl);
export const toOklch = useMode(modeOklch);
export const toOklab = useMode(modeOklab);
export const toRgb = useMode(modeRgb);
export const toHwb = useMode(modeHwb);

export function loadData<Schema extends ZodType>(key: string, schema: Schema): z.infer<Schema> {
	try {
		if (typeof localStorage !== 'undefined') {
			const raw = localStorage.getItem(key);
			if (raw) {
				const r = schema.safeParse(JSON.parse(raw));
				if (r.success) return r.data;
			}
		}
	} catch {}
	// 空对象即全默认（逐字段 catch 兜底）；每次新建，避免共享默认数组被污染
	return schema.parse({});
}
