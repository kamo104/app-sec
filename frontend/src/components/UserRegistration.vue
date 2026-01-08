<template>
  <AuthFormLayout title="Register" ref="authFormLayout">
    <template #default="{ handleSubmit: formSubmit }">
      <v-form @submit.prevent="handleSubmit" ref="form" validate-on="input lazy">
        <!-- Username Field -->
        <UsernameField
          ref="usernameField"
          v-model="formData.username"
          @touched="markFieldTouched('username')"
        />

        <!-- Email Field -->
        <EmailField
          ref="emailField"
          v-model="formData.email"
          @touched="markFieldTouched('email')"
        />

        <!-- Password Field with Strength -->
        <PasswordField
          ref="passwordField"
          v-model="formData.password"
          show-strength
          @touched="markFieldTouched('password')"
          @validation="handlePasswordValidation"
        />

        <!-- Confirm Password Field -->
        <ConfirmPasswordField
          ref="confirmPasswordField"
          v-model="formData.confirmPassword"
          :password-to-match="formData.password"
          @touched="markFieldTouched('confirmPassword')"
        />

        <!-- Status Message -->
        <StatusMessage
          v-if="statusMessage"
          :message="statusMessage"
          :type="messageType"
          @close="clearMessage"
        />

        <!-- Submit Button -->
        <AuthSubmitButton
          label="Register"
          :loading="loading"
          :disabled="loading"
          @click="() => formSubmit(handleSubmit)"
        />
      </v-form>
    </template>

    <template #navigation>
      <v-btn
        variant="text"
        color="primary"
        to="/login"
        class="text-none"
        prepend-icon="mdi-login"
      >
        Already have an account? Login here
      </v-btn>
    </template>
  </AuthFormLayout>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { registerUser, type ApiError, ResponseCode } from '@/services/api'
import { translate_response_code, translate_validation_error } from '@/wasm/api-translator.js'
import { FieldType } from '@/generated/api'

// Import reusable components
import AuthFormLayout from './auth/AuthFormLayout.vue'
import UsernameField from './auth/UsernameField.vue'
import EmailField from './auth/EmailField.vue'
import PasswordField from './auth/PasswordField.vue'
import ConfirmPasswordField from './auth/ConfirmPasswordField.vue'
import AuthSubmitButton from './auth/AuthSubmitButton.vue'
import StatusMessage from './auth/StatusMessage.vue'

interface FormData {
  username: string
  email: string
  password: string
  confirmPassword: string
}

const formData = reactive<FormData>({
  username: '',
  email: '',
  password: '',
  confirmPassword: '',
})

const loading = ref(false)
const statusMessage = ref('')
const messageType = ref<'success' | 'error' | 'warning' | 'info'>('success')

// Refs to component instances
const form = ref<any>(null)
const usernameField = ref<any>(null)
const emailField = ref<any>(null)
const passwordField = ref<any>(null)
const confirmPasswordField = ref<any>(null)

// Track touched state for each field
const touchedFields = reactive({
  username: false,
  email: false,
  password: false,
  confirmPassword: false,
})

// Track password validation state
const passwordValidation = reactive({
  isValid: false,
  errors: [] as string[],
})

// Mark field as touched
const markFieldTouched = (field: keyof typeof touchedFields) => {
  touchedFields[field] = true
}

// Handle password validation from PasswordField component
const handlePasswordValidation = (isValid: boolean, errors: string[]) => {
  passwordValidation.isValid = isValid
  passwordValidation.errors = errors
}

// Clear status message
const clearMessage = () => {
  statusMessage.value = ''
}

// Display a message with the given type
const showMessage = (message: string, type: 'success' | 'error' | 'warning' | 'info' = 'success') => {
  statusMessage.value = message
  messageType.value = type
}

// Main form submission handler
const handleSubmit = async () => {
  // Reset messages
  clearMessage()

  // Validate all fields using component validation methods
  const validations = await Promise.all([
    usernameField.value?.validate(),
    emailField.value?.validate(),
    passwordField.value?.validate(),
    confirmPasswordField.value?.validate(),
  ])

  const allValid = validations.every(v => v?.valid !== false)

  if (!allValid) {
    return
  }

  // If we reach here, all fields are valid
  // Call the actual API
  loading.value = true

  try {
    const response = await registerUser({
      username: formData.username,
      email: formData.email,
      password: formData.password,
    })

    const message = translate_response_code(response.code, undefined)
    showMessage(message, 'success')
  } catch (error) {
    const apiError = error as ApiError
    console.error('Registration error:', apiError)

    // Handle specific error cases
    if (apiError.validationError) {
      const { field, errors } = apiError.validationError
      const translatedErrors = errors.map(err => {
        try {
          const errorBytes = new Uint8Array([err])
          return translate_validation_error(errorBytes, undefined)
        } catch {
          return String(err)
        }
      })

      if (field === FieldType.USERNAME) {
        usernameField.value.errors = translatedErrors
        usernameField.value.hasError = true
      } else if (field === FieldType.EMAIL) {
        emailField.value.errors = translatedErrors
        emailField.value.hasError = true
      } else if (field === FieldType.PASSWORD) {
        passwordField.value.errors = translatedErrors
        passwordField.value.hasError = true
      }
      showMessage('Please fix the validation errors.', 'error')
    } else if (apiError.status === 409) {
      // Username already taken - show error message
      showMessage('Username already exists. Please choose a different one.', 'error')
    } else if (apiError.status === 0) {
      // Network error - backend not running
      showMessage('Cannot connect to the backend server. Please ensure it is running on port 4000.', 'error')
    } else {
      // Other errors
      showMessage(apiError.message || 'An error occurred during registration', 'error')
    }
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
/* No additional styles needed - handled by reusable components */
</style>
