/**
 * main.ts
 *
 * Bootstraps Vuetify and other plugins then mounts the App`
 */

// Initialize WASM modules first, before any other imports that might use them
import { initializeWasm } from '@/api/wasmLoader'

// Styles
// Import Vuetify styles (configured via vite.config.mts -> settings.scss)
// Then override fonts with system fonts
import '@/styles/main.css'
await initializeWasm()

// Now safe to import modules that depend on WASM
const { registerPlugins } = await import('@/plugins')
const { default: App } = await import('./App.vue')
const { createApp } = await import('vue')

const app = createApp(App)

registerPlugins(app)

app.mount('#app')
