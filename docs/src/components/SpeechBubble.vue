<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  text: string
  visible: boolean
  x?: number
  y?: number
}>()

const bubbleStyle = computed(() => {
  if (typeof props.x !== 'number' || typeof props.y !== 'number') {
    return undefined
  }

  return {
    left: `${props.x}px`,
    top: `${props.y}px`,
    transform: 'translate(-50%, -100%)',
  }
})
</script>

<template>
  <Transition name="bubble">
    <div
      v-if="visible"
      class="speech-bubble"
      :style="bubbleStyle"
    >
      <span class="speech-bubble__text">{{ text }}</span>
    </div>
  </Transition>
</template>

<style scoped>
.speech-bubble {
  position: absolute;
  top: 0.4%;
  left: 50%;
  width: min(188px, calc(100% - 20px));
  max-height: 128px;
  min-height: 52px;
  box-sizing: border-box;
  overflow: hidden;
  padding: 12px 18px;
  background: rgba(220, 245, 230, 0.38);
  border: 2px solid rgba(150, 210, 170, 0.32);
  border-radius: 18px;
  backdrop-filter: blur(10px);
  filter: drop-shadow(0 1px 6px rgba(160, 220, 180, 0.16));
  pointer-events: none;
  z-index: 10;
  transform: translate(-50%, -100%);
  transform-origin: bottom center;
}

.speech-bubble::after {
  content: '';
  position: absolute;
  bottom: -10px;
  left: 50%;
  transform: translateX(-50%);
  width: 0;
  height: 0;
  border-left: 10px solid transparent;
  border-right: 10px solid transparent;
  border-top: 10px solid rgba(220, 245, 230, 0.38);
}

.speech-bubble__text {
  display: block;
  color: #4a5568;
  font-size: 14px;
  line-height: 1.6;
  font-weight: 500;
  text-align: center;
  word-break: break-word;
}

.bubble-enter-active {
  animation: bubble-pop-in 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.bubble-leave-active {
  animation: bubble-pop-out 0.3s cubic-bezier(0.55, 0, 1, 0.45) forwards;
}

@keyframes bubble-pop-in {
  0% {
    opacity: 0;
    transform: translate(-50%, -100%) scale(0) translateY(10px);
  }
  50% {
    transform: translate(-50%, -100%) scale(1.08) translateY(-2px);
  }
  100% {
    opacity: 1;
    transform: translate(-50%, -100%) scale(1) translateY(0);
  }
}

@keyframes bubble-pop-out {
  0% {
    opacity: 1;
    transform: translate(-50%, -100%) scale(1);
  }
  50% {
    transform: translate(-50%, -100%) scale(1.05);
  }
  100% {
    opacity: 0;
    transform: translate(-50%, -100%) scale(0) translateY(10px);
  }
}
</style>
