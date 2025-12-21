<template>
  <v-btn
    type="submit"
    color="primary"
    size="large"
    block
    :loading="loading"
    :disabled="loading || disabled"
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

const handleClick = () => {
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