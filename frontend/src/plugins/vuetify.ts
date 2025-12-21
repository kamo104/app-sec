/**
 * plugins/vuetify.ts
 *
 * Framework documentation: https://vuetifyjs.com
 */

// Composables
import { createVuetify } from 'vuetify'

// Styles are imported via src/styles/settings.scss which configures system fonts
// No need to import 'vuetify/styles' here as it's handled by the config file

// https://vuetifyjs.com/en/introduction/why-vuetify/#feature-guides
export default createVuetify({
  theme: {
    defaultTheme: 'system',
  },
  defaults: {
    VBtn: {
      color: 'primary',
    },
  },
})
