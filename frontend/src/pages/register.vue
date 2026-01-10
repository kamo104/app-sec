<template>
  <v-container class="py-8 fill-height" fluid>
    <v-row justify="center" align="center" class="fill-height">
      <v-col cols="12" md="6" lg="5">
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
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { StatusCodes } from 'http-status-codes'
import { registerUser, type RegisterErrorResponse } from '@/api/client'
import { translate, translate_error_code, translate_field_validation_error } from '@/wasm/translator.js'

import AuthFormLayout from '@/components/auth/AuthFormLayout.vue'
import UsernameField from '@/components/auth/UsernameField.vue'
import EmailField from '@/components/auth/EmailField.vue'
import PasswordField from '@/components/auth/PasswordField.vue'
import ConfirmPasswordField from '@/components/auth/ConfirmPasswordField.vue'
import AuthSubmitButton from '@/components/auth/AuthSubmitButton.vue'
import StatusMessage from '@/components/auth/StatusMessage.vue'

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

const form = ref<InstanceType<typeof import('vuetify/components').VForm> | null>(null)
const usernameField = ref<InstanceType<typeof UsernameField> | null>(null)
const emailField = ref<InstanceType<typeof EmailField> | null>(null)
const passwordField = ref<InstanceType<typeof PasswordField> | null>(null)
const confirmPasswordField = ref<InstanceType<typeof ConfirmPasswordField> | null>(null)

const touchedFields = reactive({
  username: false,
  email: false,
  password: false,
  confirmPassword: false,
})

const passwordValidation = reactive({
  isValid: false,
  errors: [] as string[],
})

const markFieldTouched = (field: keyof typeof touchedFields): void => {
  touchedFields[field] = true
}

const handlePasswordValidation = (isValid: boolean, errors: string[]): void => {
  passwordValidation.isValid = isValid
  passwordValidation.errors = errors
}

const clearMessage = (): void => {
  statusMessage.value = ''
}

const showMessage = (message: string, type: 'success' | 'error' | 'warning' | 'info' = 'success'): void => {
  statusMessage.value = message
  messageType.value = type
}

const handleSubmit = async (): Promise<void> => {
  clearMessage()

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
      const message = translate('SUCCESS_REGISTERED', undefined)
      showMessage(message, 'success')
    } else if (error) {
      handleRegistrationError(error, response.status)
    }
  } catch (e) {
    console.error('Registration error:', e)
    showMessage(translate_error_code('INTERNAL', undefined), 'error')
  } finally {
    loading.value = false
  }
}

const handleRegistrationError = (error: RegisterErrorResponse, status: number): void => {
  console.error('Registration error:', error)

  if (error.validation) {
    for (const fieldError of error.validation.fieldErrors) {
      const translatedErrors = fieldError.errors.map(err => {
        return translate_field_validation_error(fieldError.field, err, undefined)
      })

      if (fieldError.field === 'USERNAME' && usernameField.value) {
        usernameField.value.errors = translatedErrors
        usernameField.value.hasError = true
      } else if (fieldError.field === 'EMAIL' && emailField.value) {
        emailField.value.errors = translatedErrors
        emailField.value.hasError = true
      } else if (fieldError.field === 'PASSWORD' && passwordField.value) {
        passwordField.value.errors = translatedErrors
        passwordField.value.hasError = true
      }
    }
    showMessage(translate_error_code('VALIDATION', undefined), 'error')
  } else if (status === StatusCodes.CONFLICT) {
    showMessage(translate_error_code(error.error, undefined), 'error')
  } else if (status === 0) {
    showMessage(translate_error_code('INTERNAL', undefined), 'error')
  } else {
    showMessage(translate_error_code(error.error, undefined), 'error')
  }
}
</script>
