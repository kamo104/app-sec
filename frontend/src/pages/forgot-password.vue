<template>
  <AuthFormLayout title="Forgot Password">
    <template #default="{ handleSubmit: formSubmit }">
      <v-form @submit.prevent="handleSubmit" ref="form" validate-on="input lazy">
        <p class="text-body-1 mb-6">
          Enter your email address or username and we'll send you a link to reset your password.
        </p>

        <UsernameField
          ref="usernameField"
          v-model="username"
          label="Email or Username"
          :validate-length="false"
        />

        <StatusMessage
          v-if="statusMessage"
          :message="statusMessage"
          :type="messageType"
          @close="clearMessage"
        />

        <AuthSubmitButton
          label="Send Reset Link"
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
        prepend-icon="mdi-arrow-left"
      >
        Back to Login
      </v-btn>
    </template>
  </AuthFormLayout>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { requestPasswordReset } from '@/api/client'
import { translate_success_code } from '@/wasm/translator.js'

// Import reusable components
import AuthFormLayout from '@/components/auth/AuthFormLayout.vue'
import UsernameField from '@/components/auth/UsernameField.vue'
import AuthSubmitButton from '@/components/auth/AuthSubmitButton.vue'
import StatusMessage from '@/components/auth/StatusMessage.vue'

const username = ref('')
const loading = ref(false)
const statusMessage = ref('')
const messageType = ref<'success' | 'error' | 'warning' | 'info'>('success')

// Refs to component instances
const usernameField = ref<any>(null)

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
  clearMessage()

  // Validate field
  const validation = await usernameField.value?.validate()
  if (validation?.valid === false) {
    return
  }

  if (!username.value) {
    showMessage('Please enter your email or username', 'warning')
    return
  }

  loading.value = true

  try {
    const { data, error } = await requestPasswordReset({ body: { email: username.value } })

    if (data) {
      showMessage(translate_success_code(data.success, undefined), 'success')
      username.value = ''
    } else if (error) {
      // requestPasswordReset always returns success for security, so this branch
      // only handles unexpected network errors
      showMessage('An error occurred during password reset request', 'error')
    }
  } catch (e: any) {
    console.error('Password reset request failed', e)
    showMessage('An error occurred during password reset request', 'error')
  } finally {
    loading.value = false
  }
}
</script>
