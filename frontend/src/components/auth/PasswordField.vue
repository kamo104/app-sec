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

    <div v-if="showStrength && touched && score !== null" class="auth-password-strength-indicator">
      <div class="auth-strength-label">
        Score: <span :class="['auth-strength-value', getStrengthClass(strength)]">{{ score }}</span> / 7
      </div>
      <div class="auth-strength-bar-container">
        <div
          class="auth-strength-bar"
          :class="getStrengthClass(strength)"
          :style="{ width: (score / 7 * 100) + '%' }"
        ></div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { validate_password_detailed } from '@/wasm/field-validator.js'
import { ValidationDetailedPasswordData, ValidationErrorData, PasswordStrength } from '@/generated/api'
import { translate_validation_error } from '@/wasm/api-translator.js'

interface Props {
  modelValue: string
  required?: boolean
  showStrength?: boolean
  validate?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  required: true,
  showStrength: true,
  validate: true
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
  touched: []
  validation: [isValid: boolean, errors: string[]]
}>()

const showPassword = ref(false)
const touched = ref(false)
const hasError = ref(false)
const errors = ref<string[]>([])
const score = ref<number | null>(null)
const strength = ref<PasswordStrength>(PasswordStrength.PASSWORD_STRENGTH_UNSPECIFIED)

const rules = [
  async (value: string): Promise<string | boolean> => {
    if (!props.validate) {
      return true
    }

    if (!value) {
      return true
    }

    try {
      const resultBytes = validate_password_detailed(value)
      const result = ValidationDetailedPasswordData.decode(resultBytes)

      const translatedErrors = result.data?.errors.map((err: number) => {
        const errorData = ValidationErrorData.encode({ field: result.data!.field, errors: [err] }).finish()
        return translate_validation_error(errorData, undefined)
      }) || []

      errors.value = translatedErrors
      score.value = result.score
      strength.value = result.strength
      hasError.value = translatedErrors.length > 0

      emit('validation', translatedErrors.length === 0, translatedErrors)

      if (touched.value && translatedErrors.length > 0) {
        return translatedErrors[0]!
      }
      return true
    } catch (error) {
      console.error('Password validation error:', error)
      hasError.value = false
      errors.value = []
      score.value = null
      strength.value = PasswordStrength.PASSWORD_STRENGTH_UNSPECIFIED
      return true
    }
  }
]

const getStrengthClass = (strengthValue: PasswordStrength): string => {
  switch (strengthValue) {
    case PasswordStrength.PASSWORD_STRENGTH_WEAK:
      return 'weak'
    case PasswordStrength.PASSWORD_STRENGTH_MEDIUM:
      return 'medium'
    case PasswordStrength.PASSWORD_STRENGTH_STRONG:
      return 'strong'
    case PasswordStrength.PASSWORD_STRENGTH_CIA:
      return 'cia'
    default:
      return 'weak'
  }
}

const handleInput = async (value: string) => {
  if (!touched.value) {
    touched.value = true
    emit('touched')
  }
  emit('update:modelValue', value)
}

const validate_field = async (): Promise<{ valid: boolean; errors: string[] }> => {
  if (!touched.value) {
    touched.value = true
  }

  if (!props.validate) {
    return { valid: true, errors: [] }
  }

  try {
    const resultBytes = validate_password_detailed(props.modelValue)
    const result = ValidationDetailedPasswordData.decode(resultBytes)

    const translatedErrors = result.data?.errors.map((err: number) => {
      const errorData = ValidationErrorData.encode({ field: result.data!.field, errors: [err] }).finish()
      return translate_validation_error(errorData, undefined)
    }) || []

    errors.value = translatedErrors
    score.value = result.score
    strength.value = result.strength
    hasError.value = translatedErrors.length > 0

    return { valid: translatedErrors.length === 0, errors: translatedErrors }
  } catch (error) {
    console.error('Password validation error:', error)
    return { valid: true, errors: [] }
  }
}

defineExpose({
  validate: validate_field,
  touched,
  hasError,
  errors,
  score
})
</script>

<style scoped>
/* Common styles imported from src/styles/auth.css */
</style>
