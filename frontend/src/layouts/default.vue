<template>
  <v-app-bar flat density="compact">
    <v-container class="d-flex align-center">
      <!-- Logo/Brand -->
      <router-link to="/" class="text-decoration-none d-flex align-center">
        <v-icon color="primary" size="28" class="mr-2">mdi-shark-fin</v-icon>
        <span class="text-h6 font-weight-bold text-primary">MemeShark</span>
      </router-link>

      <v-spacer />

      <!-- Navigation links -->
      <template v-if="authStore.isAuthenticated">
        <!-- Admin menu -->
        <v-menu v-if="authStore.isAdmin">
          <template #activator="{ props }">
            <v-btn
              v-bind="props"
              variant="text"
              prepend-icon="mdi-shield-account"
              append-icon="mdi-chevron-down"
              class="mr-2"
            >
              Admin
            </v-btn>
          </template>
          <v-list density="compact">
            <v-list-item to="/admin/users" prepend-icon="mdi-account-group">
              <v-list-item-title>Users</v-list-item-title>
            </v-list-item>
            <v-list-item to="/admin/deleted-posts" prepend-icon="mdi-delete-restore">
              <v-list-item-title>Deleted Posts</v-list-item-title>
            </v-list-item>
          </v-list>
        </v-menu>

        <!-- User menu -->
        <v-menu>
          <template #activator="{ props }">
            <v-btn
              v-bind="props"
              variant="text"
              append-icon="mdi-chevron-down"
            >
              <v-avatar color="primary" size="28" class="mr-2">
                <span class="text-caption">{{ authStore.user?.username?.charAt(0).toUpperCase() }}</span>
              </v-avatar>
              {{ authStore.user?.username }}
            </v-btn>
          </template>
          <v-list density="compact">
            <v-list-item prepend-icon="mdi-account">
              <v-list-item-title>{{ authStore.user?.email }}</v-list-item-title>
              <v-list-item-subtitle>
                <v-chip size="x-small" :color="authStore.isAdmin ? 'warning' : 'default'">
                  {{ authStore.user?.role }}
                </v-chip>
              </v-list-item-subtitle>
            </v-list-item>
            <v-divider />
            <v-list-item to="/account" prepend-icon="mdi-cog">
              <v-list-item-title>Account</v-list-item-title>
            </v-list-item>
            <v-list-item prepend-icon="mdi-logout" @click="logout">
              <v-list-item-title>Logout</v-list-item-title>
            </v-list-item>
          </v-list>
        </v-menu>
      </template>

      <!-- Guest menu -->
      <template v-else>
        <v-menu>
          <template #activator="{ props }">
            <v-btn icon="mdi-account-circle" v-bind="props" />
          </template>
          <v-list density="compact">
            <v-list-item to="/login" prepend-icon="mdi-login">
              <v-list-item-title>Login</v-list-item-title>
            </v-list-item>
            <v-list-item to="/register" prepend-icon="mdi-account-plus">
              <v-list-item-title>Register</v-list-item-title>
            </v-list-item>
          </v-list>
        </v-menu>
      </template>
    </v-container>
  </v-app-bar>

  <v-main>
    <router-view />
  </v-main>

  <AppFooter />
</template>

<script lang="ts" setup>
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { logoutUser } from '@/api/client'

const router = useRouter()
const authStore = useAuthStore()

const logout = async (): Promise<void> => {
  try {
    await logoutUser()
  } catch (e) {
    console.error('Logout error:', e)
  } finally {
    authStore.clearUser()
    router.push('/login')
  }
}
</script>
