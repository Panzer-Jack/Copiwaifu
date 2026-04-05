<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { getLanguageCopy } from './i18n'
import { checkForAppUpdates } from './updater'
import MainWindow from './windows/MainWindow.vue'
import SettingsWindow from './windows/SettingsWindow.vue'
import { APP_LANGUAGE } from './types/agent'
import type { AppBootstrap, WindowVisibilityPayload } from './types/agent'

const windowLabel = ref(getCurrentWindow().label)
const bootstrap = ref<AppBootstrap | null>(null)
const errorMessage = ref('')
const ui = computed(() => getLanguageCopy(bootstrap.value?.settings.language ?? APP_LANGUAGE.ENGLISH))

let unlistenSettings: UnlistenFn | null = null
let unlistenVisibility: UnlistenFn | null = null
let hasCheckedForUpdates = false
let updateCheckTimer: number | null = null

async function loadBootstrap() {
  try {
    bootstrap.value = await invoke<AppBootstrap>('get_app_bootstrap')
    errorMessage.value = ''
  }
  catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error)
  }
}

async function maybeCheckForUpdates() {
  if (
    hasCheckedForUpdates
    || import.meta.env.DEV
    || windowLabel.value !== 'main'
    || !bootstrap.value
  ) {
    return
  }

  hasCheckedForUpdates = true

  try {
    await checkForAppUpdates(bootstrap.value.settings.language ?? APP_LANGUAGE.ENGLISH)
  }
  catch (error) {
    console.warn('failed to check for updates', error)
  }
}

function scheduleUpdateCheck() {
  if (updateCheckTimer !== null) {
    window.clearTimeout(updateCheckTimer)
  }

  updateCheckTimer = window.setTimeout(() => {
    updateCheckTimer = null
    void maybeCheckForUpdates()
  }, 3_000)
}

onMounted(async () => {
  await loadBootstrap()
  scheduleUpdateCheck()

  unlistenSettings = await listen<AppBootstrap>('settings:updated', (event) => {
    bootstrap.value = event.payload
  })

  unlistenVisibility = await listen<WindowVisibilityPayload>('window:visibility-changed', (event) => {
    if (!bootstrap.value) {
      return
    }

    bootstrap.value = {
      ...bootstrap.value,
      mainWindowVisible: event.payload.visible,
    }
  })
})

onUnmounted(() => {
  if (unlistenSettings) {
    void unlistenSettings()
  }
  if (unlistenVisibility) {
    void unlistenVisibility()
  }
  if (updateCheckTimer !== null) {
    window.clearTimeout(updateCheckTimer)
  }
})
</script>

<template>
  <main
    v-if="bootstrap"
    class="shell"
    :class="`shell--${windowLabel}`"
  >
    <SettingsWindow
      v-if="windowLabel === 'settings'"
      :bootstrap="bootstrap"
    />
    <MainWindow
      v-else
      :bootstrap="bootstrap"
    />
  </main>

  <main
    v-else
    class="shell shell--loading"
  >
    <div
      v-if="errorMessage"
      class="status-card status-card--error"
    >
      <h1>{{ ui.status.launchFailed }}</h1>
      <p>{{ errorMessage }}</p>
    </div>
    <div
      v-else
      class="status-card"
    >
      <h1>Copiwaifu</h1>
      <p>{{ ui.status.syncing }}</p>
    </div>
  </main>
</template>

<style>
html,
body,
#app {
  width: 100%;
  height: 100%;
  margin: 0;
  overflow: hidden;
}

body {
  overscroll-behavior: none;
}
</style>

<style scoped>
.shell {
  width: 100vw;
  height: 100vh;
}

.shell--loading,
.shell--settings {
  display: flex;
  align-items: stretch;
  justify-content: stretch;
  overflow-y: auto;
  background:
    radial-gradient(circle at top left, rgba(255, 229, 189, 0.95), transparent 42%),
    linear-gradient(160deg, #f7f3ea, #e7efe8 58%, #dde7ef);
  color: #203031;
}

.shell--main {
  background: transparent;
  overflow: hidden;
}

.status-card {
  margin: auto;
  width: min(320px, calc(100vw - 32px));
  padding: 24px;
  border: 1px solid rgba(77, 107, 107, 0.16);
  border-radius: 22px;
  background: rgba(255, 255, 255, 0.82);
  box-shadow: 0 22px 60px rgba(51, 80, 79, 0.12);
  backdrop-filter: blur(12px);
}

.status-card--error {
  border-color: rgba(170, 83, 83, 0.24);
}

.status-card h1 {
  margin: 0;
  font-size: 22px;
  line-height: 1.2;
}

.status-card p {
  margin: 12px 0 0;
  font-size: 14px;
  line-height: 1.6;
}
</style>
