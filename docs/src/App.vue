<script setup lang="ts">
import main from '@/assets/images/main.png'
</script>

<template>
 
  <div
    class="bg-gothic-bg w-100vw h-100vh fixed z--1"
    :style="{ backgroundImage: `url(${main})`, backgroundSize: 'cover', backgroundPosition: '80%' }"
  />

</template>

<style scoped>
.main-fade-enter-active {
  transition-property: all;
  transition-duration: 500ms;
  transition-timing-function: ease;
}

.main-fade-enter-from {
  opacity: 0;
  transform: translateY(-1.25rem);
}
</style>
