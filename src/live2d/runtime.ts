import type { Live2DSprite } from 'easy-live2d'
import { Application } from 'pixi.js'
import { createLive2DModelSprite } from './model'

const SPRITE_FIT_PADDING = 8

export interface MountLive2DModelOptions {
  modelEntryUrl: string
  onReady?: (sprite: Live2DSprite) => void
}

export interface CreateLive2DRuntimeOptions {
  canvas: HTMLCanvasElement
  resolution?: number
  resizeTo?: Window | HTMLElement
}

function waitForNextFrame() {
  return new Promise<void>((resolve) => {
    requestAnimationFrame(() => {
      resolve()
    })
  })
}

function fitSpriteToViewport(sprite: Live2DSprite, width: number, height: number) {
  const modelSize = sprite.getModelCanvasSize()
  const availableWidth = Math.max(1, width - SPRITE_FIT_PADDING * 2)
  const availableHeight = Math.max(1, height - SPRITE_FIT_PADDING * 2)

  if (!modelSize || modelSize.width <= 0 || modelSize.height <= 0) {
    sprite.width = availableWidth
    sprite.height = availableHeight
    sprite.x = SPRITE_FIT_PADDING
    sprite.y = SPRITE_FIT_PADDING
    return
  }

  const scale = Math.min(availableWidth / modelSize.width, availableHeight / modelSize.height)
  const fittedWidth = modelSize.width * scale
  const fittedHeight = modelSize.height * scale

  sprite.width = fittedWidth
  sprite.height = fittedHeight
  sprite.x = Math.round((width - fittedWidth) / 2)
  sprite.y = Math.round(height - fittedHeight - SPRITE_FIT_PADDING)
}

export function createLive2DRuntime(options: CreateLive2DRuntimeOptions) {
  const app = new Application()
  let initialized = false
  let disposed = false
  let sprite: Live2DSprite | null = null
  let mountToken = 0
  let resizeToken = 0

  async function init() {
    if (initialized || disposed) {
      return
    }

    await app.init({
      canvas: options.canvas,
      backgroundAlpha: 0,
      autoDensity: true,
      resizeTo: options.resizeTo ?? window,
      resolution: options.resolution ?? Math.max(window.devicePixelRatio || 1, 1),
    })

    initialized = true
  }

  function getSprite() {
    return sprite
  }

  function detachSprite() {
    if (!sprite) {
      return
    }

    app.stage.removeChild(sprite as any)
    sprite.destroy()
    sprite = null
  }

  function destroyModel() {
    mountToken += 1
    detachSprite()
  }

  async function syncSize() {
    if (!initialized || disposed || !sprite) {
      return
    }

    const token = ++resizeToken
    await waitForNextFrame()
    await waitForNextFrame()

    if (token !== resizeToken || disposed || !sprite) {
      return
    }

    const width = Math.round(options.canvas.clientWidth)
    const height = Math.round(options.canvas.clientHeight)
    if (width <= 0 || height <= 0) {
      return
    }

    fitSpriteToViewport(sprite, width, height)
    sprite.onResize()
  }

  async function mountModel(mountOptions: MountLive2DModelOptions) {
    if (!initialized || disposed) {
      return null
    }

    const token = ++mountToken
    const nextSprite = await createLive2DModelSprite({
      modelEntryUrl: mountOptions.modelEntryUrl,
    })

    if (disposed || token !== mountToken) {
      nextSprite.destroy()
      return null
    }

    nextSprite.onLive2D('ready', () => {
      if (disposed || token !== mountToken || sprite !== nextSprite) {
        return
      }

      void syncSize()
      mountOptions.onReady?.(nextSprite)
    })

    detachSprite()

    if (disposed || token !== mountToken) {
      nextSprite.destroy()
      return null
    }

    app.stage.addChild(nextSprite as any)
    sprite = nextSprite
    await syncSize()

    return nextSprite
  }

  function dispose() {
    if (disposed) {
      return
    }

    disposed = true
    destroyModel()

    if (initialized) {
      app.destroy(true)
    }
  }

  return {
    init,
    getSprite,
    syncSize,
    mountModel,
    destroyModel,
    dispose,
  }
}
