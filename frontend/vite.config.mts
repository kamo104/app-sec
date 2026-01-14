import { execSync } from 'node:child_process'
import { existsSync, mkdirSync } from 'node:fs'
import { fileURLToPath, URL } from 'node:url'
import Vue from '@vitejs/plugin-vue'
// Plugins
import AutoImport from 'unplugin-auto-import/vite'
import Components from 'unplugin-vue-components/vite'
import { VueRouterAutoImports } from 'unplugin-vue-router'

import VueRouter from 'unplugin-vue-router/vite'
// Utilities
import { defineConfig } from 'vite'
import Layouts from 'vite-plugin-vue-layouts-next'
import Vuetify, { transformAssetUrls } from 'vite-plugin-vuetify'

// Custom plugin to build WebAssembly modules before build
function wasmPlugin () {
  return {
    name: 'wasm-builder',
    buildStart () {
      const wasmDir = './src/wasm'
      if (!existsSync(wasmDir)) {
        mkdirSync(wasmDir, { recursive: true })
      }

      try {
        console.log('Building WebAssembly field validator...')
        // Use wasm-pack to build the field-validator library
        execSync(
          'cd ../field-validator && wasm-pack build --target web --out-dir ../frontend/src/wasm --out-name field-validator --release --features wasm --quiet',
          { stdio: 'inherit' },
        )

        console.log('Building WebAssembly translator...')
        // Use wasm-pack to build the translator library
        execSync(
          'cd ../translator && wasm-pack build --target web --out-dir ../frontend/src/wasm --out-name translator --release --features wasm --quiet',
          { stdio: 'inherit' },
        )

        // Remove unnecessary files
        execSync('rm -f ../frontend/src/wasm/package.json ../frontend/src/wasm/.gitignore ../frontend/src/wasm/README.md', { stdio: 'ignore' })
        console.log('WebAssembly modules built successfully')
      } catch (error) {
        console.warn('Could not build WebAssembly module:', (error as Error).message)
      }
    },
  }
}

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    wasmPlugin(),
    VueRouter({
      dts: 'src/typed-router.d.ts',
    }),
    Layouts(),
    AutoImport({
      imports: [
        'vue',
        VueRouterAutoImports,
        {
          pinia: ['defineStore', 'storeToRefs'],
        },
      ],
      dts: 'src/auto-imports.d.ts',
      eslintrc: {
        enabled: true,
      },
      vueTemplate: true,
    }),
    Components({
      dts: 'src/components.d.ts',
    }),
    Vue({
      template: { transformAssetUrls },
    }),
    // https://github.com/vuetifyjs/vuetify-loader/tree/master/packages/vite-plugin#readme
    Vuetify({
      autoImport: true,
      styles: {
        configFile: 'src/styles/settings.scss',
      },
    }),
  ],
  optimizeDeps: {
    exclude: [
      'vuetify',
      'vue-router',
      'unplugin-vue-router/runtime',
      'unplugin-vue-router/data-loaders',
      'unplugin-vue-router/data-loaders/basic',
    ],
  },
  define: { 'process.env': {} },
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('src', import.meta.url)),
    },
    extensions: [
      '.js',
      '.json',
      '.jsx',
      '.mjs',
      '.ts',
      '.tsx',
      '.vue',
    ],
  },
  server: {
    port: 3000,
  },
})
