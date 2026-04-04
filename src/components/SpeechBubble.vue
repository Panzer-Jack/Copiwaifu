<script setup lang="ts">
defineProps<{
  text: string
  visible: boolean
}>()
</script>

<template>
  <Transition name="bubble">
    <div
      v-if="visible"
      class="speech-bubble"
    >
      <span class="speech-bubble__text">{{ text }}</span>
    </div>
  </Transition>
</template>

<style scoped>
.speech-bubble {
  position: absolute;
  top: 10%;
  right: 15%;
  max-width: 240px;
  max-height: 120px;
  overflow: hidden;
  padding: 12px 16px;
  background: white;
  border-radius: 16px;
  filter: drop-shadow(0 2px 8px rgba(0, 0, 0, 0.15));
  pointer-events: none;
  z-index: 10;
}

/* 底部三角尖角 */
.speech-bubble::after {
  content: '';
  position: absolute;
  bottom: -8px;
  left: 50%;
  transform: translateX(-50%);
  width: 0;
  height: 0;
  border-left: 8px solid transparent;
  border-right: 8px solid transparent;
  border-top: 8px solid white;
}

.speech-bubble__text {
  font-size: 14px;
  line-height: 1.5;
  color: #333;
  word-break: break-word;
}

/* 进场动画 */
.bubble-enter-active {
  transition: all 0.3s ease-out;
}

/* 退场动画 */
.bubble-leave-active {
  transition: all 0.25s ease-in;
}

.bubble-enter-from {
  opacity: 0;
  transform: scale(0.8);
}

.bubble-leave-to {
  opacity: 0;
}
</style>
