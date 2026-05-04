<script setup lang="ts">
import type { UnlistenFn } from '@tauri-apps/api/event'
import type { AgentType, AppBootstrap, TAgentState } from '../types/agent'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import PetContextMenu from '../components/PetContextMenu.vue'
import SpeechBubble from '../components/SpeechBubble.vue'
import { useAgentState } from '../composables/useAgentState'
import { useContextMenu } from '../composables/useContextMenu'
import { useMainWindowLive2d } from '../composables/useMainWindowLive2d'
import { limitAiTalkBubbleText, useSpeechBubble } from '../composables/useSpeechBubble'
import { formatAgentLabel, getLanguageCopy } from '../i18n'
import { AGENT_STATE } from '../types/agent'

const props = defineProps<{
  bootstrap: AppBootstrap
}>()

const canvasRef = ref<HTMLCanvasElement>()
const { isVisible, displayedText, say, hide } = useSpeechBubble()
const { currentState, activeAgent, serverPort, sessionInfo } = useAgentState()
const lastActiveAgent = ref<AgentType | null>(null)
let idleGreetingTimer: ReturnType<typeof setInterval> | null = null
let sameStateBubbleTimer: ReturnType<typeof setTimeout> | null = null
let lastStateBubbleShownAt = 0
let aiTalkRequestToken = 0
let unlistenAiTalkDebug: UnlistenFn | null = null
let aiTalkDebugListenerDisposed = false

const MENU_WIDTH = 176
const MENU_HEIGHT = 196
const MENU_EDGE_GAP = 12
const SAME_STATE_BUBBLE_REFRESH_COOLDOWN_MS = 4500

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

function currentAgentLabel() {
  return formatAgentLabel(
    activeAgent.value ?? lastActiveAgent.value,
    props.bootstrap.settings.language,
  )
}

function handleWindowKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    closeMenu()
  }
}

function bubbleTextForState(state: TAgentState) {
  const name = props.bootstrap.settings.name
  const agentLabel = currentAgentLabel()
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

const stateBubbleText = computed(() => bubbleTextForState(currentState.value))
const aiTalkTriggerKey = computed(() => {
  const context = sessionInfo.value.aiTalkContext
  if (!context) {
    return ''
  }

  return [
    context.agent,
    context.sessionId,
    context.state,
    context.turnIndex,
    context.updatedAtMs,
  ].join(':')
})

interface AiTalkGenerateResponse {
  text: string
}

interface AiTalkDebugPayload {
  stage: string
  data: unknown
}

function clearSameStateBubbleTimer() {
  if (sameStateBubbleTimer) {
    clearTimeout(sameStateBubbleTimer)
    sameStateBubbleTimer = null
  }
}

function isUrgentBubbleState(state: TAgentState) {
  return state === AGENT_STATE.ERROR || state === AGENT_STATE.NEEDS_ATTENTION
}

function isAiTalkTerminalState(state: TAgentState) {
  return state === AGENT_STATE.COMPLETE || state === AGENT_STATE.ERROR
}

function showStateBubble(text: string, duration = 2200) {
  lastStateBubbleShownAt = Date.now()
  say(text, duration)
}

async function showAiTalkOrFallback(state: TAgentState, fallbackText: string) {
  const agent = activeAgent.value ?? lastActiveAgent.value
  const sessionId = sessionInfo.value.sessionId
  if (!agent || !sessionId) {
    showStateBubble(fallbackText)
    return
  }

  const token = ++aiTalkRequestToken
  const request = {
    agent,
    sessionId,
    state,
    windowSize: props.bootstrap.settings.windowSize,
    language: props.bootstrap.settings.language,
  }

  console.log('[AI Talk] generate request', {
    request,
    context: sessionInfo.value.aiTalkContext,
  })

  try {
    const response = await invoke<AiTalkGenerateResponse | null>('generate_ai_talk', {
      request,
    })

    console.log('[AI Talk] generate response', response)

    if (token !== aiTalkRequestToken || currentState.value !== state) {
      return
    }

    const text = response?.text
      ? limitAiTalkBubbleText(
          response.text,
          props.bootstrap.settings.windowSize,
          props.bootstrap.settings.language,
        )
      : fallbackText

    if (text) {
      showStateBubble(text, response?.text ? 2800 : 2200)
    }
  } catch (error) {
    console.warn('failed to generate AI Talk bubble', error)
    if (token === aiTalkRequestToken && currentState.value === state) {
      showStateBubble(fallbackText)
    }
  }
}

function scheduleSameStateBubbleRefresh(delay: number) {
  if (sameStateBubbleTimer) {
    return
  }

  sameStateBubbleTimer = setTimeout(() => {
    sameStateBubbleTimer = null

    if (currentState.value === AGENT_STATE.IDLE) {
      return
    }

    const text = stateBubbleText.value
    if (text) {
      showStateBubble(text)
    }
  }, delay)
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
  aiTalkDebugListenerDisposed = false
  window.addEventListener('click', closeMenu)
  window.addEventListener('blur', closeMenu)
  window.addEventListener('keydown', handleWindowKeydown)
  void listen<AiTalkDebugPayload>('ai-talk:debug', (event) => {
    console.log('[AI Talk debug]', event.payload.stage, event.payload.data)
  })
    .then((unlisten) => {
      if (aiTalkDebugListenerDisposed) {
        unlisten()
        return
      }
      unlistenAiTalkDebug = unlisten
    })
    .catch((error) => {
      console.warn('failed to listen for AI Talk debug events', error)
    })
})

