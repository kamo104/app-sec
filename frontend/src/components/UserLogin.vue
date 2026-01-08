<template>
  <AuthFormLayout title="Login" ref="authFormLayout">
    <template #default="{ handleSubmit: formSubmit }">
      <v-form @submit.prevent="handleSubmit" ref="form" validate-on="input lazy">
        <!-- Username Field (no length validation for login) -->
        <UsernameField
          ref="usernameField"
          v-model="formData.username"
          :validate-length="false"
          @touched="markFieldTouched('username')"
        />

        <!-- Password Field (no validation for login - backend handles it) -->
        <PasswordField
          ref="passwordField"
          v-model="formData.password"
          :show-strength="false"
          :validate="false"
          @touched="markFieldTouched('password')"
        />

        <!-- Remember Me Checkbox -->
        <RememberMeCheckbox
          v-model="formData.rememberMe"
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
          label="Login"
          :loading="loading"
          :disabled="loading"
          @click="() => formSubmit(handleSubmit)"
        />

        <!-- Forgot Password Link -->
        <ForgotPasswordLink />
      </v-form>
    </template>

    <template #navigation>
      <v-btn
        variant="text"
        color="primary"
        to="/register"
        class="text-none"
        prepend-icon="mdi-account-plus"
      >
        Don't have an account? Register here
      </v-btn>
    </template>
  </AuthFormLayout>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { useRouter } from 'vue-router'
import { loginUser, requestPasswordReset, type ApiError, ResponseCode } from '@/services/api'
import { translate_response_code, translate_validation_error } from '@/wasm/api-translator.js'
import { FieldType } from '@/generated/api'
import { useAuthStore } from '@/stores/auth'

// Import reusable components
import AuthFormLayout from './auth/AuthFormLayout.vue'
import UsernameField from './auth/UsernameField.vue'
import PasswordField from './auth/PasswordField.vue'
import RememberMeCheckbox from './auth/RememberMeCheckbox.vue'
import AuthSubmitButton from './auth/AuthSubmitButton.vue'
import StatusMessage from './auth/StatusMessage.vue'
import ForgotPasswordLink from './auth/ForgotPasswordLink.vue'

interface FormData {
  username: string
  password: string
  rememberMe: boolean
}

const formData = reactive<FormData>({
  username: '',
  password: '',
  rememberMe: false,
})

const router = useRouter()
const authStore = useAuthStore()
const loading = ref(false)
const statusMessage = ref('')
const messageType = ref<'success' | 'error' | 'warning' | 'info'>('success')

// Refs to component instances
const form = ref<any>(null)
const usernameField = ref<any>(null)
const passwordField = ref<any>(null)

// Track touched state for each field
const touchedFields = reactive({
  username: false,
  password: false,
})

// Mark field as touched
const markFieldTouched = (field: keyof typeof touchedFields) => {
  touchedFields[field] = true
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
    passwordField.value?.validate(),
  ])

  const allValid = validations.every(v => v?.valid !== false)

  if (!allValid) {
    return
  }

  // If we reach here, all fields are valid
  // Call the actual API
  loading.value = true

  try {
    const loginResponse = await loginUser({
      username: formData.username,
      password: formData.password,
    })

    showMessage('Login successful', 'success')

    // Store user in auth store
    authStore.setUser(loginResponse)

    // Redirect to dashboard
    console.log('Login successful:', loginResponse)
    router.push('/dashboard')
  } catch (error) {
    const apiError = error as ApiError
    console.error('Login error:', apiError)

    // Handle specific error cases
    if (apiError.validationError) {
      const { field, errors } = apiError.validationError
      const translatedErrors = errors.map(err => {
        try {
          const errorBytes = new Uint8Array([err])
          return translate_validation_error(errorBytes, 'en')
        } catch {
          return String(err)
        }
      })

      if (field === FieldType.USERNAME) {
        usernameField.value.errors = translatedErrors
        usernameField.value.hasError = true
      } else if (field === FieldType.PASSWORD) {
        passwordField.value.errors = translatedErrors
        passwordField.value.hasError = true
      }
      showMessage('Please fix the validation errors.', 'error')
    } else if (apiError.status === 401) {
      // Use the actual error message from the backend
      showMessage(apiError.message || 'Invalid username or password', 'error')
    } else if (apiError.status === 0) {
      // Network error - backend not running
      showMessage('Cannot connect to the backend server. Please ensure it is running on port 4000.', 'error')
    } else {
      // Other errors
      showMessage(apiError.message || 'An error occurred during login', 'error')
    }
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
/* No additional styles needed - handled by reusable components */
</style>
