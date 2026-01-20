<template>
  <v-container class="py-4">
    <!-- Header with search and create button -->
    <v-row align="center" class="mb-4">
      <v-col cols="12" md="6">
        <h1 class="text-h4 font-weight-bold">MemeShark Feed</h1>
      </v-col>
      <v-col cols="12" md="6">
        <v-row align="center" justify="end">
          <v-col class="flex-grow-1 flex-md-grow-0" cols="auto" style="min-width: 200px;">
            <v-text-field
              v-model="searchQuery"
              clearable
              density="compact"
              hide-details
              :label="t('search_placeholder')"
              prepend-inner-icon="mdi-magnify"
              variant="outlined"
              @click:clear="clearSearch"
              @keyup.enter="performSearch"
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
    <v-row v-if="loading" class="py-8" justify="center">
      <v-progress-circular color="primary" indeterminate size="64" />
    </v-row>

    <!-- Error state -->
    <v-alert v-else-if="error" class="mb-4" type="error">
      {{ error }}
    </v-alert>

    <!-- Empty state -->
    <v-row v-else-if="posts.length === 0" class="py-8" justify="center">
      <v-col class="text-center" cols="12" md="6">
        <v-icon class="mb-4" color="grey-lighten-1" size="80">mdi-image-off</v-icon>
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
        lg="3"
        md="4"
        sm="6"
      >
        <v-card
          class="h-100 d-flex flex-column"
          hover
          :to="`/posts/${post.postId}`"
        >
          <v-img
            class="bg-grey-lighten-3"
            cover
            height="200"
            :src="post.imageUrl"
          >
            <template #placeholder>
              <v-row align="center" class="fill-height ma-0" justify="center">
                <v-progress-circular color="grey-lighten-1" indeterminate />
              </v-row>
            </template>
          </v-img>

          <v-card-title class="text-subtitle-1 font-weight-medium pb-1">
            {{ post.title }}
          </v-card-title>

          <v-card-subtitle class="pb-2">
            <span v-if="post.isUserDeleted" class="text-grey font-italic">
              {{ t('by_author', { author: translate('DELETED_USER', undefined) }) }}
            </span>
            <span v-else>{{ t('by_author', { author: post.username }) }}</span>
          </v-card-subtitle>

          <v-spacer />

          <v-card-actions class="pt-0">
            <v-chip :color="post.score >= 0 ? 'success' : 'error'" size="small" variant="tonal">
              <v-icon size="small" start>{{ post.score >= 0 ? 'mdi-thumb-up' : 'mdi-thumb-down' }}</v-icon>
              {{ post.score }}
            </v-chip>
            <v-chip class="ml-2" size="small" variant="tonal">
              <v-icon size="small" start>mdi-comment</v-icon>
              {{ post.commentCount }}
            </v-chip>
            <v-spacer />
            <span class="text-caption text-grey">{{ formatDate(post.createdAt) }}</span>
          </v-card-actions>
        </v-card>
      </v-col>
    </v-row>

    <!-- Pagination -->
    <v-row v-if="totalPages > 1" class="mt-4" justify="center">
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
  import { computed, onMounted, ref, watch } from 'vue'
  import { useRoute, useRouter } from 'vue-router'
  import { listPosts, type PostResponse, searchPosts } from '@/api/client'
  import { useAuthStore } from '@/stores/auth'
  import { PAGINATION } from '@/utils/constants'
  import { translate, translate_error_code } from '@/wasm/translator.js'

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

  function t (key: string, params?: Record<string, string | number>): string {
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

  function formatDate (timestamp: number): string {
    const date = new Date(timestamp * 1000)
    const now = new Date()
    const diffMs = now.getTime() - date.getTime()
    const diffMins = Math.floor(diffMs / 60_000)
    const diffHours = Math.floor(diffMs / 3_600_000)
    const diffDays = Math.floor(diffMs / 86_400_000)

    if (diffMins < 1) return 'just now'
    if (diffMins < 60) return `${diffMins}m ago`
    if (diffHours < 24) return `${diffHours}h ago`
    if (diffDays < 7) return `${diffDays}d ago`
    return date.toLocaleDateString()
  }

  async function fetchPosts (): Promise<void> {
    loading.value = true
    error.value = null

    try {
      const offset = (currentPage.value - 1) * PAGINATION.POSTS_PER_PAGE
      const { data, error: apiError } = await listPosts({
        query: { limit: PAGINATION.POSTS_PER_PAGE, offset },
      })

      if (data) {
        posts.value = data.posts
        total.value = data.total
        isSearching.value = false
      } else {
        error.value = 'Failed to load posts'
        console.error('Failed to load posts:', apiError)
      }
    } catch (error_) {
      error.value = 'Failed to load posts'
      console.error('Failed to load posts:', error_)
    } finally {
      loading.value = false
    }
  }

  async function performSearch (): Promise<void> {
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
          limit: PAGINATION.POSTS_PER_PAGE,
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
    } catch (error_) {
      error.value = 'Search failed'
      console.error('Search failed:', error_)
    } finally {
      loading.value = false
    }
  }

  function clearSearch (): void {
    searchQuery.value = ''
    isSearching.value = false
    currentPage.value = 1
    router.replace({ query: {} })
    fetchPosts()
  }

  function onPageChange (): void {
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
  watch(() => route.query.q, newQ => {
    if (newQ && newQ !== searchQuery.value) {
      searchQuery.value = newQ as string
      performSearch()
    } else if (!newQ && isSearching.value) {
      clearSearch()
    }
  })
</script>
