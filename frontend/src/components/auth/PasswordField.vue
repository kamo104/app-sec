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
  validate_password_wasm,
  get_password_strength,
  get_password_strength_class
} from '@/wasm/field-validator.js'
import { translate_validation_error } from '@/wasm/api-translator.js'
import { PasswordStrength } from '@/generated/api'

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
  try {
    const strengthJson = get_password_strength(value)
    const result = JSON.parse(strengthJson)

    const translatedErrors = result.errors.map((err: any) => translate_validation_error(JSON.stringify(err), undefined))
    errors.value = translatedErrors
    score.value = result.score
    hasError.value = translatedErrors.length > 0

    emit('validation', result.is_valid, translatedErrors)
  } catch (error) {
    console.error('Password validation error:', error)
    errors.value = []
    score.value = null
  }
}

const getStrengthClass = (scoreValue: number | null): string => {
  if (scoreValue === null) return ''
  const strength = get_password_strength_class(scoreValue) as PasswordStrength
  switch (strength) {
    case PasswordStrength.PASSWORD_STRENGTH_WEAK:
      return 'weak'
    case PasswordStrength.PASSWORD_STRENGTH_MEDIUM:
      return 'medium'
    case PasswordStrength.PASSWORD_STRENGTH_STRONG:
      return 'strong'
    default:
      return ''
  }
}

// Rules - use WASM validation only if validate prop is true
const rules = [
  async (value: string): Promise<string | boolean> => {
    // Skip validation rules if disabled
    if (!props.validate) {
      hasError.value = false
      return true
    }

    try {
      const strengthJson = get_password_strength(value)
      const strengthResult = JSON.parse(strengthJson)
      const translatedErrors = strengthResult.errors.map((err: any) => translate_validation_error(JSON.stringify(err), undefined))

      errors.value = translatedErrors
      score.value = strengthResult.score
      hasError.value = translatedErrors.length > 0

      emit('validation', strengthResult.is_valid, translatedErrors)

      if (touched.value && translatedErrors.length > 0) {
        return translatedErrors[0]!
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

  // Skip validation if disabled
  if (!props.validate) {
    errors.value = []
    hasError.value = false
    return { valid: true, errors: [] }
  }

  try {
    const strengthJson = get_password_strength(props.modelValue)
    const result = JSON.parse(strengthJson)
    const translatedErrors = result.errors.map((err: any) => translate_validation_error(JSON.stringify(err), undefined))

    errors.value = translatedErrors
    score.value = result.score
    hasError.value = translatedErrors.length > 0

    return { valid: result.is_valid, errors: translatedErrors }
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
