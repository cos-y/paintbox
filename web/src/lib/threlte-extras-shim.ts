/**
 * @threlte/extras 的最小 shim：只导出本项目用到的三个符号，
 * 避免引入 extras 的 barrel 入口（index.js 会连带 GLTF 组件 →
 * three GLTFLoader → DRACOLoader → draco decoder wasm，全部打进产物）。
 *
 * 用相对路径直接指向 extras 的 dist 内部模块，绕过其 exports 的 "." 限制。
 */
export { default as Gizmo } from '../../node_modules/@threlte/extras/dist/components/Gizmo/Gizmo.svelte';
export { default as OrbitControls } from '../../node_modules/@threlte/extras/dist/components/controls/OrbitControls/OrbitControls.svelte';
export { interactivity } from '../../node_modules/@threlte/extras/dist/interactivity/index.js';
