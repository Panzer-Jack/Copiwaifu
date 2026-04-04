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
  top: 4%;
  left: 50%;
  width: min(172px, calc(100vw - 16px));
  max-height: 120px;
  min-height: 50px;
  box-sizing: border-box;
  overflow: hidden;
  padding: 12px 18px;
  background: rgba(220, 245, 230, 0.65);
  border: 2px solid rgba(150, 210, 170, 0.6);
  border-radius: 18px;
  backdrop-filter: blur(12px);
  filter: drop-shadow(0 1px 6px rgba(160, 220, 180, 0.25));
  pointer-events: none;
  z-index: 10;
  transform-origin: bottom center;
  transform: translateX(-50%);
}

/* 底部三角尖角 */
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
  border-top: 10px solid rgba(220, 245, 230, 0.65);
}

.speech-bubble__text {
  display: block;
  font-size: 14px;
  line-height: 1.6;
  color: #4a5568;
  font-weight: 500;
  word-break: break-word;
  text-align: center;
}

/* 弹入动画 */
.bubble-enter-active {
  animation: bubble-pop-in 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);
}

/* 弹出动画 */
.bubble-leave-active {
  animation: bubble-pop-out 0.3s cubic-bezier(0.55, 0, 1, 0.45) forwards;
}

@keyframes bubble-pop-in {
  0% {
    opacity: 0;
    transform: translateX(-50%) scale(0) translateY(10px);
  }
  50% {
    transform: translateX(-50%) scale(1.08) translateY(-2px);
  }
  100% {
    opacity: 1;
    transform: translateX(-50%) scale(1) translateY(0);
  }
}

@keyframes bubble-pop-out {
  0% {
    opacity: 1;
    transform: translateX(-50%) scale(1);
  }
  50% {
    transform: translateX(-50%) scale(1.05);
  }
  100% {
    opacity: 0;
    transform: translateX(-50%) scale(0) translateY(10px);
  }
}
</style>
