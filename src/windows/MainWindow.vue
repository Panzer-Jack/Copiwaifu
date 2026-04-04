<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Config, Live2DSprite, LogLevel, Priority } from 'easy-live2d'
import { Application, Ticker } from 'pixi.js'
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import PetContextMenu from '../components/PetContextMenu.vue'
import SpeechBubble from '../components/SpeechBubble.vue'
import { useAgentState } from '../composables/useAgentState'
import { useSpeechBubble } from '../composables/useSpeechBubble'
import { AGENT_STATE } from '../types/agent'
import type { AppBootstrap, TAgentState } from '../types/agent'

const props = defineProps<{
  bootstrap: AppBootstrap
}>()

const canvasRef = ref<HTMLCanvasElement>()
const modelReady = ref(false)
const isAppReady = ref(false)
const menuState = ref({
  visible: false,
  x: 0,
  y: 0,
})
const { isVisible, displayedText, say, hide } = useSpeechBubble()
const { currentState, activeAgent, serverPort, sessionInfo } = useAgentState()

const currentWindow = getCurrentWindow()
const pixiApp = new Application()
const live2DSprite = ref<Live2DSprite | null>(null)
const lastAnimatedState = ref<TAgentState>(AGENT_STATE.IDLE)
let idleGreetingTimer: ReturnType<typeof setInterval> | null = null
let canvasResizeObserver: ResizeObserver | null = null
let unlistenWindowResized: UnlistenFn | null = null
let unlistenWindowScaleChanged: UnlistenFn | null = null
let sizeSyncToken = 0

const MENU_WIDTH = 176
const MENU_HEIGHT = 196
const MENU_EDGE_GAP = 12

Config.MotionGroupIdle = 'Idle'
Config.ViewScale = 2.5
Config.CubismLoggingLevel = LogLevel.LogLevel_Off

const activeModelUrl = computed(() => {
  if (props.bootstrap.settings.modelDirectory && serverPort.value) {
    return `http://127.0.0.1:${serverPort.value}/model/current/${encodeURIComponent(props.bootstrap.modelScan.modelEntryFile)}`
  }

  return props.bootstrap.modelUrl
})

const visibilityLabel = computed(() => (
  props.bootstrap.mainWindowVisible ? 'Hide' : 'Show'
))

function closeMenu() {
  menuState.value.visible = false
}

function openMenu(event: MouseEvent) {
  event.preventDefault()
  const maxX = Math.max(MENU_EDGE_GAP, window.innerWidth - MENU_WIDTH - MENU_EDGE_GAP)
  const maxY = Math.max(MENU_EDGE_GAP, window.innerHeight - MENU_HEIGHT - MENU_EDGE_GAP)
  menuState.value = {
    visible: true,
    x: Math.min(Math.max(event.clientX, MENU_EDGE_GAP), maxX),
    y: Math.min(Math.max(event.clientY, MENU_EDGE_GAP), maxY),
  }
}

function greetingLines(name: string) {
  return [
    `${name} 在这里继续盯着你的 AI 会话。`,
    `${name} 已待命，有需要随时喊我。`,
    `${name} 会把工具状态和授权请求都看住。`,
  ]
}

function randomGreeting() {
  const lines = greetingLines(props.bootstrap.settings.name)
  return lines[Math.floor(Math.random() * lines.length)]
}

function formatAgentLabel(agent: string | null) {
  if (agent === 'claude-code') {
    return 'Claude Code'
  }
  if (agent === 'copilot') {
    return 'Copilot'
  }
  if (agent === 'codex') {
    return 'Codex'
  }
  return 'AI'
}

function handleWindowKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    closeMenu()
  }
}

async function syncLive2DSize() {
  const token = ++sizeSyncToken
  await nextTick()
  await new Promise<void>((resolve) => {
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        resolve()
      })
    })
  })

  if (token !== sizeSyncToken || !canvasRef.value || !isAppReady.value) {
    return
  }

  const width = Math.round(canvasRef.value.clientWidth)
  const height = Math.round(canvasRef.value.clientHeight)
  if (width <= 0 || height <= 0) {
    return
  }

  if (!live2DSprite.value) {
    return
  }

  live2DSprite.value.width = width
  live2DSprite.value.height = height
  live2DSprite.value.onResize()
}

function resolveMotionGroup(state: TAgentState): string | null {
  const availableGroups = new Set(props.bootstrap.modelScan.availableMotionGroups.map(group => group.id))
  const direct = props.bootstrap.settings.actionGroupBindings[state]
  if (direct && availableGroups.has(direct)) {
    return direct
  }

  const idleFallback = props.bootstrap.settings.actionGroupBindings[AGENT_STATE.IDLE]
  if (idleFallback && availableGroups.has(idleFallback)) {
    return idleFallback
  }

  return null
}

