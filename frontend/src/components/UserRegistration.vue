<template>
  <v-container class="fill-height" max-width="600">
    <v-row justify="center">
      <v-col cols="12" md="8">
        <v-card class="pa-6" elevation="2" title="User Registration">
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

            <!-- Email Field -->
            <div :class="['form-field-wrapper', { 'has-error': emailHasError && emailTouched }]">
              <v-text-field
                v-model="formData.email"
                :rules="emailRules"
                label="Email"
                prepend-inner-icon="mdi-email"
                variant="outlined"
                required
                type="email"
                validate-on="input"
                @update:model-value="handleEmailInput"
                :error="emailHasError && emailTouched"
                class="custom-field"
              ></v-text-field>
            </div>

            <!-- Password Field -->
            <div :class="['form-field-wrapper', { 'has-error': passwordErrors.length > 0 && passwordTouched }]">
              <v-text-field
                v-model="formData.password"
                :rules="passwordRulesCombined"
                label="Password"
                prepend-inner-icon="mdi-lock"
                variant="outlined"
                required
                :type="showPassword ? 'text' : 'password'"
                :append-inner-icon="showPassword ? 'mdi-eye-off' : 'mdi-eye'"
                @click:append-inner="showPassword = !showPassword"
                validate-on="input"
                @update:model-value="handlePasswordInput"
                :error="passwordErrors.length > 0 && passwordTouched"
                class="custom-field"
              >
                <template v-if="passwordErrors.length > 0 && passwordTouched" #message>
                  <div v-for="(error, index) in passwordErrors" :key="index" class="password-error">
                    • {{ error }}
                  </div>
                </template>
              </v-text-field>

              <!-- Password Strength Indicator - always visible when touched and score exists -->
              <div v-if="passwordTouched && passwordScore !== null" class="password-strength-indicator">
                <div class="strength-label">
                  Score: <span :class="['strength-value', getStrengthClass(passwordScore)]">{{ passwordScore }}</span> / 7
                </div>
                <div class="strength-bar-container">
                  <div
                    class="strength-bar"
                    :class="getStrengthClass(passwordScore)"
                    :style="{ width: (passwordScore / 7 * 100) + '%' }"
                  ></div>
                </div>
              </div>
            </div>

            <!-- Confirm Password Field -->
            <div :class="['form-field-wrapper', { 'has-error': confirmPasswordHasError && confirmPasswordTouched }]">
              <v-text-field
                v-model="formData.confirmPassword"
                :rules="confirmPasswordRules"
                label="Confirm Password"
                prepend-inner-icon="mdi-lock-check"
                variant="outlined"
                required
                :type="showConfirmPassword ? 'text' : 'password'"
                :append-inner-icon="showConfirmPassword ? 'mdi-eye-off' : 'mdi-eye'"
                @click:append-inner="showConfirmPassword = !showConfirmPassword"
                validate-on="input"
                @update:model-value="handleConfirmPasswordInput"
                :error="confirmPasswordHasError && confirmPasswordTouched"
                class="custom-field"
              ></v-text-field>
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

            <!-- Submit Button -->
            <v-btn
              type="submit"
              color="primary"
              size="large"
              block
              :loading="loading"
              :disabled="loading"
            >
              Register
            </v-btn>
          </v-form>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { registerUser, type ApiError } from '@/services/api'
import { initializePasswordValidator, validatePassword, getPasswordScore } from '@/services/passwordValidator'

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

const showPassword = ref(false)
const showConfirmPassword = ref(false)
const loading = ref(false)
const statusMessage = ref('')
const messageType = ref<'success' | 'error' | 'warning' | 'info'>('success')
const form = ref<any>(null)

// Track whether fields have been interacted with
const usernameTouched = ref(false)
const emailTouched = ref(false)
const passwordTouched = ref(false)
const confirmPasswordTouched = ref(false)

