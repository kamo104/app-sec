<template>
  <div class="auth-forgot-password-wrapper">
    <v-btn
      class="auth-forgot-password-button"
      color="info"
      size="small"
      to="/forgot-password"
      variant="text"
    >
      {{ label }}
    </v-btn>
  </div>
</template>

<script setup lang="ts">
  interface Props {
    label?: string
  }

  const props = withDefaults(defineProps<Props>(), {
    label: 'Forgot your password?',
  })

  const emit = defineEmits<{
    click: []
  }>()

  function handleClick () {
    emit('click')
  }
</script>

<style scoped>
/* Common styles imported from src/styles/auth.css */
</style>
