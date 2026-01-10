<template>
  <div :class="['auth-form-field-wrapper', { 'has-error': hasError && touched }]">
    <v-text-field
      :model-value="modelValue"
      @update:model-value="handleInput"
      :rules="rules"
      label="Username"
      prepend-inner-icon="mdi-account"
      variant="outlined"
      required
      validate-on="input"
      :error="hasError && touched"
      :hide-details="hideDetails"
      class="custom-field"
    ></v-text-field>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { validate_field } from '@/wasm/field-validator.js'
import { ValidationFieldError } from '@/generated/api'
import { translate_field_validation_error } from '@/wasm/translator.js'

interface Props {
  modelValue: string
  required?: boolean
  minLength?: number
  maxLength?: number
  validateLength?: boolean
  hideDetails?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  required: true,
  minLength: 3,
  maxLength: 20,
  validateLength: true,
  hideDetails: false
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
  touched: []
}>()

const touched = ref(false)
const hasError = ref(false)
const errors = ref<string[]>([])

const rules = [
  async (value: string): Promise<string | boolean> => {
    // Skip validation if validateLength is false (e.g., on login page)
    if (!props.validateLength) {
      return true
    }

    try {
      const resultBytes = validate_field('USERNAME', value)
      const result = ValidationFieldError.decode(resultBytes)

      const translatedErrors = result.errors.map((err: number) => {
        return translate_field_validation_error(result.field, err, undefined)
      })
      errors.value = translatedErrors
      hasError.value = result.errors.length > 0

      if (touched.value && translatedErrors.length > 0) {
        return translatedErrors[0]!
      }
      return true
    } catch (error) {
      console.error('Username validation error:', error)
      hasError.value = false
      errors.value = []
      return true
    }
  }
]

const handleInput = async (value: string) => {
  if (!touched.value) {
    touched.value = true
    emit('touched')
  }
  emit('update:modelValue', value)
}

const validate = async (): Promise<{ valid: boolean; errors: string[] }> => {
  if (!touched.value) {
    touched.value = true
  }

  // Skip validation if validateLength is false (e.g., on login page)
  if (!props.validateLength) {
    return { valid: true, errors: [] }
  }

  try {
    const resultBytes = validate_field('USERNAME', props.modelValue)
    const result = ValidationFieldError.decode(resultBytes)
    const translatedErrors = result.errors.map((err: number) => {
      return translate_field_validation_error(result.field, err, undefined)
    })
    errors.value = translatedErrors
    hasError.value = result.errors.length > 0

    return { valid: result.errors.length === 0, errors: translatedErrors }
  } catch (error) {
    console.error('Username validation error:', error)
    return { valid: true, errors: [] }
  }
}

defineExpose({
  validate,
  touched,
  hasError,
  errors,
})
</script>

<style scoped>
/* Common styles imported from src/styles/auth.css */
</style>
