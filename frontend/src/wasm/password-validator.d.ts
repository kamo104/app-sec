/* tslint:disable */
/* eslint-disable */

/**
 * Validates password and returns errors as a JSON array string.
 */
export function get_password_errors(password: string): string;

/**
 * Validates password and returns detailed strength information as JSON.
 */
export function get_password_strength(password: string): string;

/**
 * Validates password and returns a boolean indicating validity.
 */
export function is_password_valid(password: string): boolean;

/**
 * Validates password and returns a JSON string with the result.
 * This is the WebAssembly interface function.
 */
export function validate_password_wasm(password: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly validate_password_wasm: (a: number, b: number) => [number, number];
  readonly is_password_valid: (a: number, b: number) => number;
  readonly get_password_errors: (a: number, b: number) => [number, number];
  readonly get_password_strength: (a: number, b: number) => [number, number];
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
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
