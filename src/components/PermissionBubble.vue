<script setup lang="ts">
defineProps<{
  visible: boolean
  toolName: string
  summary: string
  busy?: boolean
}>()

defineEmits<{
  approve: []
  deny: []
}>()
</script>

<template>
  <Transition name="bubble">
    <div
      v-if="visible"
      class="permission-bubble"
    >
      <div class="permission-bubble__eyebrow">
        AI Navigator
      </div>
      <div class="permission-bubble__title">
        {{ toolName }}
      </div>
      <div class="permission-bubble__summary">
        {{ summary }}
      </div>
      <div class="permission-bubble__actions">
        <button
          class="permission-bubble__button permission-bubble__button--allow"
          :disabled="busy"
          type="button"
          @click="$emit('approve')"
        >
          允许
        </button>
        <button
          class="permission-bubble__button permission-bubble__button--deny"
          :disabled="busy"
          type="button"
          @click="$emit('deny')"
        >
          拒绝
        </button>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.permission-bubble {
  position: absolute;
  top: 4px;
  right: 8px;
  width: min(188px, calc(100vw - 16px));
  padding: 14px 12px;
  border: 2px solid rgba(113, 178, 146, 0.7);
  border-radius: 18px;
  background:
    linear-gradient(145deg, rgba(244, 254, 248, 0.95), rgba(220, 244, 230, 0.92)),
    rgba(255, 255, 255, 0.85);
  box-shadow:
    0 18px 40px rgba(80, 140, 110, 0.18),
    inset 0 1px 0 rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(14px);
  pointer-events: auto;
  z-index: 20;
}

.permission-bubble::after {
  content: '';
  position: absolute;
  bottom: -9px;
  left: 64%;
  width: 18px;
  height: 18px;
  background: rgba(228, 248, 236, 0.95);
  border-right: 2px solid rgba(113, 178, 146, 0.7);
  border-bottom: 2px solid rgba(113, 178, 146, 0.7);
  transform: rotate(45deg);
}

.permission-bubble__eyebrow {
  font-size: 11px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: #5d8c74;
}

.permission-bubble__title {
  margin-top: 6px;
  font-size: 15px;
  font-weight: 700;
  color: #254539;
}

.permission-bubble__summary {
  margin-top: 8px;
  font-size: 12px;
  line-height: 1.5;
  color: #466356;
  word-break: break-word;
}

.permission-bubble__actions {
  display: flex;
  gap: 10px;
  margin-top: 12px;
}

.permission-bubble__button {
  flex: 1;
  min-height: 32px;
  border: 0;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 700;
  cursor: pointer;
  transition: transform 0.18s ease, opacity 0.18s ease, box-shadow 0.18s ease;
}

.permission-bubble__button:hover:not(:disabled) {
  transform: translateY(-1px);
}

.permission-bubble__button:disabled {
  cursor: wait;
  opacity: 0.65;
}

.permission-bubble__button--allow {
  background: linear-gradient(135deg, #72c88e, #4eab73);
  color: #fff;
  box-shadow: 0 8px 18px rgba(78, 171, 115, 0.28);
}

.permission-bubble__button--deny {
  background: rgba(255, 255, 255, 0.86);
  color: #436355;
  border: 1px solid rgba(140, 177, 157, 0.85);
}

.bubble-enter-active {
  animation: bubble-pop-in 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.bubble-leave-active {
  animation: bubble-pop-out 0.28s cubic-bezier(0.55, 0, 1, 0.45) forwards;
}

@keyframes bubble-pop-in {
  0% {
    opacity: 0;
    transform: scale(0) translateY(12px);
  }
  50% {
    transform: scale(1.06) translateY(-2px);
  }
  100% {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}

@keyframes bubble-pop-out {
  0% {
    opacity: 1;
    transform: scale(1);
  }
  100% {
    opacity: 0;
    transform: scale(0.2) translateY(16px);
  }
}
</style>
