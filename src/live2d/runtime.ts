import type { Live2DSprite } from 'easy-live2d'
import { Application } from 'pixi.js'
import { createLive2DModelSprite } from './model'

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

    sprite.width = width
    sprite.height = height
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