// Track errors for each field to control dynamic spacing
const usernameHasError = ref(false)
const emailHasError = ref(false)
const passwordErrors = ref<string[]>([])
const confirmPasswordHasError = ref(false)
const wasmInitialized = ref(false)
const passwordScore = ref<number | null>(null)

// Initialize WebAssembly on component mount
onMounted(async () => {
  try {
    await initializePasswordValidator()
    wasmInitialized.value = true
  } catch (error) {
    console.error('Failed to initialize password validator:', error)
  }
})

// Combined password rules that uses WebAssembly validation
const passwordRulesCombined: Array<(value: string) => Promise<string | boolean>> = [
  async (value: string): Promise<string | boolean> => {
    if (!value) {
      // Only show error if field has been touched
      if (passwordTouched.value) {
        passwordErrors.value = ['Password is required']
        passwordScore.value = null
        return 'Password is required'
      }
      // Return true (valid) if not touched yet, to prevent showing errors prematurely
      passwordErrors.value = []
      passwordScore.value = null
      return true
    }

    // Use WebAssembly validation
    try {
      const [result, score] = await Promise.all([
        validatePassword(value),
        getPasswordScore(value)
      ])

      passwordErrors.value = result.errors
      passwordScore.value = score

      if (result.errors.length > 0) {
        return result.errors[0]!
      }

      return true
    } catch (error) {
      console.error('Password validation error:', error)
      // If WASM validation fails, the app depends on WASM so we don't have a fallback
      // Just return true to allow submission and let backend handle validation
      return true
    }
  }
]

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
    if (value.length < 3) {
      usernameHasError.value = true
      return 'Username must be at least 3 characters'
    }
    if (value.length > 20) {
      usernameHasError.value = true
      return 'Username must be less than 20 characters'
    }
    if (!/^[a-zA-Z0-9_]+$/.test(value)) {
      usernameHasError.value = true
      return 'Username can only contain letters, numbers, and underscores'
    }
    usernameHasError.value = false
    return true
  }
]

const emailRules: Array<(value: string) => string | boolean> = [
  (value: string): string | boolean => {
    if (!value) {
      // Only show error if field has been touched
      if (emailTouched.value) {
        emailHasError.value = true
        return 'Email is required'
      }
      emailHasError.value = false
      return true
    }
    if (!/.+@.+\..+/.test(value)) {
      emailHasError.value = true
      return 'Email must be valid'
    }
    emailHasError.value = false
    return true
  }
]

