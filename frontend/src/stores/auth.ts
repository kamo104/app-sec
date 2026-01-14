import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { type AuthSessionResponse, type LoginResponse, refreshSession, type UserRole } from '@/api/client'

// Common user data fields shared across login and auth responses
interface UserData {
  username: string
  email: string
  role: UserRole
  sessionExpiresAt: number
  sessionCreatedAt: number
}

export const useAuthStore = defineStore('auth', () => {
  const user = ref<UserData | null>(null)
  const isAuthenticated = computed(() => user.value !== null)
  const isAdmin = computed(() => user.value?.role === 'admin')
  const sessionExpiresAt = ref<number | null>(null)
  const sessionCreatedAt = ref<number | null>(null)
  let refreshTimer: ReturnType<typeof setTimeout> | null = null

  function setUser (userData: UserData | LoginResponse | AuthSessionResponse | null): void {
    if (userData) {
      const data: UserData = {
        username: userData.username,
        email: userData.email,
        role: userData.role,
        sessionExpiresAt: userData.sessionExpiresAt,
        sessionCreatedAt: userData.sessionCreatedAt,
      }
      user.value = data
      sessionExpiresAt.value = data.sessionExpiresAt
      sessionCreatedAt.value = data.sessionCreatedAt
      localStorage.setItem('user', JSON.stringify(data))
      localStorage.setItem('sessionExpiresAt', String(data.sessionExpiresAt))
      localStorage.setItem('sessionCreatedAt', String(data.sessionCreatedAt))
      scheduleRefresh()
    } else {
      user.value = null
      sessionExpiresAt.value = null
      sessionCreatedAt.value = null
      localStorage.removeItem('user')
      localStorage.removeItem('sessionExpiresAt')
      localStorage.removeItem('sessionCreatedAt')
      clearRefreshTimer()
    }
  }

  function loadUser (): void {
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
      } catch (error) {
        console.error('Failed to parse user from localStorage', error)
        clearUser()
      }
    }
  }

  function clearUser (): void {
    user.value = null
    sessionExpiresAt.value = null
    sessionCreatedAt.value = null
    localStorage.removeItem('user')
    localStorage.removeItem('sessionExpiresAt')
    localStorage.removeItem('sessionCreatedAt')
    clearRefreshTimer()
  }

  function clearRefreshTimer (): void {
    if (refreshTimer) {
      clearTimeout(refreshTimer)
      refreshTimer = null
    }
  }

  function scheduleRefresh (): void {
    clearRefreshTimer()

    if (!sessionExpiresAt.value || !sessionCreatedAt.value) {
      return
    }

    const now = Math.floor(Date.now() / 1000)
    const sessionLifetime = sessionExpiresAt.value - sessionCreatedAt.value
    const refreshAt = sessionCreatedAt.value + Math.floor(sessionLifetime / 2)
    const refreshIn = refreshAt - now

    if (refreshIn > 0) {
      console.log(`Session refresh scheduled in ${refreshIn} seconds`)
      refreshTimer = setTimeout(async () => {
        try {
          const { data, error } = await refreshSession()
          if (data) {
            setUser(data)
            console.log('Session refreshed successfully')
          } else {
            console.error('Failed to refresh session:', error)
            clearUser()
          }
        } catch (error) {
          console.error('Failed to refresh session, logging out', error)
          clearUser()
        }
      }, refreshIn * 1000)
    }
  }

  function isSessionValid (): boolean {
    if (!sessionExpiresAt.value) {
      return false
    }
    const now = Math.floor(Date.now() / 1000)
    return sessionExpiresAt.value > now
  }

  return {
    user,
    isAuthenticated,
    isAdmin,
    sessionExpiresAt,
    isSessionValid,
    setUser,
    loadUser,
    clearUser,
  }
})
