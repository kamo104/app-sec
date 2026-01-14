<template>
  <v-btn
    block
    color="primary"
    :disabled="loading || disabled"
    :loading="loading"
    size="large"
    type="submit"
    @click="handleClick"
  >
    {{ label }}
  </v-btn>
</template>

<script setup lang="ts">
  const props = defineProps<{
    label: string
    loading?: boolean
    disabled?: boolean
  }>()

  const emit = defineEmits<{
    click: []
  }>()

  function handleClick () {
    emit('click')
  }
</script>

<style scoped>
/* Consistent button styling */
.v-btn {
  transition: opacity 0.2s ease, transform 0.1s ease;
}

.v-btn:not(:disabled):hover {
  opacity: 0.9;
}

.v-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
