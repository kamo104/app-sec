<template>
  <v-container class="fill-height" max-width="600">
    <v-row justify="center">
      <v-col cols="12" md="8">
        <v-card class="pa-6" elevation="2">
          <v-card-title>{{ title }}</v-card-title>
          <slot :form-ref="formRef" :handle-submit="handleSubmit" name="default" />

          <!-- Navigation Links -->
          <v-card-text class="text-center pt-4">
            <slot name="navigation" />
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup lang="ts">
  import { ref } from 'vue'

  const props = defineProps<{
    title: string
  }>()

  const formRef = ref<any>(null)

  // Expose form validation to parent
  async function validateForm (): Promise<{ valid: boolean }> {
    if (formRef.value) {
      return await formRef.value.validate()
    }
    return { valid: false }
  }

  // Expose submit handler
  async function handleSubmit (submitFn: () => Promise<void>) {
    const { valid } = await validateForm()
    if (valid) {
      await submitFn()
    }
  }

  // Expose methods to parent component
  defineExpose({
    validateForm,
    formRef,
  })
</script>

<style scoped>
/* Consistent container styling */
.fill-height {
  min-height: 100%;
}

/* Ensure card has consistent appearance */
.v-card {
  transition: box-shadow 0.2s ease;
}

.v-card:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15) !important;
}
</style>
