<template>
  <v-container class="fill-height" max-width="600">
    <v-row justify="center">
      <v-col cols="12" md="8">
        <v-card class="pa-6" elevation="2" title="User Login">
          <v-form @submit.prevent="handleSubmit" ref="form" validate-on="input lazy">
            <!-- Username Field -->
            <div :class="['form-field-wrapper', { 'has-error': usernameHasError && usernameTouched }]">
              <v-text-field
                v-model="formData.username"
                :rules="usernameRules"
                label="Username"
                prepend-inner-icon="mdi-account"
                variant="outlined"
                required
                validate-on="input"
                @update:model-value="handleUsernameInput"
                :error="usernameHasError && usernameTouched"
                class="custom-field"
              ></v-text-field>
            </div>

            <!-- Password Field -->
            <div :class="['form-field-wrapper', { 'has-error': passwordHasError && passwordTouched }]">
              <v-text-field
                v-model="formData.password"
                :rules="passwordRules"
                label="Password"
                prepend-inner-icon="mdi-lock"
                variant="outlined"
                required
                :type="showPassword ? 'text' : 'password'"
                :append-inner-icon="showPassword ? 'mdi-eye-off' : 'mdi-eye'"
                @click:append-inner="showPassword = !showPassword"
                validate-on="input"
                @update:model-value="handlePasswordInput"
                :error="passwordHasError && passwordTouched"
                class="custom-field"
              ></v-text-field>
            </div>

            <!-- Remember Me Checkbox -->
            <div class="mb-4">
              <v-checkbox
                v-model="formData.rememberMe"
                label="Remember me"
                density="compact"
                hide-details
              ></v-checkbox>
            </div>

            <!-- Status Message -->
            <v-alert
              v-if="statusMessage"
              :type="messageType"
              variant="tonal"
              class="mb-4"
              density="compact"
              closable
              @click:close="clearMessage"
            >
              {{ statusMessage }}
            </v-alert>

            <!-- Login Button -->
            <v-btn
              type="submit"
              color="primary"
              size="large"
              block
              :loading="loading"
              :disabled="loading"
            >
              Login
            </v-btn>

            <!-- Reset Password Link -->
            <div class="text-center mt-4">
              <v-btn
                variant="text"
                color="info"
                size="small"
                @click="handleResetPassword"
              >
                Forgot your password?
              </v-btn>
            </div>
          </v-form>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { loginUser, type ApiError } from '@/services/api'

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

const showPassword = ref(false)
const loading = ref(false)
const statusMessage = ref('')
const messageType = ref<'success' | 'error' | 'warning' | 'info'>('success')
const form = ref<any>(null)

// Track whether fields have been interacted with
const usernameTouched = ref(false)
const passwordTouched = ref(false)

// Track errors for each field
const usernameHasError = ref(false)
const passwordHasError = ref(false)

// Validation rules with error tracking
const usernameRules: Array<(value: string) => string | boolean> = [
  (value: string): string | boolean => {
    if (!value) {
      // Only show error if field has been touched
      if (usernameTouched.value) {
        usernameHasError.value = true
        return 'Username is required'
      }
      usernameHasError.value = false
      return true
    }
    usernameHasError.value = false
    return true
  }
]

const passwordRules: Array<(value: string) => string | boolean> = [
  (value: string): string | boolean => {
    if (!value) {
      // Only show error if field has been touched
      if (passwordTouched.value) {
        passwordHasError.value = true
        return 'Password is required'
      }
      passwordHasError.value = false
      return true
    }
    passwordHasError.value = false
    return true
  }
]

// Clear status message
const clearMessage = () => {
  statusMessage.value = ''
}

// Display a message with the given type
const showMessage = (message: string, type: 'success' | 'error' | 'warning' | 'info' = 'success') => {
  statusMessage.value = message
  messageType.value = type
}

// Handle input handlers to mark fields as touched
const handleUsernameInput = (value: string) => {
  if (!usernameTouched.value) {
    usernameTouched.value = true
  }
  clearMessage()
}

const handlePasswordInput = (value: string) => {
  if (!passwordTouched.value) {
    passwordTouched.value = true
  }
  clearMessage()
}

// Handle reset password
const handleResetPassword = () => {
  showMessage('Password reset functionality will be implemented soon!', 'info')
}

// Main form submission handler
const handleSubmit = async () => {
  // Reset messages
  clearMessage()

  // Validate using Vuetify form validation
  const { valid } = await form.value.validate()

  if (!valid) {
    return
  }

  // If we reach here, all fields are valid
  loading.value = true

  try {
    const response = await loginUser({
      username: formData.username,
      password: formData.password,
    })

    if (response.success && response.data) {
      showMessage(`Welcome back, ${response.data.username}!`, 'success')

      // Store user info if "Remember me" is checked
      if (formData.rememberMe) {
        localStorage.setItem('user', JSON.stringify(response.data))
      }

      // TODO: Redirect to dashboard or home page
      // For now, just show success message
      console.log('Login successful:', response.data)
    } else {
      // Handle non-success response
      console.error('Login failed:', response.message)
      showMessage(response.message || 'Login failed', 'error')
    }
  } catch (error) {
    const apiError = error as ApiError
    console.error('Login error:', apiError)

    // Handle specific error cases
    if (apiError.status === 401) {
      // Unauthorized - invalid credentials
      showMessage('Invalid username or password', 'error')
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
/* Remove all default spacing between fields */
.form-field-wrapper {
  margin-bottom: 0;
}

/* Add spacing between form fields only when field has been touched AND has an error */
.form-field-wrapper.has-error {
  margin-bottom: 12px;
}

/* Remove extra spacing after the last field */
.form-field-wrapper:last-child.has-error {
  margin-bottom: 0;
}

/* Remove Vuetify's default margin on text fields */
:deep(.custom-field) {
  margin-bottom: 0;
}
</style>
