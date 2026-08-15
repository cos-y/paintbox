import type { PaintInfo } from './paints.svelte';

// sources.json：来源标识 -> 官方色卡 PDF 元信息（url/title）。
// 由 scripts/build_data.py 从 data/wide.json 的 sources 元信息生成，
// 前端仅展示，数据侧新增来源批次时无需改前端。

export interface SourceMeta {
	url: string;
	title: string;
}

let sourceMap = new Map<string, SourceMeta>();

export const loadSourceMeta = async (fetchFn: typeof fetch) => {
	const json = await fetchFn('/sources.json').then(
		(r) => r.json() as Promise<Record<string, SourceMeta>>
	);
	sourceMap = new Map(Object.entries(json));
};

export const getSourceMeta = (id: string): SourceMeta | undefined => sourceMap.get(id);
