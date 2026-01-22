<template>
  <v-container class="py-4">
    <v-row>
      <v-col cols="12">
        <h1 class="text-h4 font-weight-bold mb-4">
          <v-icon start>mdi-shield-account</v-icon>
          User Management
        </h1>

        <!-- Access denied for non-admins -->
        <v-alert v-if="!authStore.isAdmin" class="mb-4" type="error">
          Access denied. Admin privileges required.
        </v-alert>

        <template v-else>
          <!-- Loading state -->
          <v-card v-if="loading" class="pa-8 text-center">
            <v-progress-circular color="primary" indeterminate size="64" />
          </v-card>

          <!-- Error state -->
          <v-alert v-else-if="error" class="mb-4" type="error">
            {{ error }}
            <template #append>
              <v-btn variant="text" @click="fetchUsers">Retry</v-btn>
            </template>
          </v-alert>

          <!-- Users table -->
          <v-card v-else>
            <v-data-table
              class="elevation-1"
              :headers="headers"
              :items="users"
              :items-per-page="10"
            >
              <template #item.username="{ item }">
                <span v-if="item.isDeleted" class="text-grey">
                  {{ translate('DELETED_USER', undefined) }}
                </span>
                <span v-else>{{ item.username }}</span>
              </template>

              <template #item.role="{ item }">
                <v-select
                  density="compact"
                  :disabled="item.isDeleted || String(item.userId) === authStore.user?.username || roleLoading[item.userId]"
                  hide-details
                  :items="roleOptions"
                  :model-value="item.role"
                  style="max-width: 120px;"
                  variant="outlined"
                  @update:model-value="(newRole: string) => updateRole(String(item.userId), newRole as 'user' | 'admin')"
                />
              </template>

              <template #item.actions="{ item }">
                <v-btn
                  v-if="item.isDeleted"
                  color="success"
                  icon="mdi-restore"
                  :loading="restoreLoading[item.userId]"
                  size="small"
                  variant="text"
                  @click="restoreUser(item)"
                />
                <v-btn
                  v-else
                  color="error"
                  :disabled="String(item.userId) === authStore.user?.username"
                  icon="mdi-delete"
                  size="small"
                  variant="text"
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
          <v-btn color="error" :loading="deleteLoading" variant="flat" @click="deleteUserConfirmed">
            Delete
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-container>
</template>

<script setup lang="ts">
  import { onMounted, reactive, ref } from 'vue'
  import { deleteUser as apiDeleteUser, listUsers, restoreUser as apiRestoreUser, updateUserRole, type UserInfoResponse } from '@/api/client'
  import { useAuthStore } from '@/stores/auth'
  import { translate } from '@/wasm/translator.js'

  const authStore = useAuthStore()

  const users = ref<UserInfoResponse[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)
  const roleLoading = reactive<Record<string | number, boolean>>({})
  const restoreLoading = reactive<Record<string | number, boolean>>({})
  const deleteDialog = ref(false)
  const deleteLoading = ref(false)
  const userToDelete = ref<UserInfoResponse | null>(null)

  const headers = [
    { title: 'ID', key: 'userId', width: '80px' },
    { title: 'Username', key: 'username' },
    { title: 'Email', key: 'email' },
    { title: 'Role', key: 'role' },
    { title: 'Actions', key: 'actions', width: '100px', sortable: false },
  ]

  const roleOptions = [
    { title: 'User', value: 'user' },
    { title: 'Admin', value: 'admin' },
  ]

  async function fetchUsers (): Promise<void> {
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
    } catch (error_) {
      error.value = 'Failed to load users'
      console.error('Failed to load users:', error_)
    } finally {
      loading.value = false
    }
  }

  async function updateRole (userId: string, newRole: 'user' | 'admin'): Promise<void> {
    roleLoading[userId] = true

    try {
      const { error: apiError } = await updateUserRole({
        path: { user_id: Number(userId) },
        body: { role: newRole },
      })

      if (apiError) {
        console.error('Failed to update role:', apiError)
      } else {
        const user = users.value.find(u => String(u.userId) === userId)
        if (user) {
          user.role = newRole
        }
      }
    } catch (error_) {
      console.error('Failed to update role:', error_)
    } finally {
      roleLoading[userId] = false
    }
  }

  function confirmDelete (user: UserInfoResponse): void {
    userToDelete.value = user
    deleteDialog.value = true
  }

  async function deleteUserConfirmed (): Promise<void> {
    if (!userToDelete.value) return

    deleteLoading.value = true

    try {
      const { error: apiError } = await apiDeleteUser({ path: { user_id: Number(userToDelete.value!.userId) } })

      if (apiError) {
        console.error('Failed to delete user:', apiError)
      } else {
        // Mark the user as deleted instead of removing from the list
        const user = users.value.find(u => String(u.userId) === String(userToDelete.value!.userId))
        if (user) {
          user.isDeleted = true
        }
        deleteDialog.value = false
      }
    } catch (error_) {
      console.error('Failed to delete user:', error_)
    } finally {
      deleteLoading.value = false
    }
  }

  async function restoreUser (user: UserInfoResponse): Promise<void> {
    restoreLoading[user.userId] = true

    try {
      const { error: apiError } = await apiRestoreUser({ path: { user_id: Number(user.userId) } })

      if (apiError) {
        console.error('Failed to restore user:', apiError)
      } else {
        user.isDeleted = false
      }
    } catch (error_) {
      console.error('Failed to restore user:', error_)
    } finally {
      restoreLoading[user.userId] = false
    }
  }

  onMounted(() => {
    if (authStore.isAdmin) {
      fetchUsers()
    }
  })
</script>
