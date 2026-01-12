<template>
  <v-container class="py-4">
    <v-row>
      <v-col cols="12">
        <h1 class="text-h4 font-weight-bold mb-4">
          <v-icon start>mdi-shield-account</v-icon>
          User Management
        </h1>

        <!-- Access denied for non-admins -->
        <v-alert v-if="!authStore.isAdmin" type="error" class="mb-4">
          Access denied. Admin privileges required.
        </v-alert>

        <template v-else>
          <!-- Loading state -->
          <v-card v-if="loading" class="pa-8 text-center">
            <v-progress-circular indeterminate color="primary" size="64" />
          </v-card>

          <!-- Error state -->
          <v-alert v-else-if="error" type="error" class="mb-4">
            {{ error }}
            <template #append>
              <v-btn variant="text" @click="fetchUsers">Retry</v-btn>
            </template>
          </v-alert>

          <!-- Users table -->
          <v-card v-else>
            <v-data-table
              :headers="headers"
              :items="users"
              :items-per-page="10"
              class="elevation-1"
            >
              <template #item.role="{ item }">
                <v-select
                  :model-value="item.role"
                  :items="roleOptions"
                  density="compact"
                  variant="outlined"
                  hide-details
                  :disabled="item.userId === authStore.user?.username || roleLoading[item.userId]"
                  style="max-width: 120px;"
                  @update:model-value="(newRole: string) => updateRole(item.userId, newRole as 'user' | 'admin')"
                />
              </template>

              <template #item.createdAt="{ item }">
                {{ formatDate(item.createdAt) }}
              </template>

              <template #item.actions="{ item }">
                <v-btn
                  icon="mdi-delete"
                  variant="text"
                  color="error"
                  size="small"
                  :disabled="item.userId === authStore.user?.username"
                  @click="confirmDelete(item)"
                />
              </template>
            </v-data-table>
          </v-card>
        </template>
      </v-col>
    </v-row>

    <!-- Delete confirmation dialog -->
    <v-dialog v-model="deleteDialog" max-width="400">
      <v-card>
        <v-card-title>Delete User?</v-card-title>
        <v-card-text>
          Are you sure you want to delete user <strong>{{ userToDelete?.username }}</strong>?
          This will also delete all their posts and comments.
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="text" @click="deleteDialog = false">Cancel</v-btn>
          <v-btn color="error" variant="flat" :loading="deleteLoading" @click="deleteUserConfirmed">
            Delete
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-container>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { listUsers, updateUserRole, deleteUser as apiDeleteUser, type UserInfoResponse } from '@/api/client'
import { useAuthStore } from '@/stores/auth'

const authStore = useAuthStore()

const users = ref<UserInfoResponse[]>([])
const loading = ref(false)
const error = ref<string | null>(null)
const roleLoading = reactive<Record<number, boolean>>({})
const deleteDialog = ref(false)
const deleteLoading = ref(false)
const userToDelete = ref<UserInfoResponse | null>(null)

const headers = [
  { title: 'ID', key: 'userId', width: '80px' },
  { title: 'Username', key: 'username' },
  { title: 'Email', key: 'email' },
  { title: 'Role', key: 'role', width: '150px' },
  { title: 'Created', key: 'createdAt' },
  { title: 'Actions', key: 'actions', width: '100px', sortable: false },
]

const roleOptions = [
  { title: 'User', value: 'user' },
  { title: 'Admin', value: 'admin' },
]

const formatDate = (timestamp: number): string => {
  return new Date(timestamp * 1000).toLocaleDateString()
}

const fetchUsers = async (): Promise<void> => {
  loading.value = true
  error.value = null

  try {
    const { data, error: apiError } = await listUsers()
    if (data) {
      users.value = data.users
    } else {
      error.value = 'Failed to load users'
      console.error('Failed to load users:', apiError)
    }
  } catch (e) {
    error.value = 'Failed to load users'
    console.error('Failed to load users:', e)
  } finally {
    loading.value = false
  }
}

const updateRole = async (userId: number, newRole: 'user' | 'admin'): Promise<void> => {
  roleLoading[userId] = true

  try {
    const { error: apiError } = await updateUserRole({
      path: { userId },
      body: { role: newRole },
    })

    if (!apiError) {
      const user = users.value.find(u => u.userId === userId)
      if (user) {
        user.role = newRole
      }
    } else {
      console.error('Failed to update role:', apiError)
    }
  } catch (e) {
    console.error('Failed to update role:', e)
  } finally {
    roleLoading[userId] = false
  }
}

const confirmDelete = (user: UserInfoResponse): void => {
  userToDelete.value = user
  deleteDialog.value = true
}

const deleteUserConfirmed = async (): Promise<void> => {
  if (!userToDelete.value) return

  deleteLoading.value = true

  try {
    const { error: apiError } = await apiDeleteUser({
      path: { userId: userToDelete.value.userId },
    })

    if (!apiError) {
      users.value = users.value.filter(u => u.userId !== userToDelete.value!.userId)
    } else {
      console.error('Failed to delete user:', apiError)
    }
  } catch (e) {
    console.error('Failed to delete user:', e)
  } finally {
    deleteLoading.value = false
    deleteDialog.value = false
    userToDelete.value = null
  }
}

onMounted(() => {
  if (authStore.isAdmin) {
    fetchUsers()
  }
})
</script>
