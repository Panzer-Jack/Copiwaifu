import type { MaybeRefOrGetter, Ref } from 'vue'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Config, LogLevel } from 'easy-live2d'
import { onMounted, onUnmounted, ref, toValue, watch } from 'vue'
import { createLive2DRuntime } from '../live2d/runtime'
import { createMotionController } from '../live2d/motion-controller'
import type { MotionGroupOption, TAgentState, WindowSizePreset } from '../types/agent'

export interface UseMainWindowLive2dOptions {
  canvasRef: Ref<HTMLCanvasElement | undefined>
  modelUrl: MaybeRefOrGetter<string>
  windowSize: MaybeRefOrGetter<WindowSizePreset>
  currentState: Ref<TAgentState>
  getActionGroupBindings: () => Record<TAgentState, string | null>
  getFallbackMotionGroups: () => readonly MotionGroupOption[]
  onModelReady?: () => void
}

export function useMainWindowLive2d(options: UseMainWindowLive2dOptions) {
  const currentWindow = getCurrentWindow()
  const runtimeReady = ref(false)

  let runtime: ReturnType<typeof createLive2DRuntime> | null = null
  let canvasResizeObserver: ResizeObserver | null = null
  let unlistenWindowResized: UnlistenFn | null = null
  let unlistenWindowScaleChanged: UnlistenFn | null = null

  const motionController = createMotionController({
    getSprite: () => runtime?.getSprite() ?? null,
    getCurrentState: () => options.currentState.value,
    getActionGroupBindings: options.getActionGroupBindings,
    getFallbackMotionGroups: options.getFallbackMotionGroups,
  })

  Config.MotionGroupIdle = 'Idle'
  Config.ViewScale = 1.7
  Config.CubismLoggingLevel = LogLevel.LogLevel_Off
  Config.MouseFollow = true

  async function syncSize() {
    await runtime?.syncSize()
  }

  async function mountModel(modelUrl: string) {
    if (!runtimeReady.value || !runtime || !modelUrl) {
      return
    }

    motionController.invalidate()
    await runtime.mountModel({
      modelEntryUrl: modelUrl,
      onReady: () => {
        options.onModelReady?.()
      },
    })
  }

  async function playState(state: TAgentState, force = false) {
    await motionController.playState(state, force)
  }

  async function refreshCurrentState(force = true) {
    motionController.syncIdleMotionGroupConfig()
    await motionController.playCurrentState(force)
  }

  onMounted(async () => {
    const canvas = options.canvasRef.value
    if (!canvas) {
      return
    }

    runtime = createLive2DRuntime({
      canvas,
      resizeTo: canvas.parentElement ?? window,
      resolution: Math.max(window.devicePixelRatio || 1, 1),
    })

    motionController.syncIdleMotionGroupConfig()
    await runtime.init()
    runtimeReady.value = true
    await syncSize()

    canvasResizeObserver = new ResizeObserver(() => {
      void syncSize()
    })
    canvasResizeObserver.observe(canvas)

    unlistenWindowResized = await currentWindow.onResized(() => {
      void syncSize()
    })
    unlistenWindowScaleChanged = await currentWindow.onScaleChanged(() => {
      void syncSize()
    })
  })

  onUnmounted(() => {
    runtimeReady.value = false
    motionController.invalidate()

    if (unlistenWindowResized) {
      void unlistenWindowResized()
      unlistenWindowResized = null
    }

    if (unlistenWindowScaleChanged) {
      void unlistenWindowScaleChanged()
      unlistenWindowScaleChanged = null
    }

    if (canvasResizeObserver) {
      canvasResizeObserver.disconnect()
      canvasResizeObserver = null
    }

    runtime?.dispose()
    runtime = null
  })

  watch(
    () => [runtimeReady.value, toValue(options.modelUrl)] as const,
    ([ready, modelUrl]) => {
      if (!ready || !modelUrl) {
        return
      }

      void mountModel(modelUrl)
    },
    { immediate: true },
  )

  watch(
    () => toValue(options.windowSize),
    () => {
      void syncSize()
    },
  )

  return {
    playState,
    refreshCurrentState,
    syncIdleMotionGroupConfig: motionController.syncIdleMotionGroupConfig,
  }
}
