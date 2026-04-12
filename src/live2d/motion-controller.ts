import { Config, Priority } from 'easy-live2d'
import { AGENT_STATE, resolveActionGroupBinding } from '../types/agent'
import type { MotionGroupOption, TAgentState } from '../types/agent'

export interface CreateMotionControllerOptions {
  getSprite: () => {
    getMotions: () => Array<{ group: string }>
    startRandomMotion: (options: {
      group: string
      priority: Priority
      onFinished?: () => void
    }) => Promise<number>
  } | null
  getCurrentState: () => TAgentState
  getActionGroupBindings: () => Record<TAgentState, string | null>
  getFallbackMotionGroups: () => readonly (MotionGroupOption | string)[]
}

export function createMotionController(options: CreateMotionControllerOptions) {
  let motionPlaybackToken = 0
  let lastPlayedState: TAgentState = AGENT_STATE.IDLE

  function invalidate() {
    motionPlaybackToken += 1
    lastPlayedState = AGENT_STATE.IDLE
  }

  function getAvailableMotionGroups() {
    const sprite = options.getSprite()
    if (!sprite) {
      return options.getFallbackMotionGroups()
    }

    return sprite.getMotions().map(motion => motion.group)
  }

  function resolveMotionGroup(state: TAgentState) {
    return resolveActionGroupBinding(
      state,
      options.getActionGroupBindings(),
      getAvailableMotionGroups(),
    ).group
  }

  function syncIdleMotionGroupConfig() {
    Config.MotionGroupIdle = resolveMotionGroup(AGENT_STATE.IDLE) || 'Idle'
  }

  function hasMotionGroup(group: string) {
    return options.getSprite()?.getMotions().some(motion => motion.group === group) ?? false
  }

  async function playState(state: TAgentState, force = false) {
    syncIdleMotionGroupConfig()

    const sprite = options.getSprite()
    if (!sprite) {
      return
    }

    if (state === AGENT_STATE.IDLE) {
      lastPlayedState = AGENT_STATE.IDLE
      return
    }

    if (!force && state === lastPlayedState) {
      return
    }

    const motionGroup = resolveMotionGroup(state)
    if (!motionGroup) {
      return
    }

    if (!hasMotionGroup(motionGroup)) {
      console.warn(`live2d motion group not found: ${motionGroup}`)
      return
    }

    try {
      const token = ++motionPlaybackToken
      const handle = await sprite.startRandomMotion({
        group: motionGroup,
        priority: Priority.Force,
        onFinished: () => {
          if (token !== motionPlaybackToken || options.getSprite() !== sprite) {
            return
          }

          if (options.getCurrentState() !== state) {
            return
          }

          if (resolveMotionGroup(state) !== motionGroup) {
            return
          }

          // After one-off non-idle motions finish, let the configured idle group take over.
          lastPlayedState = AGENT_STATE.IDLE
          syncIdleMotionGroupConfig()
        },
      })

      if (handle === -1) {
        console.warn(`failed to start live2d motion group: ${motionGroup}`)
        return
      }

      lastPlayedState = state
    }
    catch (error) {
      console.warn('failed to start live2d motion', error)
    }
  }

  async function playCurrentState(force = false) {
    await playState(options.getCurrentState(), force)
  }

  return {
    invalidate,
    playState,
    playCurrentState,
    resolveMotionGroup,
    syncIdleMotionGroupConfig,
  }
}
