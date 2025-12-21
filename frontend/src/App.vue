<template>
  <v-app>
    <v-app-bar app color="primary" density="comfortable">
      <v-app-bar-title>
        <v-icon icon="mdi-account-circle" class="mr-2"></v-icon>
        User Authentication Demo
      </v-app-bar-title>

      <v-spacer></v-spacer>

      <v-btn
        variant="text"
        to="/"
        prepend-icon="mdi-home"
        class="d-none d-md-flex"
      >
        Home
      </v-btn>

      <v-btn
        variant="text"
        to="/login"
        prepend-icon="mdi-login"
        class="d-none d-md-flex"
      >
        Login
      </v-btn>

      <v-btn
        variant="text"
        to="/register"
        prepend-icon="mdi-account-plus"
        class="d-none d-md-flex"
      >
        Register
      </v-btn>

      <!-- Mobile menu -->
      <v-menu>
        <template v-slot:activator="{ props }">
          <v-btn
            icon="mdi-dots-vertical"
            v-bind="props"
            class="d-md-none"
          ></v-btn>
        </template>
        <v-list>
          <v-list-item to="/">
            <v-list-item-title>
              <v-icon icon="mdi-home" class="mr-2"></v-icon>
              Home
            </v-list-item-title>
          </v-list-item>
          <v-list-item to="/login">
            <v-list-item-title>
              <v-icon icon="mdi-login" class="mr-2"></v-icon>
              Login
            </v-list-item-title>
          </v-list-item>
          <v-list-item to="/register">
            <v-list-item-title>
              <v-icon icon="mdi-account-plus" class="mr-2"></v-icon>
              Register
            </v-list-item-title>
          </v-list-item>
        </v-list>
      </v-menu>
    </v-app-bar>

    <v-main>
      <router-view />
    </v-main>

    <v-footer app class="text-center">
      <div class="text-caption">
        Vue 3 + Vuetify 3 • Frontend Authentication Demo
      </div>
    </v-footer>
  </v-app>
</template>

<script lang="ts" setup>
</script>
