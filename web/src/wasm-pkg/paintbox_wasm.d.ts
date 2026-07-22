/* tslint:disable */
/* eslint-disable */

export class HullProxy {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    add(rgb: number): void;
    colors(): Float32Array;
    indices(): Uint32Array;
    static new(rgbs: Uint32Array): HullProxy;
    points(): Float32Array;
}

export function color_diff(rgb_a: number, rgb_b: number): number;

export function find_direct_equivalences(index: number): any;

export function hull(li: Uint32Array): HullProxy;

export function init_panic_hook(): void;

export function init_searcher(blob: Uint8Array, equiv_blob: Uint8Array): void;

export function list_paints(): any;

export function search(rgb: number, opts: any): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_hullproxy_free: (a: number, b: number) => void;
    readonly color_diff: (a: number, b: number) => number;
    readonly find_direct_equivalences: (a: number) => [number, number, number];
    readonly hull: (a: number, b: number) => [number, number, number];
    readonly hullproxy_add: (a: number, b: number) => void;
    readonly hullproxy_colors: (a: number) => any;
    readonly hullproxy_indices: (a: number) => any;
    readonly hullproxy_new: (a: number, b: number) => [number, number, number];
    readonly hullproxy_points: (a: number) => any;
    readonly init_panic_hook: () => void;
    readonly init_searcher: (a: number, b: number, c: number, d: number) => [number, number];
    readonly list_paints: () => [number, number, number];
    readonly search: (a: number, b: any) => [number, number, number];
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
