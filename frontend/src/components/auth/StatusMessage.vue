<template>
  <v-alert
    v-if="message"
    class="mb-4"
    closable
    density="compact"
    :type="type"
    variant="tonal"
    @click:close="handleClose"
  >
    {{ message }}
  </v-alert>
</template>

<script setup lang="ts">
  const props = defineProps<{
    message: string
    type?: 'success' | 'error' | 'warning' | 'info'
  }>()

  const emit = defineEmits<{
    close: []
  }>()

  function handleClose () {
    emit('close')
  }
</script>

<style scoped>
/* Consistent alert styling */
.v-alert {
  transition: opacity 0.2s ease;
}

/* Ensure the alert has proper spacing */
:deep(.v-alert__content) {
  font-size: 0.875rem;
  line-height: 1.4;
}

/* Ensure close button is accessible */
:deep(.v-alert__close-button) {
  min-width: 24px;
  min-height: 24px;
}
</style>
