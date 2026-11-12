import type { MaybeRefOrGetter, Ref } from 'vue'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Config, LogLevel } from 'easy-live2d'
import { onMounted, onUnmounted, ref, toValue, watch } from 'vue'
import { createLive2DRuntime } from '../live2d/runtime'
import { createMotionController } from '../live2d/motion-controller'
import type { ModelScanResult, TAgentState, WindowSizePreset } from '../types/agent'

export interface UseMainWindowLive2dOptions {
  canvasRef: Ref<HTMLCanvasElement | undefined>
  modelUrl: MaybeRefOrGetter<string>
  windowSize: MaybeRefOrGetter<WindowSizePreset>
  currentState: Ref<TAgentState>
  getActionGroupBindings: () => Record<TAgentState, string | null>
  getModelScan: () => ModelScanResult
  onModelReady?: () => void
}

export function useMainWindowLive2d(options: UseMainWindowLive2dOptions) {
  const currentWindow = getCurrentWindow()
  const runtimeReady = ref(false)
  const modelError = ref<string | null>(null)

  let runtime: ReturnType<typeof createLive2DRuntime> | null = null
  let canvasResizeObserver: ResizeObserver | null = null
  let unlistenWindowResized: UnlistenFn | null = null
  let unlistenWindowScaleChanged: UnlistenFn | null = null

  const motionController = createMotionController({
    getSprite: () => runtime?.getSprite() ?? null,
    getCurrentState: () => options.currentState.value,
    getActionGroupBindings: options.getActionGroupBindings,
    getFallbackMotionGroups: () => options.getModelScan().availableMotionGroups,
  })

  Config.MotionGroupIdle = 'Idle'
  Config.ViewScale = 1.46
  Config.CubismLoggingLevel = LogLevel.LogLevel_Off
  Config.MouseFollow = true

  async function syncSize() {
    await runtime?.syncSize()
  }

  function onModelError(error: unknown) {
    modelError.value = error instanceof Error ? error.message : String(error)
  }

  async function mountModel(modelUrl: string) {
    if (!runtimeReady.value || !runtime || !modelUrl) {
      return
    }

    motionController.invalidate()
    modelError.value = null
    await runtime.mountModel({
      modelEntryUrl: modelUrl,
      onError: onModelError,
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
    try {
      await runtime.init()
    }
    catch (error) {
      onModelError(error)
      return
    }
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
    [runtimeReady, () => toValue(options.modelUrl), options.getModelScan],
    ([ready, modelUrl], [wasReady, previousUrl]) => {
      if (!ready || !modelUrl) {
        return
      }
      // Reimporting a custom model can replace the files at the same URL.
      if (wasReady && modelUrl === previousUrl && modelError.value === null) {
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
    modelError,
    playState,
    refreshCurrentState,
    syncIdleMotionGroupConfig: motionController.syncIdleMotionGroupConfig,
  }
}
