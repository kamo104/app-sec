<template>
  <v-container class="py-4">
    <v-row justify="center">
      <v-col cols="12" lg="6" md="8">
        <!-- Back button -->
        <v-btn
          class="mb-4"
          color="primary"
          prepend-icon="mdi-arrow-left"
          :to="`/posts/${postId}`"
          variant="text"
        >
          Back to Post
        </v-btn>

        <!-- Loading state -->
        <v-row v-if="loadingPost" class="py-8" justify="center">
          <v-progress-circular color="primary" indeterminate size="64" />
        </v-row>

        <!-- Not authorized -->
        <v-alert v-else-if="notAuthorized" class="mb-4" type="error">
          {{ translate('FORBIDDEN', undefined) }}
        </v-alert>

        <!-- Post not found -->
        <v-alert v-else-if="notFound" class="mb-4" type="error">
          {{ translate('POST_NOT_FOUND', undefined) }}
        </v-alert>

        <!-- Edit form -->
        <v-card v-else-if="post">
          <v-card-title class="text-h5">
            <v-icon start>mdi-pencil</v-icon>
            Edit Post
          </v-card-title>

          <v-divider />

          <v-card-text>
            <v-form ref="form" @submit.prevent="handleSubmit">
              <!-- Title -->
              <v-text-field
                v-model="formData.title"
                class="mb-4"
                counter
                :error-messages="fieldErrors.title"
                label="Title"
                :maxlength="POST_TITLE_MAX_LENGTH"
                :rules="titleRules"
                variant="outlined"
              />

              <!-- Description -->
              <v-textarea
                v-model="formData.description"
                class="mb-4"
                counter
                :error-messages="fieldErrors.description"
                label="Description (optional)"
                :maxlength="POST_DESCRIPTION_MAX_LENGTH"
                rows="3"
                :rules="descriptionRules"
                variant="outlined"
              />

              <!-- Current image preview -->
              <v-card class="mb-4" variant="outlined">
                <v-card-subtitle class="pt-2">Current Image</v-card-subtitle>
                <v-img
                  class="bg-grey-lighten-3"
                  contain
                  max-height="300"
                  :src="post.imageUrl"
                />
              </v-card>

              <!-- Error message -->
              <v-alert
                v-if="error"
                class="mb-4"
                closable
                type="error"
                @click:close="error = null"
              >
                {{ error }}
              </v-alert>

              <!-- Success message -->
              <v-alert
                v-if="success"
                class="mb-4"
                closable
                type="success"
                @click:close="success = null"
              >
                {{ success }}
              </v-alert>

              <!-- Submit button -->
              <v-btn
                block
                color="primary"
                :disabled="!isValid || !hasChanges"
                :loading="loading"
                size="large"
                type="submit"
              >
                <v-icon start>mdi-content-save</v-icon>
                Save Changes
              </v-btn>
            </v-form>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup lang="ts">
  import { computed, onMounted, reactive, ref } from 'vue'
  import { useRoute, useRouter } from 'vue-router'
  import { getPost, type PostErrorResponse, type PostResponse, updatePost } from '@/api/client'
  import { useAuthStore } from '@/stores/auth'
  import { VALIDATION_CONSTANTS } from '@/utils/validation.js'
  import { validate_field } from '@/wasm/field-validator.js'
  import { translate } from '@/wasm/translator.js'
  import { translate_error_code, translate_field_validation_error } from '@/wasm/translator.js'

  const route = useRoute()
  const router = useRouter()
  const authStore = useAuthStore()

  const postId = computed(() => Number(route.params.id))

  // Constants from WASM field-validator
  const POST_TITLE_MAX_LENGTH = VALIDATION_CONSTANTS.POST_TITLE_MAX_LENGTH
  const POST_DESCRIPTION_MAX_LENGTH = VALIDATION_CONSTANTS.POST_DESCRIPTION_MAX_LENGTH

  const form = ref<HTMLFormElement | null>(null)
  const loading = ref(false)
  const loadingPost = ref(true)
  const error = ref<string | null>(null)
  const success = ref<string | null>(null)
  const notAuthorized = ref(false)
  const notFound = ref(false)

  const post = ref<PostResponse | null>(null)

  interface FormData {
    title: string
    description: string
  }

  const formData = reactive<FormData>({
    title: '',
    description: '',
  })

  // Store original values to detect changes
  const originalData = reactive<FormData>({
    title: '',
    description: '',
  })

  const fieldErrors = reactive({
    title: [] as string[],
    description: [] as string[],
  })

  interface ValidationResult {
    field: string
    errors: string[]
  }

  function validateFieldWasm (fieldType: string, value: string): ValidationResult {
    const result = validate_field(fieldType, value)
    return JSON.parse(result) as ValidationResult
  }

  function translateValidationErrors (fieldType: string, errors: string[]): string[] {
    return errors.map(err => translate_field_validation_error(fieldType, err, 'en'))
  }

  // Real-time validation using WASM validator
  const titleRules = [
    (v: string) => {
      const result = validateFieldWasm('POST_TITLE', v)
      if (result.errors.length > 0) {
        return translateValidationErrors('POST_TITLE', result.errors).join(', ')
      }
      return true
    },
  ]

  const descriptionRules = [
    (v: string) => {
      if (!v) return true // Optional field
      const result = validateFieldWasm('POST_DESCRIPTION', v)
      if (result.errors.length > 0) {
        return translateValidationErrors('POST_DESCRIPTION', result.errors).join(', ')
      }
      return true
    },
  ]

  const isValid = computed(() => {
    const titleResult = validateFieldWasm('POST_TITLE', formData.title)
    const descResult = formData.description ? validateFieldWasm('POST_DESCRIPTION', formData.description) : { errors: [] }

    return titleResult.errors.length === 0 && descResult.errors.length === 0
  })

  const hasChanges = computed(() => {
    return formData.title !== originalData.title || formData.description !== originalData.description
  })

  async function fetchPost (): Promise<void> {
    loadingPost.value = true
    notAuthorized.value = false
    notFound.value = false

    try {
      const { data, error: apiError } = await getPost({
        path: { post_id: postId.value },
      })

      if (data) {
        post.value = data

        // Check if user can edit this post
        const canEdit = data.username === authStore.user?.username || authStore.isAdmin
        if (!canEdit) {
          notAuthorized.value = true
          return
        }

        // Populate form with current values
        formData.title = data.title
        formData.description = data.description || ''
        originalData.title = data.title
        originalData.description = data.description || ''
      } else if (apiError) {
        const err = apiError as PostErrorResponse
        if (err.error === 'NOT_FOUND') {
          notFound.value = true
        } else {
          error.value = translate_error_code(err.error, undefined)
        }
      }
    } catch (e) {
      error.value = translate_error_code('INTERNAL', undefined)
      console.error('Failed to fetch post:', e)
    } finally {
      loadingPost.value = false
    }
  }

  async function handleSubmit (): Promise<void> {
    // Clear previous errors
    fieldErrors.title = []
    fieldErrors.description = []
    error.value = null
    success.value = null

    // Validate with WASM validators
    const titleValidation = validateFieldWasm('POST_TITLE', formData.title)
    if (titleValidation.errors.length > 0) {
      fieldErrors.title = translateValidationErrors('POST_TITLE', titleValidation.errors)
      return
    }

    if (formData.description) {
      const descValidation = validateFieldWasm('POST_DESCRIPTION', formData.description)
      if (descValidation.errors.length > 0) {
        fieldErrors.description = translateValidationErrors('POST_DESCRIPTION', descValidation.errors)
        return
      }
    }

    loading.value = true

    try {
      const { data, error: apiError } = await updatePost({
        path: { post_id: postId.value },
        body: {
          title: formData.title.trim(),
          description: formData.description.trim() || null,
        },
      })

      if (data !== undefined && apiError === undefined) {
        // Update original data to reflect saved state
        originalData.title = formData.title
        originalData.description = formData.description
        success.value = translate('POST_UPDATED', undefined)

        // Navigate back to post after short delay
        setTimeout(() => {
          router.push(`/posts/${postId.value}`)
        }, 1000)
      } else if (apiError) {
        handleUpdateError(apiError as PostErrorResponse)
      } else {
        error.value = translate_error_code('INTERNAL', undefined)
      }
    } catch (e) {
      error.value = translate_error_code('INTERNAL', undefined)
      console.error('Failed to update post:', e)
    } finally {
      loading.value = false
    }
  }

  function handleUpdateError (apiError: PostErrorResponse): void {
    console.error('Update post error:', apiError)

    if (apiError.error === 'VALIDATION' && apiError.validation) {
      for (const fieldError of apiError.validation.fieldErrors) {
        const translatedErrors = translateValidationErrors(fieldError.field, fieldError.errors)

        if (fieldError.field === 'POST_TITLE') {
          fieldErrors.title = translatedErrors
        } else if (fieldError.field === 'POST_DESCRIPTION') {
          fieldErrors.description = translatedErrors
        }
      }
      error.value = translate_error_code('VALIDATION', undefined)
    } else {
      error.value = translate_error_code(apiError.error, undefined)
    }
  }

  onMounted(() => {
    if (!authStore.isAuthenticated) {
      router.push('/login')
      return
    }
    fetchPost()
  })
</script>
