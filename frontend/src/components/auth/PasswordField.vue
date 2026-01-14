<template>
  <div :class="['auth-form-field-wrapper', { 'has-error': hasError && touched }]">
    <v-text-field
      :append-inner-icon="showPassword ? 'mdi-eye-off' : 'mdi-eye'"
      class="custom-field"
      :error="hasError && touched"
      :hide-details="hideDetails ? true : 'auto'"
      :label="translate('PASSWORD_LABEL', undefined)"
      :model-value="modelValue"
      prepend-inner-icon="mdi-lock"
      required
      :rules="rules"
      :type="showPassword ? 'text' : 'password'"
      validate-on="input"
      variant="outlined"
      @click:append-inner="showPassword = !showPassword"
      @update:model-value="handleInput"
    />

    <!-- Password validation errors (shown when showStrength is true) -->
    <div v-if="showStrength && errors.length > 0 && touched" class="auth-password-errors">
      <div v-for="(error, index) in errors" :key="index" class="auth-password-error">
        {{ error }}
      </div>
    </div>

    <div v-if="showStrength && touched && score !== null" class="auth-password-strength-indicator">
      <div class="auth-strength-label">
        Score: <span :class="['auth-strength-value', getStrengthClass(strength)]">{{ score }}</span> / 7
      </div>
      <div class="auth-strength-bar-container">
        <div
          class="auth-strength-bar"
          :class="getStrengthClass(strength)"
          :style="{ width: (score / 7 * 100) + '%' }"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
  import { ref } from 'vue'
  import { type PasswordStrength, validate_password_detailed, type ValidationDetailedPasswordData } from '@/wasm/field-validator.js'
  import { translate, translate_field_validation_error } from '@/wasm/translator.js'

  interface Props {
    modelValue: string
    required?: boolean
    showStrength?: boolean
    validate?: boolean
    hideDetails?: boolean
  }

  const props = withDefaults(defineProps<Props>(), {
    required: true,
    showStrength: true,
    validate: true,
    hideDetails: false,
  })

  const emit = defineEmits<{
    'update:modelValue': [value: string]
    'touched': []
    'validation': [isValid: boolean, errors: string[]]
  }>()

  const showPassword = ref(false)
  const touched = ref(false)
  const hasError = ref(false)
  const errors = ref<string[]>([])
  const score = ref<number | null>(null)
  const strength = ref<PasswordStrength>('PASSWORD_STRENGTH_UNSPECIFIED')

  const rules = [
    async (value: string): Promise<string | boolean> => {
      if (!props.validate) {
        return true
      }

      try {
        const resultJson = validate_password_detailed(value || '')
        const result: ValidationDetailedPasswordData = JSON.parse(resultJson)

        const translatedErrors = result.data?.errors.map((err: string) => {
          const fieldName = typeof result.data!.field === 'string' ? result.data!.field : 'PASSWORD'
          const errorCode = typeof err === 'string' ? err : 'VALIDATION_ERROR_CODE_UNSPECIFIED'
          return translate_field_validation_error(fieldName, errorCode, undefined)
        }) || []

        errors.value = translatedErrors
        score.value = result.score
        strength.value = result.strength
        hasError.value = translatedErrors.length > 0

        emit('validation', translatedErrors.length === 0, translatedErrors)

        // When showStrength is true, errors are displayed via #message slot and don't need to be returned
        if (props.showStrength) {
          return true
        }

        if (touched.value && translatedErrors.length > 0) {
          return translatedErrors[0]!
        }
        return true
      } catch (error) {
        console.error('Password validation error:', error)
        hasError.value = false
        errors.value = []
        score.value = null
        strength.value = 'PASSWORD_STRENGTH_UNSPECIFIED'
        return true
      }
    },
  ]

  function getStrengthClass (strengthValue: PasswordStrength): string {
    switch (strengthValue) {
      case 'PASSWORD_STRENGTH_WEAK': {
        return 'weak'
      }
      case 'PASSWORD_STRENGTH_MEDIUM': {
        return 'medium'
      }
      case 'PASSWORD_STRENGTH_STRONG': {
        return 'strong'
      }
      case 'PASSWORD_STRENGTH_CIA': {
        return 'cia'
      }
      default: {
        return 'weak'
      }
    }
  }

  async function handleInput (value: string) {
    if (!touched.value) {
      touched.value = true
      emit('touched')
    }
    emit('update:modelValue', value)
  }

  async function validate_field (): Promise<{ valid: boolean, errors: string[] }> {
    if (!touched.value) {
      touched.value = true
    }

    if (!props.validate) {
      return { valid: true, errors: [] }
    }

    try {
      const resultJson = validate_password_detailed(props.modelValue || '')
      const result: ValidationDetailedPasswordData = JSON.parse(resultJson)

      const translatedErrors = result.data?.errors.map((err: string) => {
        const fieldName = typeof result.data!.field === 'string' ? result.data!.field : 'PASSWORD'
        const errorCode = typeof err === 'string' ? err : 'VALIDATION_ERROR_CODE_UNSPECIFIED'
        return translate_field_validation_error(fieldName, errorCode, undefined)
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
    score,
  })
</script>

<style scoped>
/* Common styles imported from src/styles/auth.css */
</style>
