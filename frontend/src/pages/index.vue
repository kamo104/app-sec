<template>
  <v-container class="py-8">
    <!-- Header Section -->
    <v-row justify="center" class="mb-6">
      <v-col cols="12" md="10" class="text-center">
        <h1 class="text-h4 font-weight-bold mb-2">MemeShark</h1>
        <p class="text-body-1 text-grey">
          The ocean of memes
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
  </v-container>
</template>

<script lang="ts" setup>
</script>
