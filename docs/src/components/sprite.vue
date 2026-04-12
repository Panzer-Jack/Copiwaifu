<script setup lang="ts">
import { Config, Live2DSprite, LogLevel, Priority } from 'easy-live2d'
import { Application, Ticker } from 'pixi.js'
import { nextTick, onMounted, onUnmounted, ref } from 'vue'
import { useSpeechBubble } from '@/composables/useSpeechBubble'
import SpeechBubble from './SpeechBubble.vue'

const stageRef = ref<HTMLDivElement>()
const canvasRef = ref<HTMLCanvasElement>()
const bubbleX = ref<number>()
const bubbleY = ref<number>()

const bubbleLines = [
  'Ciallo ～(∠・ω< )⌒★!',
  'わたしはあなたのLive2D AIナビゲーターだよ、えへへ😋 ',
  '長官！CC、Codex、Copilotとの同期が完了しました。',
  '一緒にこの世界を変えましょう！',
  '我是你的 Live2D AI 领航员，嘻嘻😋 ',
  '长官！我们已经与CC、Codex、Copilot同步完成。',
  '让我们一起改变这个世界吧！',
  'I\'m your Live2D AI Navigator, hehe😋 ',
  'Commander! We\'ve synced with CC, Codex & Copilot.',
  'Let\'s change the world together!',
]
const { isVisible: bubbleVisible, displayedText: bubbleText, say, hide } = useSpeechBubble()

const pixiApp = new Application()
let live2DSprite: Live2DSprite | null = null
let resizeObserver: ResizeObserver | null = null
let sizeSyncToken = 0
let bubbleRevealTimer: ReturnType<typeof setTimeout> | null = null
let bubbleSequenceTimer: ReturnType<typeof setTimeout> | null = null
let bubbleRevealStarted = false
let bubbleRevealToken = 0
let bubbleLineIndex = 0

const MODEL_HEAD_X_RATIO = 0.5
const MODEL_HEAD_Y_RATIO = 0.16
const MODEL_BOTTOM_PADDING = 6
const BUBBLE_MIN_EDGE_GAP = 96
const BUBBLE_SHOW_DELAY = 180
const BUBBLE_LINE_DURATION = 2200
const BUBBLE_LINE_GAP = 500

Config.MotionGroupIdle = 'Idle'
Config.MouseFollow = true
Config.ViewScale = 1
Config.CubismLoggingLevel = LogLevel.LogLevel_Off

function updateBubblePosition() {
  if (!stageRef.value || !live2DSprite) {
    return
  }

  const modelCanvasSize = live2DSprite.getModelCanvasSize()
  if (!modelCanvasSize) {
    return
  }

  const stageWidth = stageRef.value.clientWidth
  const stageHeight = stageRef.value.clientHeight
  if (stageWidth <= 0 || stageHeight <= 0) {
    return
  }

  // easy-live2d 最终是按模型原始比例缩放到舞台内，这里直接用 contain 逻辑估算当前显示框。
  const uniformScale = Math.min(
    stageWidth / modelCanvasSize.width,
    stageHeight / modelCanvasSize.height,
  )
  const renderedWidth = modelCanvasSize.width * uniformScale
  const renderedHeight = modelCanvasSize.height * uniformScale
  const modelLeft = (stageWidth - renderedWidth) / 2
  const modelTop = stageHeight - renderedHeight - MODEL_BOTTOM_PADDING

  const headAnchorX = modelLeft + renderedWidth * MODEL_HEAD_X_RATIO
  const headAnchorY = modelTop + renderedHeight * MODEL_HEAD_Y_RATIO

  bubbleX.value = Math.min(
    Math.max(headAnchorX, BUBBLE_MIN_EDGE_GAP),
    Math.max(BUBBLE_MIN_EDGE_GAP, stageWidth - BUBBLE_MIN_EDGE_GAP),
  )
  bubbleY.value = Math.max(48, headAnchorY)
}

function clearBubbleRevealTimer() {
  if (bubbleRevealTimer) {
    clearTimeout(bubbleRevealTimer)
    bubbleRevealTimer = null
  }
}

function clearBubbleSequenceTimer() {
  if (bubbleSequenceTimer) {
    clearTimeout(bubbleSequenceTimer)
    bubbleSequenceTimer = null
  }
}

