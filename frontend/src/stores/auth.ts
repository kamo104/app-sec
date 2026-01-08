import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { LoginResponseData } from '@/services/api'

export const useAuthStore = defineStore('auth', () => {
  const user = ref<LoginResponseData | null>(null)
  const isAuthenticated = computed(() => user.value !== null)

  function setUser(userData: LoginResponseData | null): void {
    user.value = userData
    if (userData) {
      localStorage.setItem('user', JSON.stringify(userData))
    } else {
      localStorage.removeItem('user')
    }
  }

  function loadUser(): void {
    const userStr = localStorage.getItem('user')
    if (userStr) {
      try {
        user.value = JSON.parse(userStr)
      } catch (e) {
        console.error('Failed to parse user from localStorage', e)
        localStorage.removeItem('user')
      }
    }
  }

  function clearUser(): void {
    user.value = null
    localStorage.removeItem('user')
  }

  return {
    user,
    isAuthenticated,
    setUser,
    loadUser,
    clearUser,
  }
})
