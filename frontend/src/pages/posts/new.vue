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
                :maxlength="POST_TITLE_MAX_LENGTH"
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
                :maxlength="POST_DESCRIPTION_MAX_LENGTH"
                rows="3"
                class="mb-4"
              />

              <!-- Image upload -->
              <v-file-input
                v-model="formData.image"
                label="Image"
                variant="outlined"
                :accept="IMAGE_ALLOWED_MIME_TYPES.join(',')"
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
import { createPost, type PostErrorResponse } from '@/api/client'
import {
  validate_field,
  get_post_title_max_length,
  get_post_description_max_length,
  get_image_allowed_mime_types,
  validate_image_size,
  validate_image_mime,
} from '@/wasm/field-validator.js'
import { translate_error_code, translate_field_validation_error } from '@/wasm/translator.js'
import { VALIDATION_CONSTANTS } from '@/utils/validation.js'

const router = useRouter()

// Constants from WASM field-validator (single source of truth)
const POST_TITLE_MAX_LENGTH = VALIDATION_CONSTANTS.POST_TITLE_MAX_LENGTH
const POST_DESCRIPTION_MAX_LENGTH = VALIDATION_CONSTANTS.POST_DESCRIPTION_MAX_LENGTH
const IMAGE_ALLOWED_MIME_TYPES = VALIDATION_CONSTANTS.IMAGE_ALLOWED_MIME_TYPES

const form = ref<HTMLFormElement | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)
const imagePreview = ref<string | null>(null)

const formData = reactive({
  title: '',
  description: '',
  image: null as File[] | File | null | undefined,
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

const validateFieldWasm = (fieldType: string, value: string): ValidationResult => {
  const result = validate_field(fieldType, value)
  return JSON.parse(result) as ValidationResult
}

const translateValidationErrors = (fieldType: string, errors: string[]): string[] => {
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

const hasValidImage = (v: File[] | File | null | undefined): v is File[] | File => {
  if (!v) return false
  if (Array.isArray(v)) return v.length > 0 && v[0] instanceof File
  return v instanceof File
}

const getImageFile = (v: File[] | File | null | undefined): File | null => {
  if (!v) return null
  if (Array.isArray(v)) return v.length > 0 ? v[0] : null
  return v instanceof File ? v : null
}

const imageRules = [
  (v: File[] | File | null | undefined) => hasValidImage(v) || translate_error_code('IMAGE_REQUIRED', undefined),
  (v: File[] | File | null | undefined) => {
    const file = getImageFile(v)
    return !file || validate_image_size(file.size) || translate_error_code('INVALID_IMAGE', undefined)
  },
  (v: File[] | File | null | undefined) => {
    const file = getImageFile(v)
    return !file || validate_image_mime(file.type) || translate_error_code('INVALID_IMAGE', undefined)
  },
]

const isValid = computed(() => {
  const file = getImageFile(formData.image)
  const titleResult = validateFieldWasm('POST_TITLE', formData.title)
  const descResult = formData.description ? validateFieldWasm('POST_DESCRIPTION', formData.description) : { errors: [] }
  
  if (!file) return false
  
  return titleResult.errors.length === 0 &&
    descResult.errors.length === 0 &&
    validate_image_size(file.size) &&
    validate_image_mime(file.type)
})

const onImageChange = (files: File[] | File | null | undefined): void => {
  const file = getImageFile(files)
  if (file) {
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

  // Validate with WASM validators (should already pass due to rules, but double-check)
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

  const imageFile = getImageFile(formData.image)
  if (!imageFile) {
    fieldErrors.image = [translate_error_code('IMAGE_REQUIRED', undefined)]
    return
  }

  loading.value = true

  try {
    const formDataObj = new FormData()
    formDataObj.append('title', formData.title.trim())
    if (formData.description.trim()) {
      formDataObj.append('description', formData.description.trim())
    }
    formDataObj.append('image', imageFile)

    const { data, error: apiError } = await createPost({
      body: formDataObj as unknown as { title: string; description?: string; image: Blob },
    })

    if (data) {
      router.push(`/posts/${data.postId}`)
    } else if (apiError) {
      handleCreatePostError(apiError as PostErrorResponse)
    } else {
      error.value = translate_error_code('INTERNAL', undefined)
    }
  } catch (e) {
    error.value = translate_error_code('INTERNAL', undefined)
    console.error('Failed to create post:', e)
  } finally {
    loading.value = false
  }
}

const handleCreatePostError = (apiError: PostErrorResponse): void => {
  console.error('Create post error:', apiError)

  // Handle validation errors with field-specific messages
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
  } else if (apiError.error === 'INVALID_IMAGE') {
    fieldErrors.image = [translate_error_code('INVALID_IMAGE', undefined)]
  } else {
    error.value = translate_error_code(apiError.error, undefined)
  }
}
</script>
