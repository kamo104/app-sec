<template>
  <div :class="['auth-form-field-wrapper', { 'has-error': hasError && touched }]">
    <v-text-field
      :model-value="modelValue"
      @update:model-value="handleInput"
      :rules="rules"
      label="Email"
      prepend-inner-icon="mdi-email"
      variant="outlined"
      required
      type="email"
      validate-on="input"
      :error="hasError && touched"
      class="custom-field"
    ></v-text-field>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { validate_field } from '@/wasm/field-validator.js'
import type { ValidationFieldError } from '@/generated/api-client'
import { translate_field_validation_error } from '@/wasm/translator.js'

interface Props {
  modelValue: string
  required?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  required: true,
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
    try {
      const resultJson = validate_field('EMAIL', value)
      const result: ValidationFieldError = JSON.parse(resultJson)

      const translatedErrors = result.errors.map((err) => {
        const fieldName = typeof result.field === 'string' ? result.field : 'EMAIL'
        const errorCode = typeof err === 'string' ? err : 'VALIDATION_ERROR_CODE_UNSPECIFIED'
        return translate_field_validation_error(fieldName, errorCode, undefined)
      })
      errors.value = translatedErrors
      hasError.value = result.errors.length > 0

      if (touched.value && translatedErrors.length > 0) {
        return translatedErrors[0]!
      }
      return true
    } catch (error) {
      console.error('Email validation error:', error)
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

  try {
    const resultJson = validate_field('EMAIL', props.modelValue)
    const result: ValidationFieldError = JSON.parse(resultJson)
    const translatedErrors = result.errors.map((err) => {
      const fieldName = typeof result.field === 'string' ? result.field : 'EMAIL'
      const errorCode = typeof err === 'string' ? err : 'VALIDATION_ERROR_CODE_UNSPECIFIED'
      return translate_field_validation_error(fieldName, errorCode, undefined)
    })
    errors.value = translatedErrors
    hasError.value = result.errors.length > 0

    return { valid: result.errors.length === 0, errors: translatedErrors }
  } catch (error) {
    console.error('Email validation error:', error)
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
