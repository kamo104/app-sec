<template>
  <v-app>
    <v-app-bar app color="primary" density="comfortable">
      <v-app-bar-title>
        <v-icon icon="mdi-account-plus" class="mr-2"></v-icon>
        User Registration Demo
      </v-app-bar-title>
    </v-app-bar>

    <v-main>
      <v-container class="py-8">
        <!-- Header Section -->
        <v-row justify="center" class="mb-6">
          <v-col cols="12" md="8" class="text-center">
            <h1 class="text-h4 font-weight-bold mb-2">User Registration Component</h1>
            <p class="text-body-1 text-grey">
              Complete the form below with frontend validation for username, email, and password
            </p>
            <v-divider class="mt-4"></v-divider>
          </v-col>
        </v-row>

        <!-- User Registration Component -->
        <UserRegistration />

        <!-- Features List -->
        <v-row justify="center" class="mt-6">
          <v-col cols="12" md="8">
            <v-card variant="tonal" color="info" class="pa-4">
              <v-card-title class="d-flex align-center">
                <v-icon icon="mdi-shield-check" class="mr-2"></v-icon>
                Validation Features
              </v-card-title>
              <v-card-text>
                <v-list density="compact">
                  <v-list-item>
                    <v-list-item-title>Username Validation:</v-list-item-title>
                    <v-list-item-subtitle>3-20 chars, letters/numbers/underscores only</v-list-item-subtitle>
                  </v-list-item>
                  <v-list-item>
                    <v-list-item-title>Email Validation:</v-list-item-title>
                    <v-list-item-subtitle>Required format: user@domain.com</v-list-item-subtitle>
                  </v-list-item>
                  <v-list-item>
                    <v-list-item-title>Password Validation:</v-list-item-title>
                    <v-list-item-subtitle>8+ chars, uppercase, lowercase, number, special char</v-list-item-subtitle>
                  </v-list-item>
                  <v-list-item>
                    <v-list-item-title>Confirm Password:</v-list-item-title>
                    <v-list-item-subtitle>Must match password exactly</v-list-item-subtitle>
                  </v-list-item>
                </v-list>
              </v-card-text>
            </v-card>
          </v-col>
        </v-row>
      </v-container>
    </v-main>

    <v-footer app class="text-center">
      <div class="text-caption">
        Vue 3 + Vuetify 3 • Frontend Validation Demo
      </div>
    </v-footer>
  </v-app>
</template>

<script lang="ts" setup>
  import UserRegistration from '@/components/UserRegistration.vue'
</script>
