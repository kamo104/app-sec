import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { LoginResponseData } from '@/services/api'
import { refreshSession } from '@/services/api'

export const useAuthStore = defineStore('auth', () => {
  const user = ref<LoginResponseData | null>(null)
  const isAuthenticated = computed(() => user.value !== null)
  const sessionExpiresAt = ref<number | null>(null)
  const sessionCreatedAt = ref<number | null>(null)
  let refreshTimer: ReturnType<typeof setTimeout> | null = null

  function setUser(userData: LoginResponseData | null): void {
    user.value = userData
    if (userData) {
      sessionExpiresAt.value = userData.sessionExpiresAt
      sessionCreatedAt.value = userData.sessionCreatedAt
      localStorage.setItem('user', JSON.stringify(userData))
      localStorage.setItem('sessionExpiresAt', String(userData.sessionExpiresAt))
      localStorage.setItem('sessionCreatedAt', String(userData.sessionCreatedAt))
      scheduleRefresh()
    } else {
      sessionExpiresAt.value = null
      sessionCreatedAt.value = null
      localStorage.removeItem('user')
      localStorage.removeItem('sessionExpiresAt')
      localStorage.removeItem('sessionCreatedAt')
      clearRefreshTimer()
    }
  }

  function loadUser(): void {
    const userStr = localStorage.getItem('user')
    const expiresAtStr = localStorage.getItem('sessionExpiresAt')
    const createdAtStr = localStorage.getItem('sessionCreatedAt')

    if (userStr && expiresAtStr && createdAtStr) {
      try {
        const userData = JSON.parse(userStr)
        const expiresAt = Number(expiresAtStr)
        const createdAt = Number(createdAtStr)

        // Check if session is already expired
        const now = Math.floor(Date.now() / 1000)
        if (expiresAt > now) {
          user.value = userData
          sessionExpiresAt.value = expiresAt
          sessionCreatedAt.value = createdAt
          scheduleRefresh()
        } else {
          // Session expired, clear it
          clearUser()
        }
      } catch (e) {
        console.error('Failed to parse user from localStorage', e)
        clearUser()
      }
    }
  }

  function clearUser(): void {
    user.value = null
    sessionExpiresAt.value = null
    sessionCreatedAt.value = null
    localStorage.removeItem('user')
    localStorage.removeItem('sessionExpiresAt')
    localStorage.removeItem('sessionCreatedAt')
    clearRefreshTimer()
  }

  function clearRefreshTimer(): void {
    if (refreshTimer) {
      clearTimeout(refreshTimer)
      refreshTimer = null
    }
  }

  function scheduleRefresh(): void {
    clearRefreshTimer()

    if (!sessionExpiresAt.value || !sessionCreatedAt.value) return

    const now = Math.floor(Date.now() / 1000)
    const sessionLifetime = sessionExpiresAt.value - sessionCreatedAt.value
    const refreshAt = sessionCreatedAt.value + Math.floor(sessionLifetime / 2)
    const refreshIn = refreshAt - now

    if (refreshIn > 0) {
      console.log(`Session refresh scheduled in ${refreshIn} seconds`)
      refreshTimer = setTimeout(async () => {
        try {
          const newData = await refreshSession()
          setUser(newData)
          console.log('Session refreshed successfully')
        } catch (e) {
          console.error('Failed to refresh session, logging out', e)
          clearUser()
        }
      }, refreshIn * 1000)
    }
  }

  function isSessionValid(): boolean {
    if (!sessionExpiresAt.value) return false
    const now = Math.floor(Date.now() / 1000)
    return sessionExpiresAt.value > now
  }

  return {
    user,
    isAuthenticated,
    sessionExpiresAt,
    isSessionValid,
    setUser,
    loadUser,
    clearUser,
  }
})
