import { CubismSetting, Live2DSprite } from 'easy-live2d'
import { Application, Ticker } from 'pixi.js'
import type { MotionGroupOption } from '../types/agent'

export interface CreateLive2DModelSpriteOptions {
  modelEntryUrl: string
}

function toAbsoluteModelEntryUrl(modelEntryUrl: string) {
  return new URL(modelEntryUrl, window.location.href).toString()
}

function buildMotionGroupOptions(groups: string[]): MotionGroupOption[] {
  const seen = new Set<string>()
  const options: MotionGroupOption[] = []

  for (const group of groups) {
    const trimmedGroup = group.trim()
    if (!trimmedGroup || seen.has(trimmedGroup)) {
      continue
    }

    seen.add(trimmedGroup)
    options.push({
      id: trimmedGroup,
      group: trimmedGroup,
      label: trimmedGroup,
    })
  }

  return options
}

export async function createLive2DModelSprite(
  options: CreateLive2DModelSpriteOptions,
) {
  const modelEntryUrl = toAbsoluteModelEntryUrl(options.modelEntryUrl)
  const response = await fetch(modelEntryUrl)

  if (!response.ok) {
    throw new Error(`failed_to_read_model_json: ${response.status}`)
  }

  const modelJSON = await response.json()
  const modelSetting = new CubismSetting({ modelJSON })

  modelSetting.redirectPath(({ file }) => {
    return new URL(file, modelEntryUrl).toString()
  })

  return new Live2DSprite({
    modelSetting,
    ticker: Ticker.shared,
  })
}

export async function readAvailableMotionGroups(options: CreateLive2DModelSpriteOptions) {
  const app = new Application()
  let sprite: Live2DSprite | null = null

  await app.init({
    canvas: document.createElement('canvas'),
    width: 1,
    height: 1,
    resolution: 1,
    autoDensity: true,
    backgroundAlpha: 0,
  })

  try {
    sprite = await createLive2DModelSprite(options)
    app.stage.addChild(sprite as any)
    await sprite.ready
    return buildMotionGroupOptions(
      sprite.getMotions().map(motion => motion.group),
    )
  }
  finally {
    if (sprite) {
      app.stage.removeChild(sprite as any)
      sprite.destroy()
    }

    app.destroy(true)
  }
}
