<template>
  <div class="auth-remember-me-wrapper">
    <v-checkbox
      :model-value="modelValue"
      @update:model-value="handleInput"
      label="Remember me"
      density="compact"
      hide-details
    ></v-checkbox>
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

const handleInput = (value: boolean | null) => {
  emit('update:modelValue', value === null ? false : value)
}
</script>

<style scoped>
/* Common styles imported from src/styles/auth.css */
</style>