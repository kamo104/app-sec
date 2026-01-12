<template>
  <v-container class="py-4">
    <v-row>
      <v-col cols="12">
        <h1 class="text-h4 font-weight-bold mb-4">
          <v-icon start>mdi-delete-restore</v-icon>
          Deleted Posts
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
              <v-btn variant="text" @click="fetchDeletedPosts">Retry</v-btn>
            </template>
          </v-alert>

          <!-- Empty state -->
          <v-card v-else-if="posts.length === 0" class="pa-8 text-center">
            <v-icon size="80" color="grey-lighten-1" class="mb-4">mdi-delete-empty</v-icon>
            <h3 class="text-h6 text-grey">No deleted posts</h3>
            <p class="text-body-2 text-grey-darken-1">
              Posts that are deleted will appear here for restoration.
            </p>
          </v-card>

          <!-- Deleted posts table -->
          <v-card v-else>
            <v-data-table
              :headers="headers"
              :items="posts"
              :items-per-page="10"
              class="elevation-1"
            >
              <template #item.title="{ item }">
                <div class="d-flex align-center">
                  <span>{{ item.title }}</span>
                </div>
              </template>

              <template #item.deletedAt="{ item }">
                {{ formatDate(item.deletedAt) }}
              </template>

              <template #item.actions="{ item }">
                <v-btn
                  color="success"
                  variant="tonal"
                  size="small"
                  prepend-icon="mdi-restore"
                  :loading="restoreLoading[item.postId]"
                  @click="restorePost(item.postId)"
                >
                  Restore
                </v-btn>
              </template>
            </v-data-table>
          </v-card>
        </template>
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { listDeletedPosts, restorePost as apiRestorePost, type DeletedPostResponse } from '@/api/client'
import { useAuthStore } from '@/stores/auth'

const authStore = useAuthStore()

const posts = ref<DeletedPostResponse[]>([])
const loading = ref(false)
const error = ref<string | null>(null)
const restoreLoading = reactive<Record<number, boolean>>({})

const headers = [
  { title: 'ID', key: 'postId', width: '80px' },
  { title: 'Title', key: 'title' },
  { title: 'Author', key: 'username' },
  { title: 'Created', key: 'createdAt' },
  { title: 'Deleted', key: 'deletedAt' },
  { title: 'Actions', key: 'actions', width: '140px', sortable: false },
]

const formatDate = (timestamp: number): string => {
  return new Date(timestamp * 1000).toLocaleString()
}

const fetchDeletedPosts = async (): Promise<void> => {
  loading.value = true
  error.value = null

  try {
    const { data, error: apiError } = await listDeletedPosts()
    if (data) {
      posts.value = data.posts
    } else {
      error.value = 'Failed to load deleted posts'
      console.error('Failed to load deleted posts:', apiError)
    }
  } catch (e) {
    error.value = 'Failed to load deleted posts'
    console.error('Failed to load deleted posts:', e)
  } finally {
    loading.value = false
  }
}

const restorePost = async (postId: number): Promise<void> => {
  restoreLoading[postId] = true

  try {
    const { error: apiError } = await apiRestorePost({ path: { post_id: postId } })

    if (!apiError) {
      posts.value = posts.value.filter(p => p.postId !== postId)
    } else {
      console.error('Failed to restore post:', apiError)
    }
  } catch (e) {
    console.error('Failed to restore post:', e)
  } finally {
    restoreLoading[postId] = false
  }
}

onMounted(() => {
  if (authStore.isAdmin) {
    fetchDeletedPosts()
  }
})
</script>
