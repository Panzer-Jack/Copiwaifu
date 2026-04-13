<script setup lang="ts">
import { computed } from 'vue'
import type { WindowSizePreset } from '../types/agent'

const props = defineProps<{
  text: string
  visible: boolean
  windowSize: WindowSizePreset
}>()

const bubbleClassName = computed(() => `speech-bubble--${props.windowSize}`)
</script>

<template>
  <Transition name="bubble">
    <div
      v-if="visible"
      class="speech-bubble"
      :class="bubbleClassName"
    >
      <span class="speech-bubble__text">{{ text }}</span>
    </div>
  </Transition>
</template>

<style scoped>
.speech-bubble {
  --bubble-width: min(172px, calc(100vw - 16px));
  --bubble-max-height: 120px;
  --bubble-min-height: 50px;
  --bubble-padding: 12px 18px;
  --bubble-radius: 18px;
  --bubble-border-width: 2px;
  --bubble-font-size: 14px;
  --bubble-pointer-size: 10px;
  position: absolute;
  top: 4%;
  left: 50%;
  width: var(--bubble-width);
  max-height: var(--bubble-max-height);
  min-height: var(--bubble-min-height);
  box-sizing: border-box;
  overflow: hidden;
  padding: var(--bubble-padding);
  background: rgba(220, 245, 230, 0.65);
  border: var(--bubble-border-width) solid rgba(150, 210, 170, 0.6);
  border-radius: var(--bubble-radius);
  backdrop-filter: blur(12px);
  filter: drop-shadow(0 1px 6px rgba(160, 220, 180, 0.25));
  pointer-events: none;
  z-index: 10;
  transform-origin: bottom center;
  transform: translateX(-50%);
}

.speech-bubble--small {
  --bubble-width: min(156px, calc(100vw - 16px));
  --bubble-max-height: 102px;
  --bubble-min-height: 42px;
  --bubble-padding: 10px 14px;
  --bubble-radius: 14px;
  --bubble-border-width: 1.5px;
  --bubble-font-size: 12px;
  --bubble-pointer-size: 8px;
}

.speech-bubble--medium {
  --bubble-width: min(184px, calc(100vw - 16px));
  --bubble-max-height: 120px;
  --bubble-min-height: 50px;
  --bubble-padding: 12px 18px;
  --bubble-radius: 18px;
  --bubble-border-width: 2px;
  --bubble-font-size: 14px;
  --bubble-pointer-size: 10px;
}

.speech-bubble--large {
  --bubble-width: min(228px, calc(100vw - 20px));
  --bubble-max-height: 148px;
  --bubble-min-height: 60px;
  --bubble-padding: 14px 22px;
  --bubble-radius: 22px;
  --bubble-border-width: 2px;
  --bubble-font-size: 15px;
  --bubble-pointer-size: 12px;
}

/* 底部三角尖角 */
.speech-bubble::after {
  content: '';
  position: absolute;
  bottom: calc(var(--bubble-pointer-size) * -1);
  left: 50%;
  transform: translateX(-50%);
  width: 0;
  height: 0;
  border-left: var(--bubble-pointer-size) solid transparent;
  border-right: var(--bubble-pointer-size) solid transparent;
  border-top: var(--bubble-pointer-size) solid rgba(220, 245, 230, 0.65);
}

.speech-bubble__text {
  display: block;
  font-family: 'M PLUS Rounded 1c', 'Outfit', 'PingFang SC', sans-serif;
  font-size: var(--bubble-font-size);
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
