<template>
  <v-footer
    app
    height="40"
    class="px-4"
  >
    <div class="text-caption text-disabled">
      &copy; {{ (new Date()).getFullYear() }} MemeShark — The ocean of memes
    </div>

    <v-spacer></v-spacer>

    <div class="text-caption text-disabled">
      Built with security in mind
    </div>
  </v-footer>
</template>

<script setup lang="ts">
  // No items needed
</script>

<style scoped lang="sass">
</style>
