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
            <v-icon start>mdi-plus-circle</v-icon>
            Create New Post
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

              <!-- Image upload -->
              <v-file-input
                v-model="formData.image"
                :accept="IMAGE_ALLOWED_MIME_TYPES.join(',')"
                class="mb-4"
                :error-messages="fieldErrors.image"
                label="Image"
                prepend-icon="mdi-camera"
                :rules="imageRules"
                show-size
                variant="outlined"
                @update:model-value="onImageChange"
              />

              <!-- Image preview -->
              <v-card
                v-if="imagePreview"
                class="mb-4"
                variant="outlined"
              >
                <v-img
                  class="bg-grey-lighten-3"
                  contain
                  max-height="300"
                  :src="imagePreview"
                />
                <v-card-actions>
                  <v-btn
                    color="error"
                    prepend-icon="mdi-delete"
                    variant="text"
                    @click="clearImage"
                  >
                    Remove Image
                  </v-btn>
                </v-card-actions>
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

              <!-- Submit button -->
              <v-btn
                block
                color="primary"
                :disabled="!isValid"
                :loading="loading"
                size="large"
                type="submit"
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
  import { computed, reactive, ref } from 'vue'
  import { useRouter } from 'vue-router'
  import { createPost, type PostErrorResponse } from '@/api/client'
  import { VALIDATION_CONSTANTS } from '@/utils/validation.js'
  import {
    get_image_allowed_mime_types,
    get_post_description_max_length,
    get_post_title_max_length,
    validate_field,
    validate_image_mime,
    validate_image_size,
  } from '@/wasm/field-validator.js'
  import { translate_error_code, translate_field_validation_error } from '@/wasm/translator.js'

  const router = useRouter()

  // Constants from WASM field-validator (single source of truth)
  const POST_TITLE_MAX_LENGTH = VALIDATION_CONSTANTS.POST_TITLE_MAX_LENGTH
  const POST_DESCRIPTION_MAX_LENGTH = VALIDATION_CONSTANTS.POST_DESCRIPTION_MAX_LENGTH
  const IMAGE_ALLOWED_MIME_TYPES = VALIDATION_CONSTANTS.IMAGE_ALLOWED_MIME_TYPES

  const form = ref<HTMLFormElement | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)
  const imagePreview = ref<string | null>(null)

  interface FormData {
    title: string
    description: string
    image: File[] | File | null
  }

  const formData = reactive<FormData>({
    title: '',
    description: '',
    image: null,
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

  function hasValidImage (v: File[] | File | null): v is File[] | File {
    if (!v) return false
    if (Array.isArray(v)) return v.length > 0 && v[0] instanceof File
    return v instanceof File
  }

  function getImageFile (v: File[] | File | null): File | null {
    if (!v) return null
    if (Array.isArray(v)) {
      const file = v.find(f => f instanceof File)
      return file ?? null
    }
    return v instanceof File ? v : null
  }

  const imageRules = [
    (v: File[] | File | null) => hasValidImage(v) || translate_error_code('IMAGE_REQUIRED', undefined),
    (v: File[] | File | null) => {
      const file = getImageFile(v)
      return !file || validate_image_size(file.size) || translate_error_code('INVALID_IMAGE', undefined)
    },
    (v: File[] | File | null) => {
      const file = getImageFile(v)
      return !file || validate_image_mime(file.type) || translate_error_code('INVALID_IMAGE', undefined)
    },
  ]

  const isValid = computed(() => {
    const file = getImageFile(formData.image)
    const titleResult = validateFieldWasm('POST_TITLE', formData.title)
    const descResult = formData.description ? validateFieldWasm('POST_DESCRIPTION', formData.description) : { errors: [] }

    if (!file) return false

    return titleResult.errors.length === 0
      && descResult.errors.length === 0
      && validate_image_size(file.size)
      && validate_image_mime(file.type)
  })

  function onImageChange (files: File[] | File | null): void {
    const file = getImageFile(files)
    if (file) {
      const reader = new FileReader()
      reader.addEventListener('load', e => {
        imagePreview.value = e.target?.result as string
      })
      reader.readAsDataURL(file)
    } else {
      imagePreview.value = null
    }
  }

  function clearImage (): void {
    formData.image = null
    imagePreview.value = null
  }

  async function handleSubmit (): Promise<void> {
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
      const { data, error: apiError } = await createPost({
        body: {
          title: formData.title.trim(),
          description: formData.description.trim() || undefined,
          image: imageFile,
        },
      })

      if (data) {
        router.push(`/posts/${data.postId}`)
      } else if (apiError) {
        handleCreatePostError(apiError as PostErrorResponse)
      } else {
        error.value = translate_error_code('INTERNAL', undefined)
      }
    } catch (error_) {
      error.value = translate_error_code('INTERNAL', undefined)
      console.error('Failed to create post:', error_)
    } finally {
      loading.value = false
    }
  }

  function handleCreatePostError (apiError: PostErrorResponse): void {
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
