<template>
  <AuthFormLayout title="Forgot Password">
    <template #default="{ handleSubmit: formSubmit }">
      <v-form ref="form" validate-on="input lazy" @submit.prevent="handleSubmit">
        <p class="text-body-1 mb-6">
          Enter your email address and we'll send you a link to reset your password.
        </p>

        <EmailField
          ref="emailField"
          v-model="email"
        />

        <StatusMessage
          v-if="statusMessage"
          :message="statusMessage"
          :type="messageType"
          @close="clearMessage"
        />

        <AuthSubmitButton
          :disabled="loading"
          label="Send Reset Link"
          :loading="loading"
          @click="() => formSubmit(handleSubmit)"
        />
      </v-form>
    </template>

    <template #navigation>
      <v-btn
        class="text-none"
        color="primary"
        prepend-icon="mdi-arrow-left"
        to="/login"
        variant="text"
      >
        Back to Login
      </v-btn>
    </template>
  </AuthFormLayout>
</template>

<script setup lang="ts">
  import { ref } from 'vue'
  import { requestPasswordReset } from '@/api/client'
  // Import reusable components
  import AuthFormLayout from '@/components/auth/AuthFormLayout.vue'

  import AuthSubmitButton from '@/components/auth/AuthSubmitButton.vue'
  import EmailField from '@/components/auth/EmailField.vue'
  import StatusMessage from '@/components/auth/StatusMessage.vue'
  import { translate } from '@/wasm/translator.js'

  const email = ref('')
  const loading = ref(false)
  const statusMessage = ref('')
  const messageType = ref<'success' | 'error' | 'warning' | 'info'>('success')

  // Refs to component instances
  const emailField = ref<any>(null)

  // Clear status message
  function clearMessage () {
    statusMessage.value = ''
  }

  // Display a message with the given type
  function showMessage (message: string, type: 'success' | 'error' | 'warning' | 'info' = 'success') {
    statusMessage.value = message
    messageType.value = type
  }

  // Main form submission handler
  async function handleSubmit () {
    clearMessage()

    // Validate field
    const validation = await emailField.value?.validate()
    if (validation?.valid === false) {
      return
    }

    if (!email.value) {
      showMessage('Please enter your email address', 'warning')
      return
    }

    loading.value = true

    try {
      const { data, error } = await requestPasswordReset({ body: { email: email.value } })

      if (data) {
        showMessage(translate('SUCCESS_PASSWORD_RESET_REQUESTED', undefined), 'success')
        email.value = ''
      } else if (error) {
        // requestPasswordReset always returns success for security, so this branch
        // only handles unexpected network errors
        showMessage('An error occurred during password reset request', 'error')
      }
    } catch (error: any) {
      console.error('Password reset request failed', error)
      showMessage('An error occurred during password reset request', 'error')
    } finally {
      loading.value = false
    }
  }
</script>