async function animateForState(state: TAgentState) {
  if (!modelReady.value || !live2DSprite.value || state === lastAnimatedState.value) {
    return
  }

  lastAnimatedState.value = state
  const motionGroup = resolveMotionGroup(state)
  if (!motionGroup) {
    return
  }

  try {
    await live2DSprite.value.startRandomMotion({
      group: motionGroup,
      priority: Priority.Force,
    })
  }
  catch (error) {
    console.warn('failed to start live2d motion', error)
  }
}

function bubbleTextForState(state: TAgentState) {
  if (state === AGENT_STATE.THINKING) {
    return `${props.bootstrap.settings.name} 正在思考中...`
  }
  if (state === AGENT_STATE.TOOL_USE) {
    return sessionInfo.value.toolName
      ? `${props.bootstrap.settings.name} 正在执行：${sessionInfo.value.toolName}`
      : `${props.bootstrap.settings.name} 正在执行操作...`
  }
  if (state === AGENT_STATE.ERROR) {
    return `${props.bootstrap.settings.name} 捕获到一个错误。`
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

function destroySprite() {
  modelReady.value = false
  if (live2DSprite.value) {
    pixiApp.stage.removeChild(live2DSprite.value as any)
    live2DSprite.value.destroy()
    live2DSprite.value = null
  }
}

async function mountModel(modelPath: string) {
  if (!isAppReady.value || !canvasRef.value || !modelPath) {
    return
  }

  destroySprite()
  lastAnimatedState.value = AGENT_STATE.IDLE

  const sprite = new Live2DSprite({
    modelPath,
    ticker: Ticker.shared,
  })

  sprite.onLive2D('ready', () => {
    modelReady.value = true
    say(randomGreeting(), 2800)
    startIdleGreetingLoop()
  })

  pixiApp.stage.addChild(sprite as any)
  live2DSprite.value = sprite
  await syncLive2DSize()
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

onMounted(async () => {
  const resolution = Math.max(window.devicePixelRatio || 1, 1)

  await pixiApp.init({
    canvas: canvasRef.value,
    backgroundAlpha: 0,
    autoDensity: true,
    resizeTo: window,
    resolution,
  })

  isAppReady.value = true
  await syncLive2DSize()
  if (canvasRef.value) {
    canvasResizeObserver = new ResizeObserver(() => {
      void syncLive2DSize()
    })
    canvasResizeObserver.observe(canvasRef.value)
  }
  unlistenWindowResized = await currentWindow.onResized(() => {
    void syncLive2DSize()
  })
  unlistenWindowScaleChanged = await currentWindow.onScaleChanged(() => {
    void syncLive2DSize()
  })
  window.addEventListener('click', closeMenu)
  window.addEventListener('blur', closeMenu)
  window.addEventListener('keydown', handleWindowKeydown)
})

onUnmounted(() => {
  if (idleGreetingTimer) {
    clearInterval(idleGreetingTimer)
  }
  if (unlistenWindowResized) {
    void unlistenWindowResized()
    unlistenWindowResized = null
  }
  if (unlistenWindowScaleChanged) {
    void unlistenWindowScaleChanged()
    unlistenWindowScaleChanged = null
  }
  window.removeEventListener('click', closeMenu)
  window.removeEventListener('blur', closeMenu)
  window.removeEventListener('keydown', handleWindowKeydown)
  if (canvasResizeObserver) {
    canvasResizeObserver.disconnect()
    canvasResizeObserver = null
  }
  destroySprite()
  hide()
})

watch(
  () => [isAppReady.value, activeModelUrl.value] as const,
  ([ready, modelUrl]) => {
    if (!ready || !modelUrl) {
      return
    }

    void mountModel(modelUrl)
  },
  { immediate: true },
)

watch(
  () => props.bootstrap.settings.windowSize,
  () => {
    void syncLive2DSize()
  },
)

watch(currentState, (state, previous) => {
  void animateForState(state)

  if (state === AGENT_STATE.IDLE) {
    if (previous && previous !== AGENT_STATE.IDLE) {
      say(`${formatAgentLabel(activeAgent.value)} 这轮处理完成了，${props.bootstrap.settings.name} 已接住。`, 2600)
    }
    return
  }

  const text = bubbleTextForState(state)
  if (text) {
    say(text, 2200)
  }
})
</script>

<template>
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
      :visibility-label="visibilityLabel"
      @close="closeMenu"
      @open-settings="openSettings"
      @toggle-visibility="toggleVisibility"
      @exit="exitApp"
    />

    <SpeechBubble
      :text="displayedText"
      :visible="isVisible"
    />
  </div>
</template>

<style scoped>
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
