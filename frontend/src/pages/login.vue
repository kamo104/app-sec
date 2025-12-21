<template>
  <v-container class="py-8 fill-height" fluid>
    <v-row justify="center" align="center" class="fill-height">
      <v-col cols="12" md="6" lg="5">
        <UserLogin />
      </v-col>
    </v-row>
  </v-container>
</template>

<script lang="ts" setup>
import UserLogin from '@/components/UserLogin.vue'
</script>
