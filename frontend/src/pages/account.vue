<template>
  <v-container class="py-4">
    <v-row justify="center">
      <v-col cols="12" lg="6" md="8">
        <!-- Back button -->
        <v-btn
          class="mb-4"
          color="primary"
          prepend-icon="mdi-arrow-left"
          to="/"
          variant="text"
        >
          Back to Feed
        </v-btn>

        <v-card>
          <v-card-title class="text-h5">
            <v-icon start>mdi-account-cog</v-icon>
            Account Settings
          </v-card-title>

          <v-divider />

          <v-card-text v-if="authStore.isAuthenticated">
            <!-- User Profile Section -->
            <div class="d-flex align-center mb-6">
              <v-avatar class="mr-4" color="primary" size="64">
                <span class="text-h4">{{ userInitial }}</span>
              </v-avatar>
              <div>
                <div class="text-h6">{{ authStore.user?.username }}</div>
                <div class="text-body-2 text-medium-emphasis">{{ authStore.user?.email }}</div>
                <v-chip
                  class="mt-1"
                  :color="authStore.isAdmin ? 'warning' : 'default'"
                  size="small"
                >
                  {{ authStore.user?.role }}
                </v-chip>
              </div>
            </div>

            <v-divider class="mb-6" />

            <!-- Account Information -->
            <div class="text-subtitle-1 font-weight-bold mb-3">Account Information</div>

            <v-list class="mb-6" density="compact">
              <v-list-item prepend-icon="mdi-account">
                <v-list-item-title>Username</v-list-item-title>
                <v-list-item-subtitle>{{ authStore.user?.username }}</v-list-item-subtitle>
              </v-list-item>

              <v-list-item prepend-icon="mdi-email">
                <v-list-item-title>Email</v-list-item-title>
                <v-list-item-subtitle>{{ authStore.user?.email }}</v-list-item-subtitle>
              </v-list-item>

              <v-list-item prepend-icon="mdi-shield-account">
                <v-list-item-title>Role</v-list-item-title>
                <v-list-item-subtitle>{{ authStore.user?.role }}</v-list-item-subtitle>
              </v-list-item>
            </v-list>

            <!-- Session Information -->
            <div class="text-subtitle-1 font-weight-bold mb-3">Session Information</div>

            <v-list class="mb-6" density="compact">
              <v-list-item prepend-icon="mdi-clock-start">
                <v-list-item-title>Session Created</v-list-item-title>
                <v-list-item-subtitle>{{ sessionCreatedFormatted }}</v-list-item-subtitle>
              </v-list-item>

              <v-list-item prepend-icon="mdi-clock-end">
                <v-list-item-title>Session Expires</v-list-item-title>
                <v-list-item-subtitle>{{ sessionExpiresFormatted }}</v-list-item-subtitle>
              </v-list-item>
            </v-list>

            <v-divider class="mb-6" />

            <!-- Actions -->
            <div class="text-subtitle-1 font-weight-bold mb-3">Actions</div>

            <v-btn
              block
              class="mb-2"
              color="primary"
              prepend-icon="mdi-lock-reset"
              to="/forgot-password"
              variant="outlined"
            >
              Change Password
            </v-btn>

            <v-btn
              block
              color="error"
              :loading="loggingOut"
              prepend-icon="mdi-logout"
              variant="tonal"
              @click="handleLogout"
            >
              Logout
            </v-btn>
          </v-card-text>

          <v-card-text v-else>
            <v-alert type="warning" variant="tonal">
              You are not logged in. Please log in to view your account settings.
            </v-alert>
            <v-btn
              block
              class="mt-4"
              color="primary"
              to="/login"
            >
              Go to Login
            </v-btn>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup lang="ts">
  import { computed, ref } from 'vue'
  import { useRouter } from 'vue-router'
  import { logoutUser } from '@/api/client'
  import { useAuthStore } from '@/stores/auth'

  const router = useRouter()
  const authStore = useAuthStore()
  const loggingOut = ref(false)

  const userInitial = computed(() => {
    return authStore.user?.username?.charAt(0).toUpperCase() || '?'
  })

  function formatTimestamp (timestamp: number | null | undefined): string {
    if (!timestamp) return 'N/A'
    return new Date(timestamp * 1000).toLocaleString()
  }

  const sessionCreatedFormatted = computed(() => {
    return formatTimestamp(authStore.user?.sessionCreatedAt)
  })

  const sessionExpiresFormatted = computed(() => {
    return formatTimestamp(authStore.user?.sessionExpiresAt)
  })

  async function handleLogout (): Promise<void> {
    loggingOut.value = true
    try {
      await logoutUser()
    } catch (error) {
      console.error('Logout error:', error)
    } finally {
      authStore.clearUser()
      loggingOut.value = false
      router.push('/login')
    }
  }
</script>
