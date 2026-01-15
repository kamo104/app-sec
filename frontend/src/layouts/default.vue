<template>
  <v-app-bar density="compact" flat>
    <v-container fluid class="d-flex align-center pl-4 pr-4">
      <!-- Logo/Brand -->
      <router-link class="text-decoration-none d-flex align-center" to="/">
        <v-icon class="mr-2" color="primary" size="28">mdi-shark-fin</v-icon>
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
              append-icon="mdi-chevron-down"
              class="mr-2"
              prepend-icon="mdi-shield-account"
              variant="text"
            >
              Admin
            </v-btn>
          </template>
          <v-list density="compact">
            <v-list-item prepend-icon="mdi-account-group" to="/admin/users">
              <v-list-item-title>Users</v-list-item-title>
            </v-list-item>
            <v-list-item prepend-icon="mdi-delete-restore" to="/admin/deleted-posts">
              <v-list-item-title>Deleted Posts</v-list-item-title>
            </v-list-item>
          </v-list>
        </v-menu>

        <!-- User menu -->
        <v-menu>
          <template #activator="{ props }">
            <v-btn
              v-bind="props"
              append-icon="mdi-chevron-down"
              variant="text"
            >
              <v-avatar class="mr-2" color="primary" size="28">
                <span class="text-caption">{{ authStore.user?.username?.charAt(0).toUpperCase() }}</span>
              </v-avatar>
              {{ authStore.user?.username }}
            </v-btn>
          </template>
          <v-list density="compact">
            <v-list-item prepend-icon="mdi-account">
              <v-list-item-title>{{ authStore.user?.email }}</v-list-item-title>
              <v-list-item-subtitle>
                <v-chip :color="authStore.isAdmin ? 'warning' : 'default'" size="x-small">
                  {{ authStore.user?.role }}
                </v-chip>
              </v-list-item-subtitle>
            </v-list-item>
            <v-divider />
            <v-list-item prepend-icon="mdi-cog" to="/account">
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
            <v-list-item prepend-icon="mdi-login" to="/login">
              <v-list-item-title>Login</v-list-item-title>
            </v-list-item>
            <v-list-item prepend-icon="mdi-account-plus" to="/register">
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

  <!-- <AppFooter /> -->
</template>

<script lang="ts" setup>
  import { useRouter } from 'vue-router'
  import { logoutUser } from '@/api/client'
  import { useAuthStore } from '@/stores/auth'

  const router = useRouter()
  const authStore = useAuthStore()

  async function logout (): Promise<void> {
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
