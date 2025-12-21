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
import { validateEmail } from '@/services/fieldValidator'

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
    if (!value) {
      // Only show error if field has been touched
      if (touched.value) {
        hasError.value = true
        errors.value = ['Email is required']
        return 'Email is required'
      }
      hasError.value = false
      errors.value = []
      return true
    }

    try {
      const result = await validateEmail(value)
      errors.value = result.errors
      hasError.value = !result.isValid

      if (result.errors.length > 0) {
        return result.errors[0]!
      }
      return true
    } catch (error) {
      console.error('Email validation error:', error)
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

  if (!props.modelValue) {
    errors.value = ['Email is required']
    hasError.value = true
    return { valid: false, errors: ['Email is required'] }
  }

  try {
    const result = await validateEmail(props.modelValue)
    errors.value = result.errors
    hasError.value = !result.isValid

    return { valid: result.isValid, errors: result.errors }
  } catch (error) {
    console.error('Email validation error:', error)
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
