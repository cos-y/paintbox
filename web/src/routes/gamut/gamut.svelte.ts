import { untrack } from 'svelte';
import * as THREE from 'three';
import { clamp, isMedia, loadData } from '$lib/utils.svelte';
import { isTauri } from '@tauri-apps/api/core';
import { callWasm } from '$lib/wasmClient';
import { z } from 'zod';

const STORAGE_KEY = 'paintbox:gamut';

const SourceSchema = z.object({
	id: z.string(),
	type: z.enum(['color', 'paint', 'stock']),
	/** color card: 6-digit hex as number */
	rgb: z.number().optional(),
	/** paint card: brand:code id */
	paintId: z.string().optional(),
	/** 暂时从色域中排除（不移除卡片） */
	hidden: z.boolean().optional()
});

type SerializedSource = z.infer<typeof SourceSchema>;
export type { SerializedSource };

// 持久化 schema：类型与校验单一来源（z.infer 推导 Serialized）。
// 格式自引入（24fb59c, 2026-08-09）起一直为对象，未变过；
// 缺字段/类型错误由逐字段 .catch() 兜底为默认值。
const GamutSchema = z.object({
	sources: z.array(SourceSchema).catch([]),
	clipL: z.tuple([z.number(), z.number()]).catch([0, 16]),
	clipA: z.tuple([z.number(), z.number()]).catch([-12, 14]),
	clipB: z.tuple([z.number(), z.number()]).catch([-15, 12]),
	nextId: z.number().catch(0),
	/** localStorage 里是否有已保存的记录（区分首次使用 vs 用户保存过但清空了 sources） */
	persisted: z.boolean().catch(true)
});

type Serialized = z.infer<typeof GamutSchema>;

// 只读一次持久化数据
const initial = loadData(STORAGE_KEY, GamutSchema);

class GamutStore {
	sources = $state<SerializedSource[]>(initial.sources);
	clipL = $state<[number, number]>(initial.clipL);
	clipA = $state<[number, number]>(initial.clipA);
	clipB = $state<[number, number]>(initial.clipB);
	nextId = $state(initial.nextId);
	persisted = $state(initial.persisted);

	persist() {
		// untrack：避免在组件 $effect 中调用时，本函数对 store 字段的读取
		// 被追踪为 effect 依赖（否则读写同一 $state 会无限循环）
		untrack(() => {
			const data: Serialized = {
				sources: this.sources,
				clipL: this.clipL,
				clipA: this.clipA,
				clipB: this.clipB,
				nextId: this.nextId,
				persisted: true
			};
			localStorage.setItem(STORAGE_KEY, JSON.stringify(data));
		});
	}
}

export const store = new GamutStore();

// ---- 运行时状态：scene 渲染输入（页面间导航保持）----

/** 体素网格分辨率（Tauri 端降低以适配移动端性能） */
export const ndiv = isTauri() ? 12 : 16;

export const rangeL: [number, number] = [0, ndiv];
export const rangeA: [number, number] = [-Math.ceil(0.7 * ndiv), Math.ceil(0.85 * ndiv)];
export const rangeB: [number, number] = [-Math.ceil(0.9 * ndiv), Math.ceil(0.7 * ndiv)];

class SceneProps {
	/** wasm 输出：体素实例矩阵 / 颜色（由页面驱动更新） */
	matrices = $state(new Float32Array()) as Float32Array<ArrayBufferLike>;
	colors = $state(new Float32Array()) as Float32Array<ArrayBufferLike>;

	/** 裁剪范围（RangeSlider 绑定 + scene 裁剪平面），初值来自持久化并 clamp 到 range */
	clipL = $state<[number, number]>(store.clipL.map((x) => clamp(x, ...rangeL)) as [number, number]);
	clipA = $state<[number, number]>(store.clipA.map((x) => clamp(x, ...rangeA)) as [number, number]);
	clipB = $state<[number, number]>(store.clipB.map((x) => clamp(x, ...rangeB)) as [number, number]);

	clip = $derived([
		new THREE.Vector3(this.clipL[0], this.clipA[0], this.clipB[0]),
		new THREE.Vector3(this.clipL[1], this.clipA[1], this.clipB[1])
	]);
	range = $derived([
		new THREE.Vector3(rangeL[0], rangeA[0], rangeB[0]),
		new THREE.Vector3(rangeL[1], rangeA[1], rangeB[1])
	]);

	/** 相机位置 / 朝向目标（scene 视角；OrbitControls 双向同步，运行时状态） */
	cameraPos = $state<[number, number, number]>([3, 0.5, 1]);
	cameraTarget = $state<[number, number, number]>([0, 0, 0]);

	/** 相机缩放（fov 系数；移动端默认放大） */
	zoom = $state(isMedia().sm ? 1 : 1.5);

	// ---- gamut 维护流程：wasm 对象生命周期 = 应用生命周期（不随页面销毁）----
	// 组件 $effect 里调用 updateColors() 驱动；页面间导航时句柄与体素数据保持，
	// 只有颜色集合真正变化才重建/增量更新，切页回来不重算。
	private gamut: string | undefined;
	private task: Promise<void> | undefined;
	private lastColors: Set<number> | undefined;

	private async freeGamut(g: string) {
		console.log('Gamut::free');
		await callWasm<void>('free', [g]).catch(() => {});
	}

	private async updateScene(g: string) {
		const [matrices, colors] = await Promise.all([
			callWasm<Float32Array>('gamut_matrices', [g]),
			callWasm<Float32Array>('gamut_colors', [g])
		]);
		this.matrices = matrices;
		this.colors = colors;
	}

	private rebuild(colors: Set<number>) {
		const fn = async () => {
			const newGamut = await callWasm<string>('new_gamut', [ndiv, new Uint32Array(colors)], {
				cancelInFlight: true
			});
			console.log('Gamut::new');
			if (this.gamut !== undefined) {
				await this.freeGamut(this.gamut);
			}
			this.gamut = newGamut;
			await this.updateScene(newGamut);
		};
		this.task = fn();
	}

	private insert(added: Set<number>) {
		const fn = async (prev: Promise<void> | undefined) => {
			await prev;
			if (this.gamut === undefined) return;
			const modified = await callWasm<boolean>('gamut_insert_many', [
				this.gamut,
				new Uint32Array(added)
			]);
			console.log('Gamut::insert', modified);
			if (modified) await this.updateScene(this.gamut);
		};
		this.task = fn(this.task);
	}

	/** 组件 $effect 中调用：颜色集合变化时增量插入或整体重建 */
	updateColors(colors: Set<number>) {
		if (this.lastColors === undefined) {
			this.rebuild(colors);
			this.lastColors = colors;
			return;
		}
		const removed = this.lastColors.difference(colors);
		const added = colors.difference(this.lastColors);
		this.lastColors = colors;
		if (removed.size > 0) {
			this.rebuild(colors);
		} else if (added.size > 0) {
			this.insert(added);
		}
	}
}

export const sceneProps = new SceneProps();
