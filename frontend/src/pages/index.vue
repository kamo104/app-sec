<template>
  <v-container class="py-8">
    <v-row justify="center">
      <v-col cols="12" md="10">
        <v-card class="pa-6" elevation="2">
          <v-card-title class="text-h4 font-weight-bold mb-4">
            Welcome to MemeShark, {{ username }}!
          </v-card-title>

          <v-card-text>
            <p class="text-body-1 mb-6">
              You have successfully logged in. This is your personal dashboard where you can manage your memes and explore the ocean of content.
            </p>

            <!-- Counter Section -->
            <v-card variant="tonal" class="pa-4 mb-6 text-center">
              <div class="text-h5 mb-2">Meme Counter</div>
              <div class="text-h2 font-weight-bold color-primary mb-4">
                {{ counter }}
                <v-progress-circular
                  v-if="loadingCounter"
                  indeterminate
                  color="primary"
                  size="32"
                  width="3"
                  class="ml-2"
                ></v-progress-circular>
              </div>
              <v-btn
                color="primary"
                size="large"
                prepend-icon="mdi-plus"
                :disabled="loadingCounter"
                @click="incrementCounter"
              >
                Increment Counter
              </v-btn>
            </v-card>

            <v-row>
              <v-col v-for="n in 3" :key="n" cols="12" sm="4">
                <v-card variant="outlined" class="text-center pa-4 h-100">
                  <v-icon size="48" color="primary" class="mb-2">
                    {{ n === 1 ? 'mdi-image-multiple' : n === 2 ? 'mdi-heart' : 'mdi-trending-up' }}
                  </v-icon>
                  <div class="text-h6 mb-1">{{ n === 1 ? 'My Memes' : n === 2 ? 'Favorites' : 'Trending' }}</div>
                  <div class="text-body-2 text-grey">
                    {{ n === 1 ? 'Manage your uploaded memes' : n === 2 ? 'View memes you liked' : 'See what is hot right now' }}
                  </div>
                </v-card>
              </v-col>
            </v-row>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { getCounter, setCounter } from '@/services/api'
import { useAuthStore } from '@/stores/auth'

const authStore = useAuthStore()
const username = ref('User')
const counter = ref(0)
const loadingCounter = ref(false)

onMounted(async () => {
  // Load username from auth store
  if (authStore.user) {
    username.value = authStore.user.username
  }

  // Fetch counter data from server
  await fetchServerCounter()
})

const fetchServerCounter = async () => {
  loadingCounter.value = true
  try {
    const { counterData } = await getCounter()
    counter.value = Number(counterData.value)
  } catch (e) {
    console.error('Failed to fetch counter from server', e)
  } finally {
    loadingCounter.value = false
  }
}

const incrementCounter = async (): Promise<void> => {
  loadingCounter.value = true
  const newValue = counter.value + 1
  try {
    const { counterData } = await setCounter(newValue)
    counter.value = Number(counterData.value)
  } catch (e) {
    console.error('Failed to update counter on server', e)
  } finally {
    loadingCounter.value = false
  }
}
</script>
