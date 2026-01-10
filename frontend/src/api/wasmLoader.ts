/**
 * WASM Loader
 *
 * Initializes WebAssembly modules for field validation and translation.
 */

import init_validator from '@/wasm/field-validator.js'
import init_translator from '@/wasm/translator.js'

let initialized = false

export async function initializeWasm(): Promise<void> {
  if (initialized) return

  await Promise.all([
    init_validator(),
    init_translator(),
  ])

  initialized = true
}
