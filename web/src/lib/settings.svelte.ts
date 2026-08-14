import { z } from 'zod';
import { loadData } from './utils.svelte';

const STORAGE_KEY = 'paintbox:settings';

// 持久化格式：扁平对象。逐字段 catch 兜底（一个脏字段不连坐整个配置），
// 整体非对象 / 损坏 JSON → {} 重置为全默认。
const SettingsSchema = z
	.object({
		/** 油漆色名用源语言显示（不经本地化翻译字典） */
		displayRaw: z.boolean().catch(false),
		/** 深色模式（预留字段：目前仅存储，未实际应用主题） */
		theme: z.enum(['system', 'dark', 'light']).catch('system')
	})
	.catch({ displayRaw: false, theme: 'system' });

const initial = loadData(STORAGE_KEY, SettingsSchema);

class SettingsStore {
	displayRaw = $state(initial.displayRaw);
	theme = $state<'system' | 'dark' | 'light'>(initial.theme);

	setDisplayRaw(v: boolean) {
		if (this.displayRaw === v) return;
		this.displayRaw = v;
		this.persist();
	}

	/** 主题切换 */
	setTheme(v: 'system' | 'dark' | 'light') {
		if (this.theme === v) return;
		this.theme = v;
		this.persist();
	}

	/** 批量替换（备份导入用）；缺省字段保留当前值 */
	replaceAll(v: { displayRaw?: boolean; theme?: 'system' | 'dark' | 'light' }) {
		if (v.displayRaw !== undefined && v.displayRaw !== this.displayRaw) {
			this.displayRaw = v.displayRaw;
		}
		if (v.theme !== undefined && v.theme !== this.theme) {
			this.theme = v.theme;
		}
		this.persist();
	}

	private persist() {
		localStorage.setItem(
			STORAGE_KEY,
			JSON.stringify({ displayRaw: this.displayRaw, theme: this.theme })
		);
	}
}

export const settings = new SettingsStore();
