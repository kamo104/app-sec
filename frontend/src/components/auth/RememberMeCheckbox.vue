<template>
  <div class="auth-remember-me-wrapper">
    <v-checkbox
      density="compact"
      hide-details
      label="Remember me"
      :model-value="modelValue"
      @update:model-value="handleInput"
    />
  </div>
</template>

<script setup lang="ts">
  interface Props {
    modelValue: boolean
  }

  defineProps<Props>()

  const emit = defineEmits<{
    'update:modelValue': [value: boolean]
  }>()

  function handleInput (value: boolean | null) {
    emit('update:modelValue', value === null ? false : value)
  }
</script>

<style scoped>
/* Common styles imported from src/styles/auth.css */
</style>
