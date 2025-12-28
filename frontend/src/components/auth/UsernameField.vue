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
      class="custom-field"
    ></v-text-field>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { validate_username_wasm } from '@/wasm/field-validator.js'
import { translate_validation_error } from '@/wasm/api-translator.js'

interface Props {
  modelValue: string
  required?: boolean
  minLength?: number
  maxLength?: number
  validateLength?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  required: true,
  minLength: 3,
  maxLength: 20,
  validateLength: true
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
      const resultJson = validate_username_wasm(value, props.minLength, props.maxLength, props.validateLength)
      const result = JSON.parse(resultJson)

      const translatedErrors = result.errors.map((err: any) => translate_validation_error(JSON.stringify(err), undefined))
      errors.value = translatedErrors
      hasError.value = !result.is_valid

      if (touched.value && translatedErrors.length > 0) {
        return translatedErrors[0]!
      }
      return true
    } catch (error) {
      console.error('Username validation error:', error)
      // If WASM fails, we cannot validate - return true to allow input
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

// Expose methods for parent component
const validate = async (): Promise<{ valid: boolean; errors: string[] }> => {
  if (!touched.value) {
    touched.value = true
  }

  try {
    const resultJson = validate_username_wasm(props.modelValue, props.minLength, props.maxLength, props.validateLength)
    const result = JSON.parse(resultJson)
    const translatedErrors = result.errors.map((err: any) => translate_validation_error(JSON.stringify(err), undefined))
    errors.value = translatedErrors
    hasError.value = !result.is_valid

    return { valid: result.is_valid, errors: translatedErrors }
  } catch (error) {
    console.error('Username validation error:', error)
    // If WASM fails, return empty errors (cannot validate)
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