function scheduleNextBubbleLine() {
  clearBubbleSequenceTimer()
  const line = bubbleLines[bubbleLineIndex]
  say(line, BUBBLE_LINE_DURATION)

  const totalDuration = Math.min(line.length, 100) * 60 + BUBBLE_LINE_DURATION + BUBBLE_LINE_GAP
  bubbleSequenceTimer = setTimeout(() => {
    bubbleLineIndex = (bubbleLineIndex + 1) % bubbleLines.length
    scheduleNextBubbleLine()
  }, totalDuration)
}

function startBubbleSequenceWithDelay() {
  clearBubbleRevealTimer()
  clearBubbleSequenceTimer()
  hide()
  bubbleRevealTimer = setTimeout(() => {
    scheduleNextBubbleLine()
    bubbleRevealTimer = null
  }, BUBBLE_SHOW_DELAY)
}

async function waitForNextFrame() {
  await new Promise<void>((resolve) => {
    requestAnimationFrame(() => {
      resolve()
    })
  })
}

async function syncLive2DSize() {
  const token = ++sizeSyncToken
  await nextTick()
  await new Promise<void>((resolve) => {
    requestAnimationFrame(() => {
      resolve()
    })
  })

  if (token !== sizeSyncToken || !stageRef.value || !live2DSprite) {
    return
  }

  live2DSprite.width = canvasRef.value?.clientWidth || stageRef.value.clientWidth
  live2DSprite.height = canvasRef.value?.clientHeight || stageRef.value.clientHeight
  live2DSprite.onResize()

  await new Promise<void>((resolve) => {
    requestAnimationFrame(() => {
      updateBubblePosition()
      resolve()
    })
  })
}

async function revealBubbleWhenSettled() {
  if (bubbleRevealStarted) {
    return
  }

  bubbleRevealStarted = true
  const token = ++bubbleRevealToken

  await syncLive2DSize()
  await waitForNextFrame()
  await syncLive2DSize()
  await waitForNextFrame()

  if (token !== bubbleRevealToken || bubbleX.value == null || bubbleY.value == null) {
    return
  }

  startBubbleSequenceWithDelay()
}

function destroySprite() {
  if (!live2DSprite) {
    return
  }

  pixiApp.stage.removeChild(live2DSprite as any)
  live2DSprite.destroy()
  live2DSprite = null
}

async function mountSprite() {
  bubbleRevealStarted = false
  bubbleRevealToken++
  clearBubbleRevealTimer()
  clearBubbleSequenceTimer()
  bubbleLineIndex = 0
  hide()
  destroySprite()

  const sprite = new Live2DSprite({
    modelPath: '/Resources/Yulia/Yulia.model3.json',
    ticker: Ticker.shared,
  })

  sprite.onLive2D('ready', () => {
    void revealBubbleWhenSettled()
  })

  pixiApp.stage.addChild(sprite as any)
  live2DSprite = sprite
  await syncLive2DSize()
}

onMounted(async () => {
  if (!canvasRef.value || !stageRef.value) {
    return
  }

  const resolution = Math.max(window.devicePixelRatio || 1, 1)

  await pixiApp.init({
    canvas: canvasRef.value,
    backgroundAlpha: 0,
    autoDensity: true,
    resizeTo: stageRef.value,
    resolution,
  })

  await mountSprite()

  resizeObserver = new ResizeObserver(() => {
    void syncLive2DSize()
  })
  resizeObserver.observe(stageRef.value)
})

onUnmounted(() => {
  clearBubbleRevealTimer()
  clearBubbleSequenceTimer()
  bubbleRevealStarted = false
  bubbleRevealToken++
  hide()
  bubbleX.value = undefined
  bubbleY.value = undefined

  if (resizeObserver) {
    resizeObserver.disconnect()
    resizeObserver = null
  }

  destroySprite()
  pixiApp.destroy()
})
</script>

<template>
  <div ref="stageRef" class="sprite-stage">
    <SpeechBubble
      :text="bubbleText"
      :visible="bubbleVisible && bubbleX != null && bubbleY != null"
      :x="bubbleX"
      :y="bubbleY"
    />

    <canvas
      ref="canvasRef"
      class="sprite-stage__canvas"
    />
  </div>
</template>

<style scoped>
.sprite-stage {
  position: relative;
  width: 100%;
  height: 100%;
  overflow: hidden;
  border-radius: inherit;
}

.sprite-stage::after {
  position: absolute;
  inset: auto 10% 6%;
  height: 20%;
  border-radius: 999px;
  background: radial-gradient(circle, rgba(110, 168, 221, 0.28) 0%, rgba(110, 168, 221, 0) 72%);
  content: '';
  pointer-events: none;
  filter: blur(18px);
}

.sprite-stage__canvas {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  pointer-events: auto;
}
</style>
