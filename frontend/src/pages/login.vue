<template>
  <v-container class="py-8 fill-height" fluid>
    <v-row justify="center" align="center" class="fill-height">
      <v-col cols="12" md="6" lg="5">
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
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { useRouter } from 'vue-router'
import { StatusCodes } from 'http-status-codes'
import { loginUser, type LoginErrorResponse } from '@/api/client'
import { translate_error_code, translate, translate_field_validation_error } from '@/wasm/translator.js'
import { useAuthStore } from '@/stores/auth'

import AuthFormLayout from '@/components/auth/AuthFormLayout.vue'
import UsernameField from '@/components/auth/UsernameField.vue'
import PasswordField from '@/components/auth/PasswordField.vue'
import RememberMeCheckbox from '@/components/auth/RememberMeCheckbox.vue'
import AuthSubmitButton from '@/components/auth/AuthSubmitButton.vue'
import StatusMessage from '@/components/auth/StatusMessage.vue'
import ForgotPasswordLink from '@/components/auth/ForgotPasswordLink.vue'

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

const form = ref<InstanceType<typeof import('vuetify/components').VForm> | null>(null)
const usernameField = ref<InstanceType<typeof UsernameField> | null>(null)
const passwordField = ref<InstanceType<typeof PasswordField> | null>(null)

const touchedFields = reactive({
  username: false,
  password: false,
})

const markFieldTouched = (field: keyof typeof touchedFields): void => {
  touchedFields[field] = true
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

  if (!formData.username.trim() || !formData.password.trim()) {
    showMessage(translate_error_code('LOGIN_FIELDS_REQUIRED', undefined), 'error')
    return
  }

  const validations = await Promise.all([
    usernameField.value?.validate(),
    passwordField.value?.validate(),
  ])

  const allValid = validations.every(v => v?.valid !== false)

  if (!allValid) {
    return
  }

  loading.value = true

  try {
    const { data, error, response } = await loginUser({
      body: {
        username: formData.username,
        password: formData.password,
      }
    })

    if (data) {
      showMessage(translate('SUCCESS_LOGGED_IN', undefined), 'success')
      authStore.setUser(data)
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

const handleLoginError = (error: LoginErrorResponse, status: number): void => {
  console.error('Login error:', error)

  if (error.validation) {
    for (const fieldError of error.validation.fieldErrors) {
      const translatedErrors = fieldError.errors.map(err => {
        return translate_field_validation_error(fieldError.field, err, 'en')
      })

      if (fieldError.field === 'USERNAME' && usernameField.value) {
        usernameField.value.errors = translatedErrors
        usernameField.value.hasError = true
      } else if (fieldError.field === 'PASSWORD' && passwordField.value) {
        passwordField.value.errors = translatedErrors
        passwordField.value.hasError = true
      }
    }
    showMessage(translate_error_code('VALIDATION', undefined), 'error')
  } else if (status === StatusCodes.UNAUTHORIZED) {
    showMessage(translate_error_code(error.error, undefined), 'error')
  } else if (status === 0) {
    showMessage(translate_error_code('INTERNAL', undefined), 'error')
  } else {
    showMessage(translate_error_code(error.error, undefined), 'error')
  }
}
</script>
