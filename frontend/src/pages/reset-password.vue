<template>
  <v-container class="fill-height" fluid>
    <v-row align="center" justify="center">
      <v-col cols="12" lg="4" md="6" sm="8">
        <v-card class="pa-6" elevation="4">
          <v-card-title class="text-h5 font-weight-bold text-center mb-4">
            Reset Your Password
          </v-card-title>

          <v-card-text>
            <div v-if="!completed">
              <p class="text-body-1 mb-6">
                Please enter your new password below.
              </p>

              <v-form ref="form" @submit.prevent="handleSubmit">
                <PasswordField
                  ref="passwordField"
                  v-model="password"
                  class="mb-4"
                  label="New Password"
                  :show-strength="true"
                />

                <PasswordField
                  ref="confirmPasswordField"
                  v-model="confirmPassword"
                  class="mb-4"
                  label="Confirm New Password"
                  :show-strength="false"
                  :validate="false"
                />

                <v-alert
                  v-if="statusMessage"
                  class="mb-4"
                  closable
                  :type="messageType"
                  @click:close="statusMessage = ''"
                >
                  {{ statusMessage }}
                </v-alert>

                <v-btn
                  block
                  class="mt-4"
                  color="primary"
                  :loading="loading"
                  size="large"
                  @click="handleSubmit"
                >
                  Reset Password
                </v-btn>
              </v-form>
            </div>

            <div v-else class="text-center">
              <v-icon class="mb-4" color="success" size="64">mdi-check-circle</v-icon>
              <p class="text-h6 mb-6">{{ statusMessage }}</p>
              <v-btn block color="primary" size="large" to="/login">
                Back to Login
              </v-btn>
            </div>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup lang="ts">
  import { StatusCodes } from 'http-status-codes'
  import { onMounted, ref } from 'vue'
  import { useRoute } from 'vue-router'
  import { completePasswordReset, type CompletePasswordResetErrorResponse } from '@/api/client'
  import PasswordField from '@/components/auth/PasswordField.vue'

  import { translate, translate_error_code } from '@/wasm/translator.js'

  const route = useRoute()
  const token = ref('')
  const password = ref('')
  const confirmPassword = ref('')
  const loading = ref(false)
  const completed = ref(false)
  const statusMessage = ref('')
  const messageType = ref<'success' | 'error'>('success')
  const passwordField = ref<any>(null)
  const confirmPasswordField = ref<any>(null)

  onMounted(() => {
    const queryToken = route.query.token as string
    if (queryToken) {
      token.value = queryToken
    } else {
      statusMessage.value = 'Invalid reset link. No token found.'
      messageType.value = 'error'
    }
  })

  async function handleSubmit () {
    if (!token.value) return

    // Validate fields
    const passwordValid = await passwordField.value?.validate()
    if (!passwordValid?.valid) return

    if (password.value !== confirmPassword.value) {
      if (confirmPasswordField.value) {
        confirmPasswordField.value.errors = ['Passwords do not match']
        confirmPasswordField.value.hasError = true
      }
      return
    }

    loading.value = true
    statusMessage.value = ''

    try {
      const { data, error, response } = await completePasswordReset({
        body: {
          token: token.value,
          newPassword: password.value,
        },
      })

      if (data) {
        statusMessage.value = translate('SUCCESS_PASSWORD_RESET_COMPLETED', undefined)
        messageType.value = 'success'
        completed.value = true
      } else if (response.status === StatusCodes.INTERNAL_SERVER_ERROR) {
        statusMessage.value = translate_error_code('INTERNAL', undefined)
        messageType.value = 'error'
      } else if (error) {
        const err = error as CompletePasswordResetErrorResponse
        statusMessage.value = translate_error_code(err.error, undefined)
        messageType.value = 'error'
      } else {
        statusMessage.value = translate_error_code('INTERNAL', undefined)
        messageType.value = 'error'
      }
    } catch (error: any) {
      console.error('Password reset failed', error)
      statusMessage.value = 'An error occurred during password reset'
      messageType.value = 'error'
    } finally {
      loading.value = false
    }
  }
</script>
