<template>
  <div :class="['auth-form-field-wrapper', { 'has-error': hasError && touched }]">
    <v-text-field
      :model-value="modelValue"
      @update:model-value="handleInput"
      :rules="rules"
      label="Password"
      prepend-inner-icon="mdi-lock"
      variant="outlined"
      required
      :type="showPassword ? 'text' : 'password'"
      :append-inner-icon="showPassword ? 'mdi-eye-off' : 'mdi-eye'"
      @click:append-inner="showPassword = !showPassword"
      validate-on="input"
      :error="hasError && touched"
      class="custom-field"
    >
      <template v-if="showStrength && errors.length > 0 && touched" #message>
        <div v-for="(error, index) in errors" :key="index" class="auth-password-error">
          • {{ error }}
        </div>
      </template>
    </v-text-field>

    <!-- Password Strength Indicator - optional -->
    <div v-if="showStrength && touched && score !== null" class="auth-password-strength-indicator">
      <div class="auth-strength-label">
        Score: <span :class="['auth-strength-value', getStrengthClass(score)]">{{ score }}</span> / 7
      </div>
      <div class="auth-strength-bar-container">
        <div
          class="auth-strength-bar"
          :class="getStrengthClass(score)"
          :style="{ width: (score / 7 * 100) + '%' }"
        ></div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import {
  validatePassword,
  getPasswordScore,
  getPasswordStrengthInfo
} from '@/services/fieldValidator'

interface Props {
  modelValue: string
  required?: boolean
  showStrength?: boolean
  validate?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  required: true,
  showStrength: false,
  validate: true,
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
  touched: []
  validation: [valid: boolean, errors: string[]]
}>()

const touched = ref(false)
const hasError = ref(false)
const showPassword = ref(false)
const errors = ref<string[]>([])
const score = ref<number | null>(null)

const handleInput = async (value: string) => {
  if (!touched.value) {
    touched.value = true
    emit('touched')
  }

  emit('update:modelValue', value)

  // Skip validation if disabled
  if (!props.validate) {
    return
  }

  // Always validate with WASM
  if (value) {
    await updateStrengthInfo(value)
  } else {
    errors.value = []
    score.value = null
    hasError.value = false
  }
}

const updateStrengthInfo = async (value: string) => {
  if (!value) {
    errors.value = []
    score.value = null
    return
  }

  try {
    const result = await getPasswordStrengthInfo(value)

    errors.value = result.errors
    score.value = result.score
    hasError.value = result.errors.length > 0

    emit('validation', result.isValid, result.errors)
  } catch (error) {
    console.error('Password validation error:', error)
    errors.value = []
    score.value = null
  }
}

const getStrengthClass = (scoreValue: number | null): string => {
  if (scoreValue === null) return ''
  if (scoreValue <= 3) return 'weak'
  if (scoreValue <= 5) return 'medium'
  return 'strong'
}

// Rules - use WASM validation only if validate prop is true
const rules = [
  async (value: string): Promise<string | boolean> => {
    if (!value) {
      if (touched.value) {
        hasError.value = true
        return 'Password is required'
      }
      hasError.value = false
      return true
    }

    // Skip validation rules if disabled
    if (!props.validate) {
      hasError.value = false
      return true
    }

    try {
      const result = await validatePassword(value)
      const passwordScore = await getPasswordScore(value)

      errors.value = result.errors
      score.value = passwordScore
      hasError.value = result.errors.length > 0

      emit('validation', result.isValid, result.errors)

      if (result.errors.length > 0) {
        return result.errors[0]!
      }
      return true
    } catch (error) {
      console.error('Password validation error:', error)
      // If WASM fails, we cannot validate - return true to allow input
      // The actual validation will happen on submit
      hasError.value = false
      return true
    }
  }
]

// Expose methods for parent component
const validate = async (): Promise<{ valid: boolean; errors: string[] }> => {
  if (!touched.value) {
    touched.value = true
  }

  if (!props.modelValue) {
    errors.value = ['Password is required']
    hasError.value = true
    return { valid: false, errors: ['Password is required'] }
  }

  // Skip validation if disabled
  if (!props.validate) {
    errors.value = []
    hasError.value = false
    return { valid: true, errors: [] }
  }

  try {
    const result = await getPasswordStrengthInfo(props.modelValue)

    errors.value = result.errors
    score.value = result.score
    hasError.value = result.errors.length > 0

    return { valid: result.isValid, errors: result.errors }
  } catch (error) {
    console.error('Password validation error:', error)
    // If WASM fails, return empty errors (cannot validate)
    return { valid: true, errors: [] }
  }
}

defineExpose({
  validate,
  touched,
  hasError,
  errors,
  score,
})
</script>

<style scoped>
/* Common styles imported from src/styles/auth.css */
</style>
