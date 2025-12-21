<template>
  <v-container class="py-8">
    <!-- Header Section -->
    <v-row justify="center" class="mb-6">
      <v-col cols="12" md="10" class="text-center">
        <h1 class="text-h4 font-weight-bold mb-2">Authentication Demo</h1>
        <p class="text-body-1 text-grey">
          Vue 3 + Vuetify 3 • Secure User Authentication System
        </p>
        <v-divider class="mt-4"></v-divider>
      </v-col>
    </v-row>

    <!-- Navigation Buttons -->
    <v-row justify="center" class="mb-6">
      <v-col cols="12" md="8" class="text-center">
        <v-btn
          color="primary"
          size="x-large"
          to="/login"
          class="mx-2 my-2"
          prepend-icon="mdi-login"
          rounded="lg"
        >
          Login
        </v-btn>
        <v-btn
          color="secondary"
          size="x-large"
          to="/register"
          class="mx-2 my-2"
          prepend-icon="mdi-account-plus"
          variant="outlined"
          rounded="lg"
        >
          Register
        </v-btn>
      </v-col>
    </v-row>

    <!-- Features List -->
    <v-row justify="center" class="mt-6">
      <v-col cols="12" md="10">
        <v-card variant="tonal" color="info" class="pa-4">
          <v-card-title class="d-flex align-center">
            <v-icon icon="mdi-shield-check" class="mr-2"></v-icon>
            Security Features
          </v-card-title>
          <v-card-text>
            <v-row>
              <v-col cols="12" md="6">
                <h3 class="text-subtitle-2 mb-2">Registration</h3>
                <v-list density="compact">
                  <v-list-item>
                    <v-list-item-title>Username Validation</v-list-item-title>
                    <v-list-item-subtitle>3-20 chars, letters/numbers/underscores</v-list-item-subtitle>
                  </v-list-item>
                  <v-list-item>
                    <v-list-item-title>Email Verification</v-list-item-title>
                    <v-list-item-subtitle>Required before login</v-list-item-subtitle>
                  </v-list-item>
                  <v-list-item>
                    <v-list-item-title>Password Strength</v-list-item-title>
                    <v-list-item-subtitle>WASM-based validation</v-list-item-subtitle>
                  </v-list-item>
                </v-list>
              </v-col>
              <v-col cols="12" md="6">
                <h3 class="text-subtitle-2 mb-2">Login & Security</h3>
                <v-list density="compact">
                  <v-list-item>
                    <v-list-item-title>Argon2 Hashing</v-list-item-title>
                    <v-list-item-subtitle>Secure password storage</v-list-item-subtitle>
                  </v-list-item>
                  <v-list-item>
                    <v-list-item-title>Email Verification Check</v-list-item-title>
                    <v-list-item-subtitle>Blocks unverified logins</v-list-item-subtitle>
                  </v-list-item>
                  <v-list-item>
                    <v-list-item-title>SQLCipher Database</v-list-item-title>
                    <v-list-item-subtitle>Encrypted at rest</v-list-item-subtitle>
                  </v-list-item>
                </v-list>
              </v-col>
            </v-row>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script lang="ts" setup>
</script>
