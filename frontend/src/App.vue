<template>
  <v-app>
    <v-app-bar app color="primary" density="comfortable">
      <v-app-bar-title class="cursor-pointer" @click="router.push('/')">
        <v-icon icon="mdi-shark" class="mr-2"></v-icon>
        MemeShark
      </v-app-bar-title>

      <v-spacer></v-spacer>

      <!-- User menu - always visible -->
      <v-menu>
        <template v-slot:activator="{ props }">
          <v-btn
            icon="mdi-account-circle"
            v-bind="props"
            color="white"
          ></v-btn>
        </template>
        <v-list>
          <template v-if="authStore.isAuthenticated">
            <v-list-item to="/account">
              <v-list-item-title>
                <v-icon icon="mdi-account" class="mr-2"></v-icon>
                Account
              </v-list-item-title>
            </v-list-item>
            <v-list-item @click="handleLogout">
              <v-list-item-title>
                <v-icon icon="mdi-logout" class="mr-2"></v-icon>
                Logout
              </v-list-item-title>
            </v-list-item>
          </template>
          <template v-else>
            <v-list-item to="/login">
              <v-list-item-title>
                <v-icon icon="mdi-login" class="mr-2"></v-icon>
                Login
              </v-list-item-title>
            </v-list-item>
            <v-list-item to="/register">
              <v-list-item-title>
                <v-icon icon="mdi-account-plus" class="mr-2"></v-icon>
                Register
              </v-list-item-title>
            </v-list-item>
          </template>
        </v-list>
      </v-menu>
    </v-app-bar>

    <v-main>
      <router-view />
    </v-main>
  </v-app>
</template>

<script lang="ts" setup>
import { useAuthStore } from '@/stores/auth'
import { useRouter } from 'vue-router'
import { logoutUser } from '@/services/api'

const authStore = useAuthStore()
const router = useRouter()

const handleLogout = async () => {
  try {
    await logoutUser()
  } catch (error) {
    console.error('Logout error:', error)
  } finally {
    authStore.clearUser()
    router.push('/login')
  }
}
</script>