const confirmPasswordRules: Array<(value: string) => string | boolean> = [
  (value: string): string | boolean => {
    if (!value) {
      // Only show error if field has been touched
      if (confirmPasswordTouched.value) {
        confirmPasswordHasError.value = true
        return 'Please confirm your password'
      }
      confirmPasswordHasError.value = false
      return true
    }
    if (value !== formData.password) {
      confirmPasswordHasError.value = true
      return 'Passwords do not match'
    }
    confirmPasswordHasError.value = false
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

const handleEmailInput = (value: string) => {
  if (!emailTouched.value) {
    emailTouched.value = true
  }
  clearMessage()
}

const handlePasswordInput = async (value: string) => {
  if (!passwordTouched.value) {
    passwordTouched.value = true
  }
  clearMessage()

  // Update score immediately on input
  if (value && wasmInitialized.value) {
    try {
      const score = await getPasswordScore(value)
      passwordScore.value = score
    } catch (error) {
      console.error('Error getting password score:', error)
    }
  } else if (!value) {
    passwordScore.value = null
    passwordErrors.value = []
  }
}

const handleConfirmPasswordInput = (value: string) => {
  if (!confirmPasswordTouched.value) {
    confirmPasswordTouched.value = true
  }
  clearMessage()
}

// Get CSS class for strength based on score
const getStrengthClass = (score: number | null): string => {
  if (score === null) return ''
  if (score <= 3) return 'weak'
  if (score <= 5) return 'medium'
  return 'strong'
}

// Main form submission handler
const handleSubmit = async () => {
  // Reset messages
  clearMessage()

  // Validate using Vuetify form validation
  // This will automatically show errors under each field
  const { valid } = await form.value.validate()

  if (!valid) {
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

    if (response.success) {
      showMessage(response.message || 'Registration successful! Welcome aboard!', 'success')
    } else {
      // Handle non-success response
      console.error('Registration failed:', response.message)
      showMessage(response.message || 'Registration failed', 'error')
    }
  } catch (error) {
    const apiError = error as ApiError
    console.error('Registration error:', apiError)

    // Handle specific error cases
    if (apiError.status === 409) {
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
/* Add consistent spacing between form fields */
.form-field-wrapper {
  margin-bottom: 16px;
}

/* Remove extra spacing after the last field */
.form-field-wrapper:last-child {
  margin-bottom: 0;
}

/* Add more spacing when there are errors to accommodate the error messages */
.form-field-wrapper.has-error {
  margin-bottom: 24px;
}

/* Remove Vuetify's default margin on text fields */
:deep(.custom-field) {
  margin-bottom: 0;
}

/* Custom styling for password error messages */
.password-error {
  line-height: 1.3;
  margin-bottom: 3px;
  color: rgb(var(--v-theme-error));
  font-size: 0.75rem;
  text-indent: -0.8em;
  padding-left: 0.8em;
}

/* Ensure the message slot displays correctly */
:deep(.v-messages__wrapper) {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

/* Remove bottom margin from last error to prevent extra spacing */
.password-error:last-child {
  margin-bottom: 0;
}

/* Password Strength Indicator Styles - positioned after the input field */
.password-strength-indicator {
  margin-top: 2px;
  padding: 6px 10px;
  background: rgba(0, 0, 0, 0.02);
  border-radius: 4px;
  border-left: 3px solid transparent;
}

/* Hide Vuetify's message area when there are no errors */
.form-field-wrapper:not(.has-error) :deep(.v-messages) {
  min-height: 0 !important;
  height: 0 !important;
  max-height: 0 !important;
  overflow: hidden !important;
  margin: 0 !important;
  padding: 0 !important;
}

/* Ensure the message wrapper itself has no height when empty */
.form-field-wrapper:not(.has-error) :deep(.v-messages__wrapper) {
  min-height: 0 !important;
  height: 0 !important;
  max-height: 0 !important;
}

/* Remove padding from input details area when no errors */
.form-field-wrapper:not(.has-error) :deep(.v-input__details) {
  padding: 0 !important;
  min-height: 0 !important;
}

.password-strength-indicator.weak {
  border-left-color: rgb(var(--v-theme-error));
}

.password-strength-indicator.medium {
  border-left-color: rgb(var(--v-theme-warning));
}

.password-strength-indicator.strong {
  border-left-color: rgb(var(--v-theme-success));
}

.strength-label {
  font-size: 0.875rem;
  font-weight: 500;
  margin-bottom: 2px;
  display: flex;
  align-items: center;
  gap: 4px;
}

.strength-value {
  font-weight: 700;
}

.strength-value.weak {
  color: rgb(var(--v-theme-error));
}

.strength-value.medium {
  color: rgb(var(--v-theme-warning));
}

.strength-value.strong {
  color: rgb(var(--v-theme-success));
}

.strength-bar-container {
  height: 4px;
  background: rgba(0, 0, 0, 0.1);
  border-radius: 2px;
  overflow: hidden;
  width: 100%;
  margin-top: 4px;
}

.strength-bar {
  height: 100%;
  transition: width 0.3s ease, background-color 0.3s ease;
  border-radius: 2px;
  min-width: 2px;
}

.strength-bar.weak {
  background: rgb(var(--v-theme-error));
}

.strength-bar.medium {
  background: rgb(var(--v-theme-warning));
}

.strength-bar.strong {
  background: rgb(var(--v-theme-success));
}
</style>
