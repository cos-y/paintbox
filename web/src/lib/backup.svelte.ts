import { z } from 'zod';
import { stock } from './stock.svelte';
import { settings } from './settings.svelte';
import { i18n, type Locale } from './i18n.svelte';

/**
 * 数据备份 / 恢复。
 *
 * 备份内容：油漆库存（stock）+ 偏好设置（settings）。
 * 导出的 JSON 结构：
 *   { app: 'paintbox', version: 1, exportedAt: ISO 时间, stock: string[], settings: {...} }
 */

export const BACKUP_APP = 'paintbox';
export const BACKUP_VERSION = 1;

/** 导入校验：严格模式（与导出时的容错 schema 不同，脏数据直接报错不静默修正） */
const BackupSchema = z.object({
	app: z.literal('paintbox'),
	version: z.number().int().min(1),
	exportedAt: z.string().optional(),
	stock: z.array(z.string()),
	settings: z
		.object({
			displayRaw: z.boolean().optional(),
			theme: z.enum(['system', 'dark', 'light']).optional(),
			/** 界面语言（en/zh/ja/es），旧备份可能没有 */
			locale: z.string().optional()
		})
		.optional()
});

export type BackupData = z.infer<typeof BackupSchema>;

export interface ParseResult {
	ok: boolean;
	data?: BackupData;
	/** 错误 key：'invalidJson' | 'invalidSchema'（i18n data.* 翻译） */
	error?: string;
}

/** 序列化当前库存 + 设置为备份 JSON 字符串 */
export function exportBackup(): string {
	return JSON.stringify(
		{
			app: BACKUP_APP,
			version: BACKUP_VERSION,
			exportedAt: new Date().toISOString(),
			stock: [...stock.values],
			settings: {
				displayRaw: settings.displayRaw,
				theme: settings.theme,
				locale: i18n.locale
			}
		},
		null,
		2
	);
}

/** 解析并校验备份文本 */
export function parseBackup(text: string): ParseResult {
	let raw: unknown;
	try {
		raw = JSON.parse(text);
	} catch {
		return { ok: false, error: 'invalidJson' };
	}
	const r = BackupSchema.safeParse(raw);
	if (!r.success) return { ok: false, error: 'invalidSchema' };
	return { ok: true, data: r.data };
}

export type ImportMode = 'merge' | 'replace';

/** 导入范围：只库存 / 库存+设置 / 只设置 */
export type ImportScope = 'stock' | 'both' | 'settings';

export interface ImportResult {
	added: number;
	total: number;
}

/**
 * 应用备份：
 * - 库存：scope 含 stock 时处理。mode merge = 并集去重；replace = 整体替换
 * - 设置（displayRaw/theme/locale）：scope 含 settings 时整体替换（标量无可合并）
 */
export function applyBackup(data: BackupData, scope: ImportScope, mode: ImportMode): ImportResult {
	let before = stock.values.size;
	if (scope === 'stock' || scope === 'both') {
		if (mode === 'merge') {
			const merged = new Set(stock.values);
			for (const id of data.stock) merged.add(id);
			stock.replaceAll(merged);
		} else {
			stock.replaceAll(new Set(data.stock));
		}
	}
	if (scope === 'both' || scope === 'settings') {
		settings.replaceAll({
			displayRaw: data.settings?.displayRaw,
			theme: data.settings?.theme
		});
		if (data.settings?.locale && i18n.locale !== data.settings.locale) {
			i18n.set(data.settings.locale as Locale);
		}
	}
	return { added: stock.values.size - before, total: stock.values.size };
}
