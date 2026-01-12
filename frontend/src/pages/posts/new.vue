<template>
  <v-container class="py-4">
    <v-row justify="center">
      <v-col cols="12" md="8" lg="6">
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

        <v-card>
          <v-card-title class="text-h5">
            <v-icon start>mdi-plus-circle</v-icon>
            Create New Post
          </v-card-title>

          <v-divider />

          <v-card-text>
            <v-form ref="form" @submit.prevent="handleSubmit">
              <!-- Title -->
              <v-text-field
                v-model="formData.title"
                label="Title"
                variant="outlined"
                :rules="titleRules"
                :error-messages="fieldErrors.title"
                counter
                maxlength="100"
                class="mb-4"
              />

              <!-- Description -->
              <v-textarea
                v-model="formData.description"
                label="Description (optional)"
                variant="outlined"
                :rules="descriptionRules"
                :error-messages="fieldErrors.description"
                counter
                maxlength="500"
                rows="3"
                class="mb-4"
              />

              <!-- Image upload -->
              <v-file-input
                v-model="formData.image"
                label="Image"
                variant="outlined"
                accept="image/jpeg,image/png,image/gif,image/webp"
                :rules="imageRules"
                :error-messages="fieldErrors.image"
                prepend-icon="mdi-camera"
                show-size
                class="mb-4"
                @update:model-value="onImageChange"
              />

              <!-- Image preview -->
              <v-card
                v-if="imagePreview"
                variant="outlined"
                class="mb-4"
              >
                <v-img
                  :src="imagePreview"
                  max-height="300"
                  contain
                  class="bg-grey-lighten-3"
                />
                <v-card-actions>
                  <v-btn
                    variant="text"
                    color="error"
                    prepend-icon="mdi-delete"
                    @click="clearImage"
                  >
                    Remove Image
                  </v-btn>
                </v-card-actions>
              </v-card>

              <!-- Error message -->
              <v-alert
                v-if="error"
                type="error"
                class="mb-4"
                closable
                @click:close="error = null"
              >
                {{ error }}
              </v-alert>

              <!-- Submit button -->
              <v-btn
                type="submit"
                color="primary"
                size="large"
                block
                :loading="loading"
                :disabled="!isValid"
              >
                <v-icon start>mdi-upload</v-icon>
                Create Post
              </v-btn>
            </v-form>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup lang="ts">
import { ref, reactive, computed } from 'vue'
import { useRouter } from 'vue-router'
import { createPost } from '@/api/client'
import { validate_field } from '@/wasm/field-validator'

const router = useRouter()

// Constants matching backend field-validator/src/lib.rs
const POST_TITLE_MIN_LENGTH = 1
const POST_TITLE_MAX_LENGTH = 100
const POST_DESCRIPTION_MAX_LENGTH = 500
const MAX_IMAGE_SIZE = 5 * 1024 * 1024 // 5MB

const form = ref<HTMLFormElement | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)
const imagePreview = ref<string | null>(null)

const formData = reactive({
  title: '',
  description: '',
  image: null as File[] | null,
})

const fieldErrors = reactive({
  title: [] as string[],
  description: [] as string[],
  image: [] as string[],
})

interface ValidationResult {
  field: string
  errors: string[]
}

const validateField = (fieldType: string, value: string): ValidationResult => {
  const result = validate_field(fieldType, value)
  return JSON.parse(result) as ValidationResult
}

const titleRules = [
  (v: string) => !!v || 'Title is required',
  (v: string) => v.trim().length >= POST_TITLE_MIN_LENGTH || `Title must be at least ${POST_TITLE_MIN_LENGTH} character`,
  (v: string) => v.length <= POST_TITLE_MAX_LENGTH || `Title must be at most ${POST_TITLE_MAX_LENGTH} characters`,
]

const descriptionRules = [
  (v: string) => !v || v.length <= POST_DESCRIPTION_MAX_LENGTH || `Description must be at most ${POST_DESCRIPTION_MAX_LENGTH} characters`,
]

const imageRules = [
  (v: File[] | null) => (v && v.length > 0) || 'Image is required',
  (v: File[] | null) => !v || !v[0] || v[0].size <= MAX_IMAGE_SIZE || 'Image must be less than 5MB',
  (v: File[] | null) => !v || !v[0] || ['image/jpeg', 'image/png', 'image/gif', 'image/webp'].includes(v[0].type) || 'Invalid image format',
]

const isValid = computed(() => {
  return formData.title.trim().length >= POST_TITLE_MIN_LENGTH &&
    formData.title.length <= POST_TITLE_MAX_LENGTH &&
    (!formData.description || formData.description.length <= POST_DESCRIPTION_MAX_LENGTH) &&
    formData.image &&
    formData.image.length > 0 &&
    formData.image[0].size <= MAX_IMAGE_SIZE
})

const onImageChange = (files: File[] | null): void => {
  if (files && files.length > 0) {
    const file = files[0]
    const reader = new FileReader()
    reader.onload = (e) => {
      imagePreview.value = e.target?.result as string
    }
    reader.readAsDataURL(file)
  } else {
    imagePreview.value = null
  }
}

const clearImage = (): void => {
  formData.image = null
  imagePreview.value = null
}

const handleSubmit = async (): Promise<void> => {
  // Clear previous errors
  fieldErrors.title = []
  fieldErrors.description = []
  fieldErrors.image = []
  error.value = null

  // Validate with WASM validators
  const titleValidation = validateField('POST_TITLE', formData.title)
  if (titleValidation.errors.length > 0) {
    fieldErrors.title = titleValidation.errors
    return
  }

  if (formData.description) {
    const descValidation = validateField('POST_DESCRIPTION', formData.description)
    if (descValidation.errors.length > 0) {
      fieldErrors.description = descValidation.errors
      return
    }
  }

  if (!formData.image || formData.image.length === 0) {
    fieldErrors.image = ['Image is required']
    return
  }

  loading.value = true

  try {
    const formDataObj = new FormData()
    formDataObj.append('title', formData.title.trim())
    if (formData.description.trim()) {
      formDataObj.append('description', formData.description.trim())
    }
    formDataObj.append('image', formData.image[0])

    const { data, error: apiError } = await createPost({
      body: formDataObj as unknown as { title: string; description?: string; image: Blob },
    })

    if (data) {
      router.push(`/posts/${data.postId}`)
    } else {
      error.value = 'Failed to create post'
      console.error('Failed to create post:', apiError)
    }
  } catch (e) {
    error.value = 'Failed to create post'
    console.error('Failed to create post:', e)
  } finally {
    loading.value = false
  }
}
</script>
