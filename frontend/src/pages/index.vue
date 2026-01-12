<template>
  <v-container class="py-4">
    <!-- Header with search and create button -->
    <v-row class="mb-4" align="center">
      <v-col cols="12" md="6">
        <h1 class="text-h4 font-weight-bold">MemeShark Feed</h1>
      </v-col>
      <v-col cols="12" md="6">
        <v-row align="center" justify="end">
          <v-col cols="auto" class="flex-grow-1 flex-md-grow-0" style="min-width: 200px;">
            <v-text-field
              v-model="searchQuery"
              density="compact"
              variant="outlined"
              :label="t('search_placeholder')"
              prepend-inner-icon="mdi-magnify"
              hide-details
              clearable
              @keyup.enter="performSearch"
              @click:clear="clearSearch"
            />
          </v-col>
          <v-col cols="auto">
            <v-btn
              color="primary"
              prepend-icon="mdi-plus"
              :to="authStore.isAuthenticated ? '/posts/new' : '/login'"
            >
              {{ t('create_post') }}
            </v-btn>
          </v-col>
        </v-row>
      </v-col>
    </v-row>

    <!-- Loading state -->
    <v-row v-if="loading" justify="center" class="py-8">
      <v-progress-circular indeterminate color="primary" size="64" />
    </v-row>

    <!-- Error state -->
    <v-alert v-else-if="error" type="error" class="mb-4">
      {{ error }}
    </v-alert>

    <!-- Empty state -->
    <v-row v-else-if="posts.length === 0" justify="center" class="py-8">
      <v-col cols="12" md="6" class="text-center">
        <v-icon size="80" color="grey-lighten-1" class="mb-4">mdi-image-off</v-icon>
        <h3 class="text-h6 text-grey">{{ isSearching ? t('no_search_results') : t('no_posts') }}</h3>
        <p class="text-body-2 text-grey-darken-1">
          {{ isSearching ? t('try_different_search') : t('be_first_to_post') }}
        </p>
      </v-col>
    </v-row>

    <!-- Posts grid -->
    <v-row v-else>
      <v-col
        v-for="post in posts"
        :key="post.postId"
        cols="12"
        sm="6"
        md="4"
        lg="3"
      >
        <v-card
          class="h-100 d-flex flex-column"
          :to="`/posts/${post.postId}`"
          hover
        >
          <v-img
            :src="post.imageUrl"
            height="200"
            cover
            class="bg-grey-lighten-3"
          >
            <template #placeholder>
              <v-row class="fill-height ma-0" align="center" justify="center">
                <v-progress-circular indeterminate color="grey-lighten-1" />
              </v-row>
            </template>
          </v-img>

          <v-card-title class="text-subtitle-1 font-weight-medium pb-1">
            {{ post.title }}
          </v-card-title>

          <v-card-subtitle class="pb-2">
            {{ t('by_author', { author: post.username }) }}
          </v-card-subtitle>

          <v-spacer />

          <v-card-actions class="pt-0">
            <v-chip size="small" :color="post.score >= 0 ? 'success' : 'error'" variant="tonal">
              <v-icon start size="small">{{ post.score >= 0 ? 'mdi-thumb-up' : 'mdi-thumb-down' }}</v-icon>
              {{ post.score }}
            </v-chip>
            <v-chip size="small" variant="tonal" class="ml-2">
              <v-icon start size="small">mdi-comment</v-icon>
              {{ post.commentCount }}
            </v-chip>
            <v-spacer />
            <span class="text-caption text-grey">{{ formatDate(post.createdAt) }}</span>
          </v-card-actions>
        </v-card>
      </v-col>
    </v-row>

    <!-- Pagination -->
    <v-row v-if="totalPages > 1" justify="center" class="mt-4">
      <v-pagination
        v-model="currentPage"
        :length="totalPages"
        :total-visible="5"
        @update:model-value="onPageChange"
      />
    </v-row>
  </v-container>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { listPosts, searchPosts, type PostResponse } from '@/api/client'
import { useAuthStore } from '@/stores/auth'
import { translate_error_code } from '@/wasm/translator.js'
import { PAGINATION } from '@/utils/constants'

const authStore = useAuthStore()
const route = useRoute()
const router = useRouter()

const posts = ref<PostResponse[]>([])
const loading = ref(false)
const error = ref<string | null>(null)
const searchQuery = ref('')
const isSearching = ref(false)
const total = ref(0)
const currentPage = ref(1)

const totalPages = computed(() => Math.ceil(total.value / PAGINATION.POSTS_PER_PAGE))

const t = (key: string, params?: Record<string, string | number>): string => {
  const translations: Record<string, string> = {
    search_placeholder: 'Search memes...',
    create_post: 'Create Post',
    no_posts: 'No memes yet',
    no_search_results: 'No memes found',
    try_different_search: 'Try a different search term',
    be_first_to_post: 'Be the first to share a meme!',
    by_author: `by ${params?.author ?? ''}`,
  }
  return translations[key] ?? key
}

const formatDate = (timestamp: number): string => {
  const date = new Date(timestamp * 1000)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / 60000)
  const diffHours = Math.floor(diffMs / 3600000)
  const diffDays = Math.floor(diffMs / 86400000)

  if (diffMins < 1) return 'just now'
  if (diffMins < 60) return `${diffMins}m ago`
  if (diffHours < 24) return `${diffHours}h ago`
  if (diffDays < 7) return `${diffDays}d ago`
  return date.toLocaleDateString()
}

const fetchPosts = async (): Promise<void> => {
  loading.value = true
  error.value = null

  try {
    const offset = (currentPage.value - 1) * POSTS_PER_PAGE
    const { data, error: apiError } = await listPosts({
      query: { limit: POSTS_PER_PAGE, offset },
    })

    if (data) {
      posts.value = data.posts
      total.value = data.total
      isSearching.value = false
    } else {
      error.value = 'Failed to load posts'
      console.error('Failed to load posts:', apiError)
    }
  } catch (e) {
    error.value = 'Failed to load posts'
    console.error('Failed to load posts:', e)
  } finally {
    loading.value = false
  }
}

const performSearch = async (): Promise<void> => {
  if (!searchQuery.value.trim()) {
    clearSearch()
    return
  }

  loading.value = true
  error.value = null
  isSearching.value = true
  currentPage.value = 1

  try {
    const { data, error: apiError } = await searchPosts({
      query: {
        q: searchQuery.value.trim(),
        limit: POSTS_PER_PAGE,
        offset: 0,
      },
    })

    if (data) {
      posts.value = data.posts
      total.value = data.total
      router.replace({ query: { q: searchQuery.value.trim() } })
    } else {
      error.value = 'Search failed'
      console.error('Search failed:', apiError)
    }
  } catch (e) {
    error.value = 'Search failed'
    console.error('Search failed:', e)
  } finally {
    loading.value = false
  }
}

const clearSearch = (): void => {
  searchQuery.value = ''
  isSearching.value = false
  currentPage.value = 1
  router.replace({ query: {} })
  fetchPosts()
}

const onPageChange = (): void => {
  if (isSearching.value) {
    performSearch()
  } else {
    fetchPosts()
  }
}

onMounted(() => {
  // Check for search query in URL
  const q = route.query.q as string | undefined
  if (q) {
    searchQuery.value = q
    performSearch()
  } else {
    fetchPosts()
  }
})

// Watch for route query changes
watch(() => route.query.q, (newQ) => {
  if (newQ && newQ !== searchQuery.value) {
    searchQuery.value = newQ as string
    performSearch()
  } else if (!newQ && isSearching.value) {
    clearSearch()
  }
})
</script>
