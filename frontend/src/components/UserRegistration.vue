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
import { registerUser, type RegisterErrorResponse, type FieldType } from '@/api/client'
import { translate_success_code, translate_error_code, translate_field_validation_error } from '@/wasm/translator.js'

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
    const { data, error, response } = await registerUser({
      body: {
        username: formData.username,
        email: formData.email,
        password: formData.password,
      }
    })

    if (data) {
      const message = translate_success_code(data.success, undefined)
      showMessage(message, 'success')
    } else if (error) {
      handleRegistrationError(error, response.status)
    }
  } catch (e) {
    console.error('Registration error:', e)
    showMessage('An error occurred during registration', 'error')
  } finally {
    loading.value = false
  }
}

const handleRegistrationError = (error: RegisterErrorResponse, status: number) => {
  console.error('Registration error:', error)

  // Handle specific error cases
  if (error.validation) {
    // ValidationErrorData has fieldErrors array
    for (const fieldError of error.validation.fieldErrors) {
      const translatedErrors = fieldError.errors.map(err => {
        return translate_field_validation_error(fieldError.field, err, undefined)
      })

      if (fieldError.field === 'USERNAME') {
        usernameField.value.errors = translatedErrors
        usernameField.value.hasError = true
      } else if (fieldError.field === 'EMAIL') {
        emailField.value.errors = translatedErrors
        emailField.value.hasError = true
      } else if (fieldError.field === 'PASSWORD') {
        passwordField.value.errors = translatedErrors
        passwordField.value.hasError = true
      }
    }
    showMessage('Please fix the validation errors.', 'error')
  } else if (status === 409) {
    // Username or email already taken - translate the error code
    showMessage(translate_error_code(error.error, undefined), 'error')
  } else if (status === 0) {
    // Network error - backend not running
    showMessage('Cannot connect to the backend server. Please ensure it is running on port 4000.', 'error')
  } else {
    // Other errors
    showMessage(translate_error_code(error.error, undefined), 'error')
  }
}
</script>

<style scoped>
/* No additional styles needed - handled by reusable components */
</style>
