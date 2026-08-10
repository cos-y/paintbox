import { clamp } from '$lib/utils.svelte';

/**
 * 滑条指针拖拽的共享逻辑（ColorSlider / RangeSlider 共用）。
 *
 * 与原生 input[type=range] 的交互解耦：
 * - 值按「指针在轨道宽度上的比例」线性映射，与原生 thumb 宽度无关。
 *   移动端不再有原生 44px thumb 带来的两端 ~22px 偏差（手柄提前到顶/到底）。
 * - 拖拽状态存在元素上的 WeakMap 里，组件重渲染不会打断进行中的拖拽。
 * - 触摸手势由轨道元素的 touch-action: pan-y 声明：竖滑滚动页面，横滑驱动滑条。
 *
 * 用法（组件内）：
 *   let trackEl = $state<HTMLElement | null>(null);
 *   const slider = $derived(useSlider({
 *     el: () => trackEl,
 *     min, max, step,
 *     start: (t) => min + t * (max - min),   // pointerdown：命中判定 + 初始值
 *     move: (t) => min + t * (max - min),    // pointermove：新值（拖拽目标已在 start 里定下）
 *     oninput: (v) => ...,
 *   }));
 *   <div class="slider-track" bind:this={trackEl} onpointerdown={slider.pointerdown}
 *        onpointermove={slider.pointermove} onpointerup={slider.pointerup}
 *        onpointercancel={slider.pointercancel}>
 */
export interface SliderOptions {
	/** 轨道元素（指针事件绑定在其上） */
	el: () => HTMLElement | null;
	/** 取值范围（整个调用应包裹在 $derived 中以保持响应式） */
	min: number;
	max: number;
	step: number;
	/**
	 * pointerdown 命中判定：t ∈ [0,1] 为指针在轨道宽度上的比例，
	 * 返回本次拖拽的初始值；返回 null 表示不接管本次拖拽。
	 */
	start: (t: number) => number | null;
	/** pointermove：t → 新值（拖拽目标已在 start 中确定） */
	move: (t: number) => number;
	/** 值变化回调（已按 step 取整并 clamp 到 [min, max]） */
	oninput: (value: number) => void;
}

interface DragState {
	dragging: boolean;
	pointerId: number;
}

// 状态挂在元素上：组件每次重渲染都会重建闭包，但拖拽状态需要跨渲染保持
const states = new WeakMap<HTMLElement, DragState>();
const stateOf = (el: HTMLElement): DragState => {
	let s = states.get(el);
	if (!s) states.set(el, (s = { dragging: false, pointerId: -1 }));
	return s;
};

export function useSlider(o: SliderOptions) {
	const snap = (v: number) => clamp(Math.round(v / o.step) * o.step, o.min, o.max);

	const fractionAt = (clientX: number): number => {
		const el = o.el();
		if (!el) return NaN;
		const rect = el.getBoundingClientRect();
		return clamp((clientX - rect.left) / rect.width, 0, 1);
	};

	const pointerdown = (e: PointerEvent) => {
		if (!e.isPrimary) return;
		const el = o.el();
		if (!el) return;
		const s = stateOf(el);
		if (s.dragging) return;
		const t = fractionAt(e.clientX);
		if (Number.isNaN(t)) return;
		const v = o.start(t);
		if (v === null || v === undefined) return;
		s.dragging = true;
		s.pointerId = e.pointerId;
		el.setPointerCapture(e.pointerId);
		e.preventDefault();
		o.oninput(snap(v));
	};

	const pointermove = (e: PointerEvent) => {
		const el = o.el();
		if (!el) return;
		const s = stateOf(el);
		if (!s.dragging || e.pointerId !== s.pointerId) return;
		const t = fractionAt(e.clientX);
		if (Number.isNaN(t)) return;
		o.oninput(snap(o.move(t)));
	};

	const pointerup = (e: PointerEvent) => {
		const el = o.el();
		if (!el) return;
		const s = stateOf(el);
		if (e.pointerId === s.pointerId) s.dragging = false;
	};

	return { pointerdown, pointermove, pointerup, pointercancel: pointerup };
}
