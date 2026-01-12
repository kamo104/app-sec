<template>
  <v-container class="py-4">
    <!-- Back button -->
    <v-btn
      variant="text"
      color="primary"
      prepend-icon="mdi-arrow-left"
      class="mb-4"
      to="/"
    >
      Back to Feed
    </v-btn>

    <!-- Loading state -->
    <v-row v-if="loading" justify="center" class="py-8">
      <v-progress-circular indeterminate color="primary" size="64" />
    </v-row>

    <!-- Error state -->
    <v-alert v-else-if="error" type="error" class="mb-4">
      {{ error }}
      <template #append>
        <v-btn variant="text" @click="fetchPost">Retry</v-btn>
      </template>
    </v-alert>

    <!-- Post content -->
    <template v-else-if="post">
      <v-row>
        <!-- Main post column -->
        <v-col cols="12" lg="8">
          <v-card>
            <!-- Post image -->
            <v-img
              :src="post.imageUrl"
              max-height="600"
              contain
              class="bg-grey-lighten-3"
            >
              <template #placeholder>
                <v-row class="fill-height ma-0" align="center" justify="center">
                  <v-progress-circular indeterminate color="grey-lighten-1" />
                </v-row>
              </template>
            </v-img>

            <v-card-title class="text-h5 font-weight-bold pt-4">
              {{ post.title }}
            </v-card-title>

            <v-card-subtitle class="pb-2">
              Posted by <strong>{{ post.username }}</strong> {{ formatDate(post.createdAt) }}
              <span v-if="post.updatedAt"> (edited)</span>
            </v-card-subtitle>

            <v-card-text v-if="post.description" class="text-body-1">
              {{ post.description }}
            </v-card-text>

            <v-divider />

            <!-- Actions bar -->
            <v-card-actions>
              <!-- Rating buttons -->
              <v-btn-group variant="outlined" divided>
                <v-btn
                  :color="post.userRating === 1 ? 'success' : undefined"
                  :disabled="!authStore.isAuthenticated || ratingLoading"
                  @click="ratePost(1)"
                >
                  <v-icon start>mdi-thumb-up</v-icon>
                  Upvote
                </v-btn>
                <v-btn disabled variant="text" class="px-2">
                  {{ post.score }}
                </v-btn>
                <v-btn
                  :color="post.userRating === -1 ? 'error' : undefined"
                  :disabled="!authStore.isAuthenticated || ratingLoading"
                  @click="ratePost(-1)"
                >
                  <v-icon start>mdi-thumb-down</v-icon>
                  Downvote
                </v-btn>
              </v-btn-group>

              <v-spacer />

              <!-- Owner/Admin actions -->
              <template v-if="canEdit">
                <v-btn
                  variant="text"
                  color="primary"
                  prepend-icon="mdi-pencil"
                  :to="`/posts/${post.postId}/edit`"
                >
                  Edit
                </v-btn>
                <v-btn
                  variant="text"
                  color="error"
                  prepend-icon="mdi-delete"
                  :loading="deleteLoading"
                  @click="confirmDelete"
                >
                  Delete
                </v-btn>
              </template>
            </v-card-actions>
          </v-card>

          <!-- Comments section -->
          <v-card class="mt-4">
            <v-card-title>
              <v-icon start>mdi-comment-multiple</v-icon>
              Comments ({{ comments.length }})
            </v-card-title>

            <v-divider />

            <!-- Add comment form -->
            <v-card-text v-if="authStore.isAuthenticated">
              <v-textarea
                v-model="newComment"
                variant="outlined"
                label="Add a comment..."
                rows="2"
                :disabled="commentLoading"
                :error-messages="commentError"
                counter
                maxlength="1000"
              />
              <v-btn
                color="primary"
                :loading="commentLoading"
                :disabled="!newComment.trim()"
                @click="submitComment"
              >
                Post Comment
              </v-btn>
            </v-card-text>
            <v-card-text v-else class="text-center py-4">
              <router-link to="/login">Login</router-link> to comment
            </v-card-text>

            <v-divider />

            <!-- Comments list -->
            <v-list v-if="comments.length > 0">
              <v-list-item
                v-for="comment in comments"
                :key="comment.commentId"
                class="py-3"
              >
                <template #prepend>
                  <v-avatar color="primary" size="40">
                    <span class="text-h6">{{ comment.username.charAt(0).toUpperCase() }}</span>
                  </v-avatar>
                </template>

                <v-list-item-title class="font-weight-medium">
                  {{ comment.username }}
                  <span class="text-caption text-grey ml-2">{{ formatDate(comment.createdAt) }}</span>
                </v-list-item-title>

                <v-list-item-subtitle class="text-body-2 mt-1" style="white-space: pre-wrap;">
                  {{ comment.content }}
                </v-list-item-subtitle>

                <template #append>
                  <v-btn
                    v-if="canDeleteComment(comment)"
                    icon="mdi-delete"
                    variant="text"
                    color="error"
                    size="small"
                    @click="deleteComment(comment.commentId)"
                  />
                </template>
              </v-list-item>
            </v-list>
            <v-card-text v-else class="text-center text-grey py-8">
              No comments yet. Be the first to comment!
            </v-card-text>
          </v-card>
        </v-col>

        <!-- Sidebar -->
        <v-col cols="12" lg="4">
          <v-card>
            <v-card-title>Post Info</v-card-title>
            <v-list>
              <v-list-item>
                <template #prepend>
                  <v-icon>mdi-account</v-icon>
                </template>
                <v-list-item-title>Author</v-list-item-title>
                <v-list-item-subtitle>{{ post.username }}</v-list-item-subtitle>
              </v-list-item>
              <v-list-item>
                <template #prepend>
                  <v-icon>mdi-calendar</v-icon>
                </template>
                <v-list-item-title>Posted</v-list-item-title>
                <v-list-item-subtitle>{{ new Date(post.createdAt * 1000).toLocaleString() }}</v-list-item-subtitle>
              </v-list-item>
              <v-list-item>
                <template #prepend>
                  <v-icon>mdi-thumb-up</v-icon>
                </template>
                <v-list-item-title>Score</v-list-item-title>
                <v-list-item-subtitle>{{ post.score }} points</v-list-item-subtitle>
              </v-list-item>
              <v-list-item>
                <template #prepend>
                  <v-icon>mdi-comment</v-icon>
                </template>
                <v-list-item-title>Comments</v-list-item-title>
                <v-list-item-subtitle>{{ post.commentCount }}</v-list-item-subtitle>
              </v-list-item>
            </v-list>
          </v-card>
        </v-col>
      </v-row>
    </template>

    <!-- Delete confirmation dialog -->
    <v-dialog v-model="deleteDialog" max-width="400">
      <v-card>
        <v-card-title>Delete Post?</v-card-title>
        <v-card-text>
          Are you sure you want to delete this post? This action cannot be undone.
        </v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn variant="text" @click="deleteDialog = false">Cancel</v-btn>
          <v-btn color="error" variant="flat" :loading="deleteLoading" @click="deletePostConfirmed">
            Delete
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-container>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  getPost,
  listComments,
  createComment,
  deleteComment as apiDeleteComment,
  ratePost as apiRatePost,
  removeRating,
  deletePost as apiDeletePost,
  type PostResponse,
  type CommentResponse,
} from '@/api/client'
import { useAuthStore } from '@/stores/auth'

