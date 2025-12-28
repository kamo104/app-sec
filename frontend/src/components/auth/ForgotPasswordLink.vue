<template>
  <div class="auth-forgot-password-wrapper">
    <v-btn
      variant="text"
      color="info"
      size="small"
      class="auth-forgot-password-button"
      to="/forgot-password"
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

const handleClick = () => {
  emit('click')
}
</script>

<style scoped>
/* Common styles imported from src/styles/auth.css */
</style>