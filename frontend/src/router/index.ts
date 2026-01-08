/**
 * router/index.ts
 *
 * Automatic routes for `./src/pages/*.vue`
 */

// Composables
import { createRouter, createWebHistory } from 'vue-router'
import { setupLayouts } from 'virtual:generated-layouts'
import { routes } from 'vue-router/auto-routes'
import { checkAuth, getCounter } from '@/services/api'
import { useAuthStore } from '@/stores/auth'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: setupLayouts(routes),
})

// Define which routes require authentication
const authRequiredRoutes = ['/dashboard']

// Define which routes should redirect to dashboard if already logged in
const guestOnlyRoutes = ['/login', '/register', '/forgot-password']

router.beforeEach(async (to, from, next) => {
  const authStore = useAuthStore()

  // Load user from localStorage on first navigation
  if (!authStore.user) {
    authStore.loadUser()
  }

  const isAuthRequired = authRequiredRoutes.includes(to.path)
  const isGuestOnly = guestOnlyRoutes.includes(to.path)

  // Handle authentication-required routes
  if (isAuthRequired) {
    if (!authStore.isAuthenticated) {
      // Not logged in, redirect to login
      next('/login')
      return
    }

    // Verify session is still valid with backend
    if (from.path !== to.path) {
      try {
        const userData = await checkAuth()
        // Update user data from backend response
        authStore.setUser(userData)

        // For dashboard, also pre-fetch counter
        if (to.path === '/dashboard') {
          try {
            const counterData = await getCounter()
            to.meta.initialCounter = counterData.value
          } catch (e) {
            console.error('Failed to fetch counter', e)
          }
        }
      } catch (e) {
        console.error('Auth check failed, session expired or invalid', e)
        authStore.clearUser()
        next('/login')
        return
      }
    }
    next()
    return
  }

  // Handle guest-only routes (login, register, etc.)
  if (isGuestOnly && authStore.isAuthenticated) {
    // Already logged in, redirect to dashboard
    next('/dashboard')
    return
  }

  // All other routes are accessible to everyone
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