const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()

const postId = computed(() => Number(route.params.id))

const post = ref<PostResponse | null>(null)
const comments = ref<CommentResponse[]>([])
const loading = ref(false)
const error = ref<string | null>(null)

const newComment = ref('')
const commentLoading = ref(false)
const commentError = ref<string | null>(null)

const ratingLoading = ref(false)
const deleteLoading = ref(false)
const deleteDialog = ref(false)

const canEdit = computed(() => {
  if (!post.value || !authStore.user) return false
  return post.value.userId === authStore.user.username || authStore.isAdmin
})

const canDeleteComment = (comment: CommentResponse): boolean => {
  if (!authStore.user) return false
  return comment.username === authStore.user.username || authStore.isAdmin
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

const fetchPost = async (): Promise<void> => {
  loading.value = true
  error.value = null

  try {
    const { data, error: apiError } = await getPost({ path: { postId: postId.value } })
    if (data) {
      post.value = data
    } else {
      error.value = 'Failed to load post'
      console.error('Failed to load post:', apiError)
    }
  } catch (e) {
    error.value = 'Failed to load post'
    console.error('Failed to load post:', e)
  } finally {
    loading.value = false
  }
}

const fetchComments = async (): Promise<void> => {
  try {
    const { data } = await listComments({ path: { postId: postId.value } })
    if (data) {
      comments.value = data.comments
    }
  } catch (e) {
    console.error('Failed to load comments:', e)
  }
}

const submitComment = async (): Promise<void> => {
  if (!newComment.value.trim()) return

  commentLoading.value = true
  commentError.value = null

  try {
    const { data, error: apiError } = await createComment({
      path: { postId: postId.value },
      body: { content: newComment.value.trim() },
    })

    if (data) {
      newComment.value = ''
      await fetchComments()
      if (post.value) {
        post.value.commentCount++
      }
    } else {
      commentError.value = 'Failed to post comment'
      console.error('Failed to post comment:', apiError)
    }
  } catch (e) {
    commentError.value = 'Failed to post comment'
    console.error('Failed to post comment:', e)
  } finally {
    commentLoading.value = false
  }
}

const deleteComment = async (commentId: number): Promise<void> => {
  try {
    const { error: apiError } = await apiDeleteComment({ path: { commentId } })
    if (!apiError) {
      comments.value = comments.value.filter(c => c.commentId !== commentId)
      if (post.value) {
        post.value.commentCount--
      }
    }
  } catch (e) {
    console.error('Failed to delete comment:', e)
  }
}

const ratePost = async (value: 1 | -1): Promise<void> => {
  if (!post.value) return

  ratingLoading.value = true

  try {
    // If clicking same rating, remove it
    if (post.value.userRating === value) {
      const { error: apiError } = await removeRating({ path: { postId: postId.value } })
      if (!apiError) {
        post.value.score -= value
        post.value.userRating = null
      }
    } else {
      const { error: apiError } = await apiRatePost({
        path: { postId: postId.value },
        body: { value },
      })
      if (!apiError) {
        // Adjust score based on previous rating
        if (post.value.userRating !== null) {
          post.value.score -= post.value.userRating
        }
        post.value.score += value
        post.value.userRating = value
      }
    }
  } catch (e) {
    console.error('Failed to rate post:', e)
  } finally {
    ratingLoading.value = false
  }
}

const confirmDelete = (): void => {
  deleteDialog.value = true
}

const deletePostConfirmed = async (): Promise<void> => {
  if (!post.value) return

  deleteLoading.value = true

  try {
    const { error: apiError } = await apiDeletePost({ path: { postId: postId.value } })
    if (!apiError) {
      router.push('/')
    } else {
      console.error('Failed to delete post:', apiError)
    }
  } catch (e) {
    console.error('Failed to delete post:', e)
  } finally {
    deleteLoading.value = false
    deleteDialog.value = false
  }
}

onMounted(() => {
  fetchPost()
  fetchComments()
})
</script>
