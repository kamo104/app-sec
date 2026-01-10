<template>
  <v-container class="fill-height" fluid>
    <v-row align="center" justify="center">
      <v-col cols="12" sm="8" md="6" lg="4">
        <v-card class="pa-6" elevation="4">
          <v-card-title class="text-h5 font-weight-bold text-center mb-4">
            Reset Your Password
          </v-card-title>

          <v-card-text>
            <div v-if="!completed">
              <p class="text-body-1 mb-6">
                Please enter your new password below.
              </p>

              <v-form @submit.prevent="handleSubmit" ref="form">
                <PasswordField
                  ref="passwordField"
                  v-model="password"
                  label="New Password"
                  :show-strength="true"
                  class="mb-4"
                />

                <PasswordField
                  ref="confirmPasswordField"
                  v-model="confirmPassword"
                  label="Confirm New Password"
                  :show-strength="false"
                  :validate="false"
                  class="mb-4"
                />

                <v-alert
                  v-if="statusMessage"
                  :type="messageType"
                  class="mb-4"
                  closable
                  @click:close="statusMessage = ''"
                >
                  {{ statusMessage }}
                </v-alert>

                <v-btn
                  color="primary"
                  size="large"
                  block
                  :loading="loading"
                  @click="handleSubmit"
                  class="mt-4"
                >
                  Reset Password
                </v-btn>
              </v-form>
            </div>

            <div v-else class="text-center">
              <v-icon color="success" size="64" class="mb-4">mdi-check-circle</v-icon>
              <p class="text-h6 mb-6">{{ statusMessage }}</p>
              <v-btn color="primary" to="/login" block size="large">
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
import { ref, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { completePasswordReset } from '@/services/api'
import { translate_response } from '@/wasm/translator.js'

import PasswordField from '@/components/auth/PasswordField.vue'

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

const handleSubmit = async () => {
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
    const { bytes } = await completePasswordReset({
      token: token.value,
      newPassword: password.value,
    })

    statusMessage.value = translate_response(bytes, undefined)
    messageType.value = 'success'
    completed.value = true
  } catch (e: any) {
    console.error('Password reset failed', e)
    statusMessage.value = e.message || 'An error occurred during password reset'
    messageType.value = 'error'
  } finally {
    loading.value = false
  }
}
</script>
