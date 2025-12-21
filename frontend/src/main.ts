/**
 * main.ts
 *
 * Bootstraps Vuetify and other plugins then mounts the App`
 */

// Plugins
import { registerPlugins } from '@/plugins'

// Components
import App from './App.vue'

// Composables
import { createApp } from 'vue'

// Styles
// Import Vuetify styles (configured via vite.config.mts -> settings.scss)
// Then override fonts with system fonts
import '@/styles/main.css'

const app = createApp(App)

registerPlugins(app)

app.mount('#app')
