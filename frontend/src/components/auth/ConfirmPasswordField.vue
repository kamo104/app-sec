<template>
  <div :class="['auth-form-field-wrapper', { 'has-error': hasError && touched }]">
    <v-text-field
      :model-value="modelValue"
      @update:model-value="handleInput"
      :rules="rules"
      :label="translate('PASSWORD_CONFIRM_LABEL', undefined)"
      prepend-inner-icon="mdi-lock-check"
      variant="outlined"
      required
      :type="showPassword ? 'text' : 'password'"
      :append-inner-icon="showPassword ? 'mdi-eye-off' : 'mdi-eye'"
      @click:append-inner="showPassword = !showPassword"
      validate-on="input"
      :error="hasError && touched"
      class="custom-field"
    ></v-text-field>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { translate } from '@/wasm/translator.js'

// Constants for validation messages
const CONFIRM_PASSWORD_REQUIRED = translate('CONFIRM_PASSWORD_REQUIRED', undefined)
const PASSWORDS_DO_NOT_MATCH = translate('PASSWORDS_DO_NOT_MATCH', undefined)

interface Props {
  modelValue: string
  passwordToMatch: string
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
const showPassword = ref(false)

const rules = [
  (value: string): string | boolean => {
    if (!value) {
      // Only show error if field has been touched
      if (touched.value) {
        hasError.value = true
        return CONFIRM_PASSWORD_REQUIRED
      }
      hasError.value = false
      return true
    }
    if (value !== props.passwordToMatch) {
      hasError.value = true
      return PASSWORDS_DO_NOT_MATCH
    }
    hasError.value = false
    return true
  }
]

const handleInput = (value: string) => {
  if (!touched.value) {
    touched.value = true
    emit('touched')
  }
  emit('update:modelValue', value)
}

// Expose methods for parent component
const validate = (): { valid: boolean; errors: string[] } => {
  if (!touched.value) {
    touched.value = true
  }

  const errors: string[] = []

  if (!props.modelValue) {
    errors.push(CONFIRM_PASSWORD_REQUIRED)
    hasError.value = true
  } else if (props.modelValue !== props.passwordToMatch) {
    errors.push(PASSWORDS_DO_NOT_MATCH)
    hasError.value = true
  } else {
    hasError.value = false
  }

  return { valid: errors.length === 0, errors }
}

defineExpose({
  validate,
  touched,
  hasError,
})
</script>

<style scoped>
/* Common styles imported from src/styles/auth.css */
</style>