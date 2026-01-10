<template>
  <v-container class="fill-height" max-width="600">
    <v-row justify="center">
      <v-col cols="12" md="8" lg="6">
        <v-card class="pa-6" elevation="2" rounded="lg">
          <v-card-title class="text-h5 text-center mb-4">
            Email Verification
          </v-card-title>

          <v-card-text>
            <!-- Loading State -->
            <div v-if="loading" class="text-center py-8">
              <v-progress-circular
                indeterminate
                color="primary"
                size="64"
                class="mb-4"
              />
              <div class="text-body-1">Verifying your email...</div>
            </div>

            <!-- Success State -->
            <v-alert
              v-else-if="success"
              type="success"
              variant="tonal"
              class="mb-4"
              icon="mdi-check-circle"
            >
              <div class="text-body-1 font-weight-bold mb-2">Email Verified!</div>
              <div class="text-body-2">
                {{ message }}
              </div>
            </v-alert>

            <!-- Error State -->
            <v-alert
              v-else-if="error"
              type="error"
              variant="tonal"
              class="mb-4"
              icon="mdi-alert-circle"
            >
              <div class="text-body-1 font-weight-bold mb-2">Verification Failed</div>
              <div class="text-body-2">
                {{ message }}
              </div>
            </v-alert>

            <!-- No Token State -->
            <v-alert
              v-else
              type="warning"
              variant="tonal"
              class="mb-4"
              icon="mdi-help-circle"
            >
              <div class="text-body-1 font-weight-bold mb-2">No Token Provided</div>
              <div class="text-body-2">
                The verification link is missing a token. Please check your email and use the complete verification link.
              </div>
            </v-alert>
          </v-card-text>

          <v-card-actions class="justify-center pt-0">
            <v-btn
              v-if="success"
              color="primary"
              variant="elevated"
              href="/login"
              prepend-icon="mdi-login"
            >
              Go to Login
            </v-btn>

            <v-btn
              v-if="error || !hasToken"
              color="secondary"
              variant="outlined"
              href="/"
              prepend-icon="mdi-home"
            >
              Return Home
            </v-btn>
          </v-card-actions>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { StatusCodes } from 'http-status-codes'
import { verifyEmail, type VerifyEmailErrorResponse } from '@/api/client'
import { translate, translate_error_code } from '@/wasm/translator.js'

const route = useRoute()
const router = useRouter()

const loading = ref(true)
const success = ref(false)
const error = ref(false)
const message = ref('')
const hasToken = ref(false)

const verifyToken = async () => {
  // Extract token from URL query parameter
  const token = route.query.token as string

  if (!token) {
    loading.value = false
    hasToken.value = false
    message.value = 'No token provided in the URL.'
    return
  }

  hasToken.value = true

  try {
    const { data, error: apiError, response } = await verifyEmail({ body: { token } })

    if (data) {
      message.value = translate('SUCCESS_EMAIL_VERIFIED', undefined)
      success.value = true
    } else if (response.status === StatusCodes.BAD_REQUEST && apiError) {
      error.value = true
      const errorResponse = apiError as VerifyEmailErrorResponse
      message.value = translate_error_code(errorResponse.error, undefined)
    } else {
      error.value = true
      message.value = translate('INTERNAL', undefined)
    }
  } catch (err) {
    error.value = true
    message.value = 'Cannot connect to the server. Please try again later.'
  } finally {
    loading.value = false
  }
}

// Run verification when component mounts
onMounted(() => {
  verifyToken()
})
</script>

<style scoped>
.v-card {
  transition: transform 0.2s ease;
}

.v-card:hover {
  transform: translateY(-2px);
}
</style>
