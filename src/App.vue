<script setup lang="ts">
import { Config, Live2DSprite, LogLevel } from 'easy-live2d'
import { Application, Ticker } from 'pixi.js'
import { onMounted, onUnmounted, ref } from 'vue'
import SpeechBubble from './components/SpeechBubble.vue'
import { useSpeechBubble } from './composables/useSpeechBubble'

const canvasRef = ref<HTMLCanvasElement>()
const { isVisible, displayedText, say, hide } = useSpeechBubble()

const greetings = [
  '你好呀～今天也要加油哦！',
  '嗨！有什么我能帮你的吗？',
  '今天天气真不错呢～',
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
      const size = live2DSprite.getModelCanvasSize()
      if (size) {
        console.log('模型原始尺寸:', size.width, 'x', size.height)
      }

      const randomGreeting = greetings[Math.floor(Math.random() * greetings.length)]
      say(randomGreeting)
    })
  }
})

onUnmounted(() => {
  hide()
  live2DSprite.destroy()
})
</script>

<template>
  <div
    class="container"
    data-tauri-drag-region
  >
    <canvas
      id="live2d"
      ref="canvasRef"
      data-tauri-drag-region
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
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  pointer-events: auto;
  z-index: 1;
  cursor: move;
}
</style>
