import { untrack } from 'svelte';
import { Vector3 } from 'three';
import { clamp, isMedia, loadData } from '$lib/utils.svelte';
import { isTauri } from '@tauri-apps/api/core';
import { callWasm, isValidObject, WorkerCancelled } from '$lib/wasmClient';
import { z } from 'zod';

const STORAGE_KEY = 'paintbox:gamut';

export type GamutMode = 'gamut' | 'scatter';

/** scatter 导出返回结构（与 wasm.worker.ts 的 scatter 方法一致） */
interface ScatterOut {
	matrices: Float32Array;
	colors: Float32Array;
	members: Uint32Array;
	offsets: Uint32Array;
}

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
	/** 显示模式（默认 scatter：无混合采样，进入页面最快） */
	mode: z.enum(['gamut', 'scatter']).catch('scatter'),
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
	/** 显示模式：gamut（混合色域）/ scatter（输入色散点） */
	mode = $state<GamutMode>(initial.mode);
	persisted = $state(initial.persisted);

	/** 切换显示模式（持久化，遵循 settings 的 setter-persist 模式） */
	setMode(v: GamutMode) {
		if (this.mode === v) return;
		this.mode = v;
		this.persist();
	}

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
				mode: this.mode,
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
		new Vector3(this.clipL[0], this.clipA[0], this.clipB[0]),
		new Vector3(this.clipL[1], this.clipA[1], this.clipB[1])
	]);
	range = $derived([
		new Vector3(rangeL[0], rangeA[0], rangeB[0]),
		new Vector3(rangeL[1], rangeA[1], rangeB[1])
	]);

	/** 相机位置 / 朝向目标（scene 视角；OrbitControls 双向同步，运行时状态） */
	cameraPos = $state<[number, number, number]>([3, 0.5, 1]);
	cameraTarget = $state<[number, number, number]>([0, 0, 0]);

	/** 相机缩放（fov 系数；移动端默认放大） */
	zoom = $state(isMedia().sm ? 1 : 1.5);

	/** 显示模式：gamut（混合色域）/ scatter（输入色散点）；localStorage 恢复，默认 scatter */
	mode = $state<GamutMode>(initial.mode);

	/**
	 * 落点表：网格坐标 key（"gx,gy,gz"）→ 该 cell 的输入色索引列表。
	 * 两种模式共用同一 grid，gamut 与 scatter 的体素矩阵平移都是网格坐标，
	 * 点击任何 voxel 都能用坐标反查落格油漆（gamut 的混合体素若无落点则只显示色卡）。
	 */
	cellColors = $state<Map<string, number[]>>(new Map());

	// ---- gamut 路由：统一管理 rust 对象生命周期与 stale 状态 ----
	// gamut 模式维护一个常驻 Gamut 对象（增量 insert/重建），scatter 模式完全不碰它
	// （懒算：切回 gamut 才按 gamutColors 签名做增量同步）。
	// 两种模式的输出都写进同一个 matrices/colors，渲染层零改动。
	private gamut: string | undefined;
	/** gamut 对象已同步的输入签名（scatter 期间保持 stale，切回时对比增量） */
	private gamutColors: Set<number> | undefined;
	/** 散点模式已计算的输入签名 */
	private lastScatter: Set<number> | undefined;
	/** 散点结果缓存：签名不变时同步重放，无需重新调 wasm */
	private scatterCache:
		| { matrices: Float32Array; colors: Float32Array; cellColors: Map<string, number[]> }
		| undefined;
	private task: Promise<void> | undefined;
	/** 意图版本：每次 updateColors 递增，异步回调写入前检查，过期即丢弃（防快速切换竞态） */
	private version = 0;

	private isCurrent(v: number): boolean {
		return v === this.version;
	}

	private async freeGamut(g: string) {
		await callWasm<void>('::free', [g]).catch(() => {});
	}

	private async updateScene(g: string, v: number) {
		// 只读缓存（insert/new 的 finalize 已算好 matrices/colors），轻量
		let matrices: Float32Array;
		let colors: Float32Array;
		try {
			[matrices, colors] = await Promise.all([
				callWasm<Float32Array>('gamut_matrices', [g]),
				callWasm<Float32Array>('gamut_colors', [g])
			]);
		} catch (e) {
			// 被更新的 rebuild 的 cancelInFlight terminate 掉是正常路径
			if (e instanceof WorkerCancelled) return;
			throw e;
		}
		if (!this.isCurrent(v)) return;
		console.log('Gamut::scene voxels', matrices.length / 16);
		this.matrices = matrices;
		this.colors = colors;
	}

	private rebuild(colors: Set<number>, v: number) {
		const fn = async () => {
			let newGamut: string;
			try {
				// 约定 4：cancelInFlight 会 terminate 整个 worker，旧句柄全部失效；
				// 下面的 freeGamut(this.gamut) 对已失效句柄是幂等安全的（'::free' 注销无操作）。
				newGamut = await callWasm<string>('gamut::new', [ndiv, new Uint32Array(colors)], {
					cancelInFlight: true
				});
			} catch (e) {
				// 被更新的请求 terminate 掉是正常路径，不是错误
				if (e instanceof WorkerCancelled) return;
				throw e;
			}
			if (!this.isCurrent(v)) {
				// 过期：不采用，立即释放防泄漏
				await this.freeGamut(newGamut);
				return;
			}
			if (this.gamut !== undefined) {
				await this.freeGamut(this.gamut);
			}
			this.gamut = newGamut;
			this.gamutColors = colors;
			await this.updateScene(newGamut, v);
		};
		this.task = fn();
	}

	/** 散点模式：全量现算 O(n)（无采样/无形态学），输出塞同一个 matrices/colors + 落点表 */
	private scatterCompute(colors: Set<number>, v: number) {
		const fn = async () => {
			let out: ScatterOut;
			try {
				// ndiv-1：gamut::new 在 Rust 内部 -1（make_grid(ndiv-1)），scatter 必须传
				// 同样的分割数，两侧才落在同一网格
				out = await callWasm<ScatterOut>('scatter', [ndiv - 1, new Uint32Array(colors)]);
			} catch (e) {
				if (e instanceof WorkerCancelled) return;
				throw e;
			}
			if (!this.isCurrent(v)) return;
			this.matrices = out.matrices;
			this.colors = out.colors;
			this.cellColors = SceneProps.buildCellColors(out);
			this.scatterCache = { matrices: out.matrices, colors: out.colors, cellColors: this.cellColors };
		};
		this.task = fn();
	}

	/** 仅维护落点表 + 散点缓存（gamut 模式 colors 变化时后台调用，O(n)，不碰渲染输出） */
	private updateCells(colors: Set<number>, v: number) {
		const fn = async () => {
			let out: ScatterOut;
			try {
				out = await callWasm<ScatterOut>('scatter', [ndiv - 1, new Uint32Array(colors)]);
			} catch (e) {
				if (e instanceof WorkerCancelled) return;
				throw e;
			}
			if (!this.isCurrent(v)) return;
			this.cellColors = SceneProps.buildCellColors(out);
			this.scatterCache = { matrices: out.matrices, colors: out.colors, cellColors: this.cellColors };
		};
		this.task = fn();
	}

	/** scatter 输出 → 落点表（矩阵平移即网格坐标） */
	private static buildCellColors(out: ScatterOut): Map<string, number[]> {
		const map = new Map<string, number[]>();
		for (let i = 0; i < out.offsets.length - 1; i++) {
			map.set(
				SceneProps.cellKey(
					out.matrices[i * 16 + 12],
					out.matrices[i * 16 + 13],
					out.matrices[i * 16 + 14]
				),
				[...out.members.subarray(out.offsets[i], out.offsets[i + 1])]
			);
		}
		return map;
	}

	private static cellKey(x: number, y: number, z: number): string {
		return `${x},${y},${z}`;
	}

	private static sameSet(a: Set<number>, b: Set<number>): boolean {
		if (a.size !== b.size) return false;
		for (const x of a) {
			if (!b.has(x)) return false;
		}
		return true;
	}

	/** 路由入口：颜色集合 + 模式变化时驱动（组件 $effect 里调用） */
	updateColors(colors: Set<number>, mode: GamutMode) {
		const v = ++this.version;
		this.mode = mode;
		const sigChanged =
			this.lastScatter === undefined || !SceneProps.sameSet(this.lastScatter, colors);
		if (sigChanged) this.lastScatter = colors;

		if (mode === 'scatter') {
			if (sigChanged || !this.scatterCache) {
				// lazy：不建/不维护 gamut 对象，输入签名变化（或无缓存）才全量现算散点
				this.scatterCompute(colors, v);
			} else {
				// 签名没变且有缓存：同步重放（立即覆盖可能的 gamut 画面，无需 wasm）
				const c = this.scatterCache;
				this.matrices = c.matrices;
				this.colors = c.colors;
				this.cellColors = c.cellColors;
			}
			return;
		}

		// gamut 模式：colors 变化时后台维护落点表（不渲染，供点击反查油漆）
		if (sigChanged) this.updateCells(colors, v);

		// 常驻对象增量维护；散点期间的增删通过 gamutColors 签名对比同步。
		// 关键：无论有无增量，切回 gamut 都要把缓存读回 matrices/colors——
		// scatterCompute 覆盖过它们，不刷新画面就停在散点。
		// 跨页面恢复：句柄可能因 worker 重启而失效（约定 3），isValidObject 校验后再用。
		if (this.gamut === undefined || this.gamutColors === undefined || !isValidObject(this.gamut)) {
			this.rebuild(colors, v);
			return;
		}
		const removed = this.gamutColors.difference(colors);
		const added = colors.difference(this.gamutColors);
		this.gamutColors = colors;
		if (removed.size > 0) {
			this.rebuild(colors, v);
			return;
		}
		// 无删除：链式等前面的任务完成，先增量 insert（若有），再无条件读回 gamut 缓存
		this.task = this.task?.then(async () => {
			if (!this.isCurrent(v) || this.gamut === undefined) return;
			// 链上再次校验：前序任务可能带 cancelInFlight（约定 4），句柄已失效 → 重建
			if (!isValidObject(this.gamut)) {
				this.rebuild(colors, v);
				return;
			}
			if (added.size > 0) {
				let modified: boolean;
				try {
					modified = await callWasm<boolean>('gamut_insert_many', [
						this.gamut,
						new Uint32Array(added)
					]);
				} catch (e) {
					if (e instanceof WorkerCancelled) return;
					throw e;
				}
				console.log('Gamut::insert', modified);
				if (!this.isCurrent(v)) return;
			}
			await this.updateScene(this.gamut, v);
		});
	}
}

export const sceneProps = new SceneProps();
