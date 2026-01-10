/**
 * router/index.ts
 *
 * Automatic routes for `./src/pages/*.vue`
 */

// Composables
import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'
import { setupLayouts } from 'virtual:generated-layouts'
import { routes } from 'vue-router/auto-routes'
import { useAuthStore } from '@/stores/auth'

// Define route metadata interface
declare module 'vue-router' {
  interface RouteMeta {
    requiresAuth?: boolean
    guestOnly?: boolean
  }
}

// Configure route metadata
const configureRoutes = (routes: RouteRecordRaw[]): RouteRecordRaw[] => {
  return routes.map(route => {
    // Guest-only routes (redirect to home if already logged in)
    if (['/login', '/register', '/forgot-password'].includes(route.path)) {
      route.meta = { ...route.meta, guestOnly: true }
    }

    // Recursively configure child routes if they exist
    if (route.children) {
      route.children = configureRoutes(route.children)
    }

    return route
  })
}

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: setupLayouts(configureRoutes(routes)),
})

router.beforeEach(async (to, _from, next) => {
  const authStore = useAuthStore()

  // Load user from localStorage on first navigation
  if (!authStore.user) {
    authStore.loadUser()
  }

  // Check if route requires authentication
  if (to.meta.requiresAuth) {
    // Check if session exists and is still valid
    if (!authStore.isAuthenticated || !authStore.isSessionValid()) {
      // Not logged in or session expired, clear and redirect to login
      authStore.clearUser()
      next('/login')
      return
    }
  }

  // Check if route is guest-only (login, register, etc.)
  if (to.meta.guestOnly) {
    if (authStore.isAuthenticated && authStore.isSessionValid()) {
      // Already logged in with valid session, redirect to home
      next('/')
      return
    }
  }

  // Allow navigation
  next()
})

// Workaround for https://github.com/vitejs/vite/issues/11804
router.onError((err, to) => {
  if (err?.message?.includes?.('Failed to fetch dynamically imported module')) {
    if (localStorage.getItem('vuetify:dynamic-reload')) {
      console.error('Dynamic import error, reloading page did not fix it', err)
    } else {
      console.log('Reloading page to fix dynamic import error')
      localStorage.setItem('vuetify:dynamic-reload', 'true')
      location.assign(to.fullPath)
    }
  } else {
    console.error(err)
  }
})

router.isReady().then(() => {
  localStorage.removeItem('vuetify:dynamic-reload')
})

export default router
