<script setup lang="ts">
defineProps<{
  visible: boolean
  x: number
  y: number
  closeLabel: string
  settingsLabel: string
  visibilityLabel: string
  exitLabel: string
}>()

defineEmits<{
  close: []
  openSettings: []
  toggleVisibility: []
  exit: []
}>()
</script>

<template>
  <Transition name="menu">
    <div
      v-if="visible"
      class="menu"
      :style="{ left: `${x}px`, top: `${y}px` }"
      @click.stop
    >
      <button
        class="menu__item"
        type="button"
        @click="$emit('close')"
      >
        {{ closeLabel }}
      </button>
      <div class="menu__divider" />
      <button
        class="menu__item"
        type="button"
        @click="$emit('openSettings')"
      >
        {{ settingsLabel }}
      </button>
      <button
        class="menu__item"
        type="button"
        @click="$emit('toggleVisibility')"
      >
        {{ visibilityLabel }}
      </button>
      <button
        class="menu__item menu__item--danger"
        type="button"
        @click="$emit('exit')"
      >
        {{ exitLabel }}
      </button>
    </div>
  </Transition>
</template>

<style scoped>
.menu {
  position: fixed;
  min-width: 160px;
  padding: 8px;
  border: 1px solid rgba(33, 54, 52, 0.14);
  border-radius: 16px;
  background: rgba(251, 248, 241, 0.94);
  box-shadow:
    0 18px 40px rgba(23, 32, 31, 0.24),
    inset 0 1px 0 rgba(255, 255, 255, 0.88);
  backdrop-filter: blur(16px);
  z-index: 40;
}

.menu__item {
  width: 100%;
  padding: 10px 12px;
  border: 0;
  border-radius: 12px;
  background: transparent;
  color: #1d302f;
  font-size: 13px;
  text-align: left;
  cursor: pointer;
  transition: background-color 0.18s ease, transform 0.18s ease;
}

.menu__item:hover {
  background: rgba(137, 180, 157, 0.14);
  transform: translateX(1px);
}

.menu__item--danger {
  color: #8b3c3c;
}

.menu__divider {
  height: 1px;
  margin: 6px 4px;
  background: rgba(44, 67, 66, 0.12);
}

.menu-enter-active,
.menu-leave-active {
  transition: opacity 0.18s ease, transform 0.18s ease;
}

.menu-enter-from,
.menu-leave-to {
  opacity: 0;
  transform: translateY(6px);
}
</style>
