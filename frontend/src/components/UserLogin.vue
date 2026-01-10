<template>
  <AuthFormLayout title="Login" ref="authFormLayout">
    <template #default="{ handleSubmit: formSubmit }">
      <v-form @submit.prevent="handleSubmit" ref="form" validate-on="input lazy">
        <!-- Username Field (no length validation for login) -->
        <UsernameField
          ref="usernameField"
          v-model="formData.username"
          :validate-length="false"
          :hide-details="true"
          @touched="markFieldTouched('username')"
        />

        <!-- Password Field (no validation for login - backend handles it) -->
        <PasswordField
          ref="passwordField"
          v-model="formData.password"
          :show-strength="false"
          :validate="false"
          :hide-details="true"
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
import { loginUser, type ErrorResponse, type FieldType } from '@/api/client'
import { translate_error_code, translate_success_code, translate_field_validation_error } from '@/wasm/translator.js'
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

  // Check if fields are empty before validation
  if (!formData.username.trim() || !formData.password.trim()) {
    showMessage(translate_error_code('LOGIN_FIELDS_REQUIRED', undefined), 'error')
    return
  }

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
    const { data, error, response } = await loginUser({
      body: {
        username: formData.username,
        password: formData.password,
      }
    })

    if (data) {
      showMessage(translate_success_code('SUCCESS_LOGGED_IN', undefined), 'success')

      // Store user in auth store
      authStore.setUser(data)

      // Redirect to home
      console.log('Login successful:', data)
      router.push('/')
    } else if (error) {
      handleLoginError(error, response.status)
    }
  } catch (e) {
    console.error('Login error:', e)
    showMessage(translate_error_code('INTERNAL', undefined), 'error')
  } finally {
    loading.value = false
  }
}

const handleLoginError = (error: ErrorResponse, status: number) => {
  console.error('Login error:', error)

  // Handle specific error cases
  if (error.validation) {
    // ValidationErrorData has fieldErrors array
    for (const fieldError of error.validation.fieldErrors) {
      const translatedErrors = fieldError.errors.map(err => {
        return translate_field_validation_error(fieldError.field, err, 'en')
      })

      if (fieldError.field === 'USERNAME') {
        usernameField.value.errors = translatedErrors
        usernameField.value.hasError = true
      } else if (fieldError.field === 'PASSWORD') {
        passwordField.value.errors = translatedErrors
        passwordField.value.hasError = true
      }
    }
    showMessage(translate_error_code('VALIDATION', undefined), 'error')
  } else if (status === 401) {
    showMessage(translate_error_code(error.error, undefined), 'error')
  } else if (status === 0) {
    showMessage(translate_error_code('INTERNAL', undefined), 'error')
  } else {
    showMessage(translate_error_code(error.error, undefined), 'error')
  }
}
</script>

<style scoped>
/* No additional styles needed - handled by reusable components */
</style>
