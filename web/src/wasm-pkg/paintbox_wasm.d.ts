/* tslint:disable */
/* eslint-disable */

export class Gamut {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    colors(): Float32Array;
    insert(rgb: number): boolean;
    insert_many(rgbs: Uint32Array): boolean;
    matrices(): Float32Array;
}

export class ScatterOut {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    colors(): Float32Array;
    matrices(): Float32Array;
    members(): Uint32Array;
    offsets(): Uint32Array;
}

export function color_diff(a: number, b: number): number;

export function init_panic_hook(): void;

export function init_searcher(blob: Uint8Array): void;

export function list_paints(): any;

export function new_gamut(ndiv: number, li: Uint32Array): Gamut;

export function scatter(ndiv: number, li: Uint32Array): ScatterOut;

export function search(rgb: number, opts: any): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_gamut_free: (a: number, b: number) => void;
    readonly __wbg_scatterout_free: (a: number, b: number) => void;
    readonly color_diff: (a: number, b: number) => number;
    readonly gamut_colors: (a: number) => any;
    readonly gamut_insert: (a: number, b: number) => number;
    readonly gamut_insert_many: (a: number, b: number, c: number) => number;
    readonly gamut_matrices: (a: number) => any;
    readonly init_panic_hook: () => void;
    readonly init_searcher: (a: number, b: number) => [number, number];
    readonly list_paints: () => [number, number, number];
    readonly new_gamut: (a: number, b: number, c: number) => [number, number, number];
    readonly scatter: (a: number, b: number, c: number) => number;
    readonly scatterout_colors: (a: number) => any;
    readonly scatterout_matrices: (a: number) => any;
    readonly scatterout_members: (a: number) => any;
    readonly scatterout_offsets: (a: number) => any;
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
