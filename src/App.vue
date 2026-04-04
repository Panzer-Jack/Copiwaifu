<script setup lang="ts">
import { Config, Live2DSprite, LogLevel, Priority } from 'easy-live2d'
import { Application, Ticker } from 'pixi.js'
import { onMounted, onUnmounted, ref, watch } from 'vue'
import PermissionBubble from './components/PermissionBubble.vue'
import SpeechBubble from './components/SpeechBubble.vue'
import { useAgentState } from './composables/useAgentState'
import { usePermission } from './composables/usePermission'
import { useSpeechBubble } from './composables/useSpeechBubble'
import type { AgentState } from './types/agent'

const canvasRef = ref<HTMLCanvasElement>()
const { isVisible, displayedText, say, hide } = useSpeechBubble()
const { currentState, activeAgent, sessionInfo } = useAgentState()
const { pendingRequest, isSubmitting, approve, deny } = usePermission()
const modelReady = ref(false)
const lastAnimatedState = ref<AgentState>('idle')
let idleGreetingTimer: ReturnType<typeof setInterval> | null = null

const greetings = [
  '你好呀～今天也要加油哦！',
  '嗨！有什么我能帮你的吗？',
  '我在这里盯着你的 AI 会话。',
]
const app = new Application()

Config.MotionGroupIdle = 'Idle'
Config.ViewScale = 2.5
Config.CubismLoggingLevel = LogLevel.LogLevel_Off

const live2DSprite = new Live2DSprite()
live2DSprite.init({
  modelPath: '/Resources/Hiyori/Hiyori.model3.json',
  ticker: Ticker.shared,
})

function randomGreeting() {
  return greetings[Math.floor(Math.random() * greetings.length)]
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

async function animateForState(state: AgentState) {
  if (!modelReady.value || state === lastAnimatedState.value) {
    return
  }

  lastAnimatedState.value = state

  if (state === 'tool_use' || state === 'error' || state === 'waiting_permission') {
    try {
      await live2DSprite.startMotion({
        group: 'TapBody',
        no: 0,
        priority: Priority.Force,
      })
    }
    catch (error) {
      console.warn('failed to start live2d motion', error)
    }
  }
}

function bubbleTextForState(state: AgentState) {
  if (state === 'thinking') {
    return '思考中...'
  }
  if (state === 'tool_use') {
    return sessionInfo.value.toolName
      ? `正在执行：${sessionInfo.value.toolName}`
      : '正在执行操作...'
  }
  if (state === 'error') {
    return '出错了，我先帮你盯住。'
  }
  return ''
}

function startIdleGreetingLoop() {
  if (idleGreetingTimer) {
    clearInterval(idleGreetingTimer)
  }

  idleGreetingTimer = setInterval(() => {
    if (currentState.value !== 'idle' || pendingRequest.value) {
      return
    }
    say(randomGreeting(), 2600)
  }, 18000)
}

onMounted(async () => {
  const resolution = Math.max(window.devicePixelRatio || 1, 1)

  await app.init({
    canvas: canvasRef.value,
    backgroundAlpha: 0,
    autoDensity: true,
    resizeTo: window,
    resolution,
  })

  if (canvasRef.value) {
    live2DSprite.width = canvasRef.value.clientWidth
    live2DSprite.height = canvasRef.value.clientHeight

    app.stage.addChild(live2DSprite)

    live2DSprite.onLive2D('ready', () => {
      modelReady.value = true
      say(randomGreeting(), 2800)
      startIdleGreetingLoop()
    })
  }
})

watch(currentState, (state, previous) => {
  void animateForState(state)

  if (pendingRequest.value) {
    return
  }

  if (state === 'idle') {
    if (previous && previous !== 'idle') {
      say(`${formatAgentLabel(activeAgent.value)} 这轮处理完成了。`, 2400)
    }
    return
  }

  const text = bubbleTextForState(state)
  if (text) {
    say(text, 2200)
  }
})

watch(pendingRequest, (request) => {
  if (request) {
    hide()
    void animateForState('waiting_permission')
    return
  }

  if (currentState.value === 'idle') {
    say(randomGreeting(), 2200)
  }
}, { immediate: true })

onUnmounted(() => {
  if (idleGreetingTimer) {
    clearInterval(idleGreetingTimer)
  }
  hide()
  live2DSprite.destroy()
})
</script>

<template>
  <div class="container">
    <canvas
      id="live2d"
      ref="canvasRef"
      data-tauri-drag-region
    />
    <PermissionBubble
      :visible="Boolean(pendingRequest)"
      :tool-name="pendingRequest?.tool_name ?? ''"
      :summary="pendingRequest?.summary ?? ''"
      :busy="isSubmitting"
      @approve="approve"
      @deny="deny"
    />
    <SpeechBubble
      :text="displayedText"
      :visible="isVisible && !pendingRequest"
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
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  pointer-events: auto;
  z-index: 1;
  cursor: move;
}
</style>