onUnmounted(() => {
  aiTalkDebugListenerDisposed = true
  if (unlistenAiTalkDebug) {
    void unlistenAiTalkDebug()
    unlistenAiTalkDebug = null
  }
  if (idleGreetingTimer) {
    clearInterval(idleGreetingTimer)
  }
  clearSameStateBubbleTimer()
  window.removeEventListener('click', closeMenu)
  window.removeEventListener('blur', closeMenu)
  window.removeEventListener('keydown', handleWindowKeydown)
  hide()
})

watch(activeAgent, (agent) => {
  if (agent) {
    lastActiveAgent.value = agent
  }
})

watch([currentState, stateBubbleText, aiTalkTriggerKey], ([state, text, triggerKey], [previousState, previousText, previousTriggerKey]) => {
  if (state !== previousState) {
    clearSameStateBubbleTimer()
    aiTalkRequestToken += 1
    void playState(state)
  }

  if (state === AGENT_STATE.IDLE) {
    if (previousState && previousState !== AGENT_STATE.IDLE) {
      showStateBubble(
        ui.value.pet.idleResume(
          currentAgentLabel(),
          props.bootstrap.settings.name,
        ),
        2600,
      )
    }
    return
  }

  const aiTalkTriggerChanged = triggerKey && triggerKey !== previousTriggerKey

  if (!text || (state === previousState && text === previousText && !aiTalkTriggerChanged)) {
    return
  }

  if (isAiTalkTerminalState(state)) {
    clearSameStateBubbleTimer()
    void showAiTalkOrFallback(state, text)
    return
  }

  if (state !== previousState || isUrgentBubbleState(state)) {
    clearSameStateBubbleTimer()
    showStateBubble(text)
    return
  }

  const elapsed = Date.now() - lastStateBubbleShownAt
  if (elapsed >= SAME_STATE_BUBBLE_REFRESH_COOLDOWN_MS) {
    showStateBubble(text)
    return
  }

  scheduleSameStateBubbleRefresh(SAME_STATE_BUBBLE_REFRESH_COOLDOWN_MS - elapsed)
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
  <div
    class="safe-top"
    data-tauri-drag-region
  />
  <div
    class="container"
    :class="`container--${props.bootstrap.settings.windowSize}`"
    @contextmenu="openMenu"
  >
    <section
      class="speech-region"
      data-tauri-drag-region
    >
      <SpeechBubble
        :text="displayedText"
        :visible="isVisible"
        :window-size="props.bootstrap.settings.windowSize"
      />
    </section>

    <section
      class="live2d-region"
      data-tauri-drag-region
    >
      <canvas
        id="live2d"
        ref="canvasRef"
        data-tauri-drag-region
      />
    </section>

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
  </div>
</template>

<style scoped>
.safe-top {
  height: var(--main-window-safe-top-height, 20px);
  cursor: move;
}
.container {
  --speech-region-height: 144px;
  position: relative;
  display: grid;
  grid-template-rows: var(--speech-region-height) minmax(0, 1fr);
  width: 100vw;
  height: calc(100vh - var(--main-window-safe-top-height, 20px));
  overflow: hidden;
  background: transparent;
  user-select: none;
  -webkit-user-select: none;
}

.container--tiny {
  --speech-region-height: 92px;
}

.container--small {
  --speech-region-height: 124px;
}

.container--medium {
  --speech-region-height: 144px;
}

.container--large {
  --speech-region-height: 172px;
}

.container--huge {
  --speech-region-height: 192px;
}

.speech-region {
  position: relative;
  min-height: 0;
  overflow: visible;
  cursor: move;
  z-index: 2;
  /* background: rgba(255, 100, 100, 0.15); */
}

.live2d-region {
  position: relative;
  min-height: 0;
  /* overflow: visible; */
  cursor: move;
  z-index: 1;
  /* background: rgba(100, 100, 255, 0.15); */
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
