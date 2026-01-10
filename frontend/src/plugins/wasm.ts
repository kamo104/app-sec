import type { App, Plugin } from 'vue'
import { initializeWasm } from '@/api/wasmLoader'

export const wasmPlugin: Plugin = {
  async install(app: App) {
    try {
      // Provide a loading state if needed
      app.provide('wasmLoading', true)

      await initializeWasm()

      app.provide('wasmLoading', false)
      console.log('WASM plugin: WebAssembly modules initialized successfully')
    } catch (error) {
      console.error('WASM plugin: Failed to initialize WebAssembly modules:', error)
      app.provide('wasmLoading', false)
      app.provide('wasmError', error)
    }
  }
}

export default wasmPlugin
