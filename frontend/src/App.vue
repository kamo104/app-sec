<template>
  <v-app>
    <v-app-bar app color="primary" density="comfortable">
      <v-app-bar-title>
        <v-icon icon="mdi-account-circle" class="mr-2"></v-icon>
        User Authentication Demo
      </v-app-bar-title>
    </v-app-bar>

    <v-main>
      <v-container class="py-8">
        <!-- Header Section -->
        <v-row justify="center" class="mb-6">
          <v-col cols="12" md="10" class="text-center">
            <h1 class="text-h4 font-weight-bold mb-2">Authentication Components</h1>
            <p class="text-body-1 text-grey">
              User registration and login with frontend validation
            </p>
            <v-divider class="mt-4"></v-divider>
          </v-col>
        </v-row>

        <!-- Side by Side Components -->
        <v-row justify="center" class="mb-6">
          <!-- Registration Component -->
          <v-col cols="12" md="5">
            <UserRegistration />
          </v-col>

          <!-- Divider for desktop -->
          <v-col cols="12" md="2" class="d-flex align-center justify-center">
            <v-divider vertical class="d-none d-md-block"></v-divider>
            <v-divider class="d-block d-md-none my-4"></v-divider>
          </v-col>

          <!-- Login Component -->
          <v-col cols="12" md="5">
            <UserLogin />
          </v-col>
        </v-row>

        <!-- Features List -->
        <v-row justify="center" class="mt-6">
          <v-col cols="12" md="10">
            <v-card variant="tonal" color="info" class="pa-4">
              <v-card-title class="d-flex align-center">
                <v-icon icon="mdi-shield-check" class="mr-2"></v-icon>
                Validation Features
              </v-card-title>
              <v-card-text>
                <v-row>
                  <v-col cols="12" md="6">
                    <h3 class="text-subtitle-2 mb-2">Registration Validation</h3>
                    <v-list density="compact">
                      <v-list-item>
                        <v-list-item-title>Username:</v-list-item-title>
                        <v-list-item-subtitle>3-20 chars, letters/numbers/underscores</v-list-item-subtitle>
                      </v-list-item>
                      <v-list-item>
                        <v-list-item-title>Email:</v-list-item-title>
                        <v-list-item-subtitle>Valid email format required</v-list-item-subtitle>
                      </v-list-item>
                      <v-list-item>
                        <v-list-item-title>Password:</v-list-item-title>
                        <v-list-item-subtitle>8+ chars with mixed case, numbers, special chars</v-list-item-subtitle>
                      </v-list-item>
                    </v-list>
                  </v-col>
                  <v-col cols="12" md="6">
                    <h3 class="text-subtitle-2 mb-2">Login Features</h3>
                    <v-list density="compact">
                      <v-list-item>
                        <v-list-item-title>Username:</v-list-item-title>
                        <v-list-item-subtitle>Required field validation</v-list-item-subtitle>
                      </v-list-item>
                      <v-list-item>
                        <v-list-item-title>Password:</v-list-item-title>
                        <v-list-item-subtitle>Required field validation</v-list-item-subtitle>
                      </v-list-item>
                      <v-list-item>
                        <v-list-item-title>Remember Me:</v-list-item-title>
                        <v-list-item-subtitle>Persistent login option</v-list-item-subtitle>
                      </v-list-item>
                      <v-list-item>
                        <v-list-item-title>Reset Password:</v-list-item-title>
                        <v-list-item-subtitle>Link to password recovery</v-list-item-subtitle>
                      </v-list-item>
                    </v-list>
                  </v-col>
                </v-row>
              </v-card-text>
            </v-card>
          </v-col>
        </v-row>
      </v-container>
    </v-main>

    <v-footer app class="text-center">
      <div class="text-caption">
        Vue 3 + Vuetify 3 • Frontend Authentication Demo
      </div>
    </v-footer>
  </v-app>
</template>

<script lang="ts" setup>
  import UserRegistration from '@/components/UserRegistration.vue'
  import UserLogin from '@/components/UserLogin.vue'
</script>
