import initFieldValidator from '@/wasm/field-validator.js'
import initApiTranslator from '@/wasm/translator.js'

let wasmInitialized = false
let initializationPromise: Promise<void> | null = null

export async function initializeWasm(): Promise<void> {
  if (wasmInitialized) return

  if (!initializationPromise) {
    initializationPromise = (async () => {
      try {
        await Promise.all([
          initFieldValidator(),
          initApiTranslator(),
        ])
        wasmInitialized = true
      } catch (error) {
        initializationPromise = null
        throw error
      }
    })()
  }

  return initializationPromise
}

export function isWasmInitialized(): boolean {
  return wasmInitialized
}
