<template>
  <v-container class="py-8 fill-height" fluid>
    <v-row align="center" class="fill-height" justify="center">
      <v-col cols="12" lg="5" md="6">
        <AuthFormLayout ref="authFormLayout" title="Login">
          <template #default="{ handleSubmit: formSubmit }">
            <v-form ref="form" validate-on="input lazy" @submit.prevent="handleSubmit">
              <!-- Username Field (no length validation for login) -->
              <UsernameField
                ref="usernameField"
                v-model="formData.username"
                :hide-details="true"
                :validate-length="false"
                @touched="markFieldTouched('username')"
              />

              <!-- Password Field (no validation for login - backend handles it) -->
              <PasswordField
                ref="passwordField"
                v-model="formData.password"
                :hide-details="true"
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
                :disabled="loading"
                label="Login"
                :loading="loading"
                @click="() => formSubmit(handleSubmit)"
              />

              <!-- Forgot Password Link -->
              <ForgotPasswordLink />
            </v-form>
          </template>

          <template #navigation>
            <v-btn
              class="text-none"
              color="primary"
              prepend-icon="mdi-account-plus"
              to="/register"
              variant="text"
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
  import { StatusCodes } from 'http-status-codes'
  import { reactive, ref } from 'vue'
  import { useRouter } from 'vue-router'
  import { type LoginErrorResponse, loginUser } from '@/api/client'
  import AuthFormLayout from '@/components/auth/AuthFormLayout.vue'
  import AuthSubmitButton from '@/components/auth/AuthSubmitButton.vue'

  import ForgotPasswordLink from '@/components/auth/ForgotPasswordLink.vue'
  import PasswordField from '@/components/auth/PasswordField.vue'
  import RememberMeCheckbox from '@/components/auth/RememberMeCheckbox.vue'
  import StatusMessage from '@/components/auth/StatusMessage.vue'
  import UsernameField from '@/components/auth/UsernameField.vue'
  import { useAuthStore } from '@/stores/auth'
  import { translate, translate_error_code, translate_field_validation_error } from '@/wasm/translator.js'

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

  const form = ref<any>(null)
  const usernameField = ref<InstanceType<typeof UsernameField> | null>(null)
  const passwordField = ref<InstanceType<typeof PasswordField> | null>(null)

  const touchedFields = reactive({
    username: false,
    password: false,
  })

  function markFieldTouched (field: keyof typeof touchedFields): void {
    touchedFields[field] = true
  }

  function clearMessage (): void {
    statusMessage.value = ''
  }

  function showMessage (message: string, type: 'success' | 'error' | 'warning' | 'info' = 'success'): void {
    statusMessage.value = message
    messageType.value = type
  }

  async function handleSubmit (): Promise<void> {
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
        },
      })

      if (data) {
        showMessage(translate('SUCCESS_LOGGED_IN', undefined), 'success')
        authStore.setUser(data)
        router.push('/')
      } else if (error) {
        handleLoginError(error, response.status)
      }
    } catch (error) {
      console.error('Login error:', error)
      showMessage(translate_error_code('INTERNAL', undefined), 'error')
    } finally {
      loading.value = false
    }
  }

  function handleLoginError (error: LoginErrorResponse, status: number): void {
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
