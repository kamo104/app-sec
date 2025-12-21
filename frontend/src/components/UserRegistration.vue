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

            <!-- Success Message -->
            <v-alert
              v-if="successMessage"
              type="success"
              variant="tonal"
              class="mb-4"
              density="compact"
            >
              {{ successMessage }}
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

            <!-- Form Data Preview -->
            <v-divider class="my-6"></v-divider>
            <div class="text-caption text-grey mb-2">Form Data Preview (for debugging):</div>
            <v-card variant="tonal" color="grey-lighten-4" class="pa-3">
              <pre class="text-caption">{{ previewData }}</pre>
            </v-card>
          </v-form>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup lang="ts">
import { ref, reactive, computed } from 'vue'

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
const successMessage = ref('')
const form = ref<any>(null)

// Track whether fields have been interacted with
const usernameTouched = ref(false)
const emailTouched = ref(false)
const passwordTouched = ref(false)
const confirmPasswordTouched = ref(false)

// Computed property for preview data (without password)
const previewData = computed(() => {
  return {
    username: formData.username,
    email: formData.email,
    password: formData.password ? '•'.repeat(formData.password.length) : '',
    confirmPassword: formData.confirmPassword ? '•'.repeat(formData.confirmPassword.length) : '',
  }
})

// Track errors for each field to control dynamic spacing
const usernameHasError = ref(false)
const emailHasError = ref(false)
const passwordErrors = ref<string[]>([])
const confirmPasswordHasError = ref(false)

// Combined password rules that updates passwordErrors array
const passwordRulesCombined: Array<(value: string) => string | boolean> = [
  (value: string): string | boolean => {
    if (!value) {
      // Only show error if field has been touched
      if (passwordTouched.value) {
        passwordErrors.value = ['Password is required']
        return 'Password is required'
      }
      // Return true (valid) if not touched yet, to prevent showing errors prematurely
      passwordErrors.value = []
      return true
    }

    const errors: string[] = []

    if (value.length < 8) errors.push('Password must be at least 8 characters')
    if (!/[A-Z]/.test(value)) errors.push('Password must contain at least one uppercase letter')
    if (!/[a-z]/.test(value)) errors.push('Password must contain at least one lowercase letter')
    if (!/[0-9]/.test(value)) errors.push('Password must contain at least one number')
    if (!/[^A-Za-z0-9]/.test(value)) errors.push('Password must contain at least one special character')

    passwordErrors.value = errors

    if (errors.length > 0) {
      // Return first error to satisfy Vuetify's rule system
      return errors[0]!
    }

    return true
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

// Clear success message when user starts typing
const clearSuccess = () => {
  successMessage.value = ''
}

// Handle input handlers to mark fields as touched
const handleUsernameInput = (value: string) => {
  if (!usernameTouched.value) {
    usernameTouched.value = true
  }
  clearSuccess()
}

const handleEmailInput = (value: string) => {
  if (!emailTouched.value) {
    emailTouched.value = true
  }
  clearSuccess()
}

const handlePasswordInput = (value: string) => {
  if (!passwordTouched.value) {
    passwordTouched.value = true
  }
  clearSuccess()
}

const handleConfirmPasswordInput = (value: string) => {
  if (!confirmPasswordTouched.value) {
    confirmPasswordTouched.value = true
  }
  clearSuccess()
}

// Main form submission handler
const handleSubmit = async () => {
  // Reset messages
  successMessage.value = ''

  // Validate using Vuetify form validation
  // This will automatically show errors under each field
  const { valid } = await form.value.validate()

  if (!valid) {
    return
  }

  // If we reach here, all fields are valid
  // Simulate API call
  loading.value = true

  setTimeout(() => {
    loading.value = false
    successMessage.value = 'Registration successful! Welcome aboard! 🎉'

    // Reset form after successful submission (optional)
    setTimeout(() => {
      formData.username = ''
      formData.email = ''
      formData.password = ''
      formData.confirmPassword = ''
      successMessage.value = ''
      // Also reset touched states
      usernameTouched.value = false
      emailTouched.value = false
      passwordTouched.value = false
      confirmPasswordTouched.value = false
      // Reset error states
      usernameHasError.value = false
      emailHasError.value = false
      passwordErrors.value = []
      confirmPasswordHasError.value = false
    }, 3000)
  }, 1500)
}
</script>

<style scoped>
pre {
  white-space: pre-wrap;
  word-wrap: break-word;
  font-family: 'Courier New', monospace;
}

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
</style>
