/* tslint:disable */
/* eslint-disable */

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly main: (a: number, b: number) => number;
  readonly wasm_bindgen__convert__closures_____invoke__h1c6acd74f92f3363: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h020c113e9228a07e: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h2c9e335e82cbb61c: (a: number, b: number, c: any, d: any) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h3691332d51cfd27e: (a: number, b: number, c: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h51c63c05007611e4: (a: number, b: number) => void;
  readonly wasm_bindgen__closure__destroy__hfbaa02e1fad5aca7: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__ha0e289e812937d0c: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__he5a4200d793a3ce5: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__ha35c21d82c126d0f: (a: number, b: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
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
