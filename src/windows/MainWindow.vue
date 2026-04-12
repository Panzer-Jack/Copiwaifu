<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import PetContextMenu from '../components/PetContextMenu.vue'
import SpeechBubble from '../components/SpeechBubble.vue'
import { useContextMenu } from '../composables/useContextMenu'
import { formatAgentLabel, getLanguageCopy } from '../i18n'
import { useAgentState } from '../composables/useAgentState'
import { useMainWindowLive2d } from '../composables/useMainWindowLive2d'
import { useSpeechBubble } from '../composables/useSpeechBubble'
import { AGENT_STATE } from '../types/agent'
import type { AppBootstrap, TAgentState } from '../types/agent'

const props = defineProps<{
  bootstrap: AppBootstrap
}>()

const canvasRef = ref<HTMLCanvasElement>()
const { isVisible, displayedText, say, hide } = useSpeechBubble()
const { currentState, activeAgent, serverPort, sessionInfo } = useAgentState()
let idleGreetingTimer: ReturnType<typeof setInterval> | null = null

const MENU_WIDTH = 176
const MENU_HEIGHT = 196
const MENU_EDGE_GAP = 12

const { menuState, closeMenu, openMenu } = useContextMenu({
  width: MENU_WIDTH,
  height: MENU_HEIGHT,
  edgeGap: MENU_EDGE_GAP,
})

const activeModelUrl = computed(() => {
  if (props.bootstrap.settings.modelDirectory && serverPort.value) {
    return `http://127.0.0.1:${serverPort.value}/model/current/${encodeURIComponent(props.bootstrap.modelScan.modelEntryFile)}`
  }

  return props.bootstrap.modelUrl
})

const ui = computed(() => getLanguageCopy(props.bootstrap.settings.language))
const visibilityLabel = computed(() => (
  ui.value.visibilityLabel(props.bootstrap.mainWindowVisible)
))

const {
  playState,
  refreshCurrentState,
  syncIdleMotionGroupConfig,
} = useMainWindowLive2d({
  canvasRef,
  modelUrl: activeModelUrl,
  windowSize: computed(() => props.bootstrap.settings.windowSize),
  currentState,
  getActionGroupBindings: () => props.bootstrap.settings.actionGroupBindings,
  getFallbackMotionGroups: () => props.bootstrap.modelScan.availableMotionGroups,
  onModelReady: () => {
    say(randomGreeting(), 2800)
    startIdleGreetingLoop()
    void refreshCurrentState()
  },
})

function greetingLines(name: string) {
  return ui.value.pet.greetings(name)
}

function randomGreeting() {
  const lines = greetingLines(props.bootstrap.settings.name)
  return lines[Math.floor(Math.random() * lines.length)]
}

function handleWindowKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    closeMenu()
  }
}

function bubbleTextForState(state: TAgentState) {
  const name = props.bootstrap.settings.name
  const agentLabel = formatAgentLabel(activeAgent.value, props.bootstrap.settings.language)
  if (state === AGENT_STATE.THINKING) {
    return ui.value.pet.thinking(agentLabel, name)
  }
  if (state === AGENT_STATE.TOOL_USE) {
    return ui.value.pet.toolUse(agentLabel, name, sessionInfo.value.toolName)
  }
  if (state === AGENT_STATE.ERROR) {
    return ui.value.pet.error(agentLabel, name)
  }
  if (state === AGENT_STATE.COMPLETE) {
    return ui.value.pet.complete(agentLabel, name)
  }
  if (state === AGENT_STATE.NEEDS_ATTENTION) {
    return ui.value.pet.needsAttention(agentLabel, name)
  }
  return ''
}

function startIdleGreetingLoop() {
  if (idleGreetingTimer) {
    clearInterval(idleGreetingTimer)
  }

  idleGreetingTimer = setInterval(() => {
    if (currentState.value !== AGENT_STATE.IDLE) {
      return
    }
    say(randomGreeting(), 2600)
  }, 18000)
}

async function openSettings() {
  closeMenu()
  await invoke('open_settings_window')
}

async function toggleVisibility() {
  closeMenu()
  await invoke('toggle_main_window_visibility')
}

async function exitApp() {
  closeMenu()
  await invoke('exit_app')
}

onMounted(() => {
  window.addEventListener('click', closeMenu)
  window.addEventListener('blur', closeMenu)
  window.addEventListener('keydown', handleWindowKeydown)
})

onUnmounted(() => {
  if (idleGreetingTimer) {
    clearInterval(idleGreetingTimer)
  }
  window.removeEventListener('click', closeMenu)
  window.removeEventListener('blur', closeMenu)
  window.removeEventListener('keydown', handleWindowKeydown)
  hide()
})

watch(currentState, (state, previous) => {
  void playState(state)

  if (state === AGENT_STATE.IDLE) {
    if (previous && previous !== AGENT_STATE.IDLE) {
      say(
        ui.value.pet.idleResume(
          formatAgentLabel(activeAgent.value, props.bootstrap.settings.language),
          props.bootstrap.settings.name,
        ),
        2600,
      )
    }
    return
  }

  const text = bubbleTextForState(state)
  if (text) {
    say(text, 2200)
  }
})

watch(
  () => props.bootstrap.settings.actionGroupBindings,
  () => {
    syncIdleMotionGroupConfig()
    void refreshCurrentState()
  },
  { deep: true },
)
</script>

<template>
  <div class="safe-top" />
  <div
    class="container"
    @contextmenu="openMenu"
  >
    <canvas
      id="live2d"
      ref="canvasRef"
      data-tauri-drag-region
    />

    <PetContextMenu
      :visible="menuState.visible"
      :x="menuState.x"
      :y="menuState.y"
      :close-label="ui.menu.close"
      :settings-label="ui.menu.settings"
      :visibility-label="visibilityLabel"
      :exit-label="ui.menu.exit"
      @close="closeMenu"
      @open-settings="openSettings"
      @toggle-visibility="toggleVisibility"
      @exit="exitApp"
    />

    <SpeechBubble
      :text="displayedText"
      :visible="isVisible"
      :window-size="props.bootstrap.settings.windowSize"
    />
  </div>
</template>

<style scoped>

.safe-top {
  height: 20px;
}
.container {
  position: relative;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
  background: transparent;
  user-select: none;
  -webkit-user-select: none;
}

#live2d {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  cursor: move;
  z-index: 1;
}
</style>
