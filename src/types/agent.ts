export type AgentType = 'claude-code' | 'copilot' | 'codex' | 'gemini' | 'opencode'

export const APP_LANGUAGE = {
  ENGLISH: 'english',
  CHINESE: 'chinese',
  JAPANESE: 'japanese',
} as const

export type AppLanguage = typeof APP_LANGUAGE[keyof typeof APP_LANGUAGE]

export const AGENT_STATE = {
  IDLE: 'idle',
  THINKING: 'thinking',
  TOOL_USE: 'tool_use',
  ERROR: 'error',
  COMPLETE: 'complete',
  NEEDS_ATTENTION: 'needs_attention',
} as const

export type TAgentState = typeof AGENT_STATE[keyof typeof AGENT_STATE]

export const SESSION_PHASE = {
  IDLE: 'idle',
  PROCESSING: 'processing',
  RUNNING_TOOL: 'running_tool',
  WAITING_ATTENTION: 'waiting_attention',
  COMPLETED: 'completed',
  ERROR: 'error',
} as const

export type SessionPhase = typeof SESSION_PHASE[keyof typeof SESSION_PHASE]

export const WINDOW_SIZE_PRESET = {
  SMALL: 'small',
  MEDIUM: 'medium',
  LARGE: 'large',
} as const

export type WindowSizePreset = typeof WINDOW_SIZE_PRESET[keyof typeof WINDOW_SIZE_PRESET]

export interface MotionGroupOption {
  id: string
  group: string
  label: string
}

export const ACTION_GROUP_BINDING_SOURCE = {
  MANUAL: 'manual',
  AUTO: 'auto',
  UNRESOLVED: 'unresolved',
} as const

export type ActionGroupBindingSource =
  typeof ACTION_GROUP_BINDING_SOURCE[keyof typeof ACTION_GROUP_BINDING_SOURCE]

export interface ResolvedActionGroupBinding {
  source: ActionGroupBindingSource
  group: string | null
}

export interface AppSettings {
  name: string
  language: AppLanguage
  autoStart: boolean
  modelDirectory: string | null
  windowSize: WindowSizePreset
  actionGroupBindings: Record<TAgentState, string | null>
}

export interface ModelScanResult {
  modelEntryFile: string
  availableMotionGroups: MotionGroupOption[]
  validationPassed: boolean
  validationMessage?: string
}

export interface ImportedModelResult {
  importedModelDirectory: string
  modelScan: ModelScanResult
}

export interface AppBootstrap {
  settings: AppSettings
  modelScan: ModelScanResult
  modelUrl: string
  mainWindowVisible: boolean
  appVersion: string
}

export interface WindowVisibilityPayload {
  visible: boolean
}

export const AGENT_STATE_ORDER = [
  AGENT_STATE.IDLE,
  AGENT_STATE.THINKING,
  AGENT_STATE.TOOL_USE,
  AGENT_STATE.ERROR,
  AGENT_STATE.COMPLETE,
  AGENT_STATE.NEEDS_ATTENTION,
] as const

export const AGENT_STATE_LABEL: Record<TAgentState, string> = {
  [AGENT_STATE.IDLE]: 'Idle / 空闲',
  [AGENT_STATE.THINKING]: 'Thinking / 思考中',
  [AGENT_STATE.TOOL_USE]: 'Tool Use / 工具调用',
  [AGENT_STATE.ERROR]: 'Error / 出错',
  [AGENT_STATE.COMPLETE]: 'Complete / 完成',
  [AGENT_STATE.NEEDS_ATTENTION]: 'Needs Attention / 需要关注',
}

export function createEmptyActionGroupBindings(): Record<TAgentState, string | null> {
  return {
    [AGENT_STATE.IDLE]: null,
    [AGENT_STATE.THINKING]: null,
    [AGENT_STATE.TOOL_USE]: null,
    [AGENT_STATE.ERROR]: null,
    [AGENT_STATE.COMPLETE]: null,
    [AGENT_STATE.NEEDS_ATTENTION]: null,
  }
}

const AUTO_MATCH_GROUP_NAMES: Record<TAgentState, string[]> = {
  [AGENT_STATE.IDLE]: ['idle'],
  [AGENT_STATE.THINKING]: ['thinking'],
  [AGENT_STATE.TOOL_USE]: ['tooluse'],
  [AGENT_STATE.ERROR]: ['error'],
  [AGENT_STATE.COMPLETE]: ['complete', 'completed'],
  [AGENT_STATE.NEEDS_ATTENTION]: ['needsattention'],
}

function normalizeMotionGroupName(value: string) {
  return value.trim().toLowerCase().replace(/[^a-z0-9]+/g, '')
}

function motionGroupName(item: MotionGroupOption | string) {
  return typeof item === 'string' ? item : item.group
}

export function detectAutoActionGroup(
  state: TAgentState,
  motionGroups: readonly (MotionGroupOption | string)[],
) {
  const aliases = new Set(AUTO_MATCH_GROUP_NAMES[state].map(normalizeMotionGroupName))

  for (const item of motionGroups) {
    const group = motionGroupName(item).trim()
    if (!group) {
      continue
    }

    if (aliases.has(normalizeMotionGroupName(group))) {
      return group
    }
  }

  return null
}

export function resolveActionGroupBinding(
  state: TAgentState,
  bindings: Record<TAgentState, string | null>,
  motionGroups: readonly (MotionGroupOption | string)[],
): ResolvedActionGroupBinding {
  const manualBinding = bindings[state]?.trim()
  if (manualBinding) {
    return {
      source: ACTION_GROUP_BINDING_SOURCE.MANUAL,
      group: manualBinding,
    }
  }

  const autoBinding = detectAutoActionGroup(state, motionGroups)
  if (autoBinding) {
    return {
      source: ACTION_GROUP_BINDING_SOURCE.AUTO,
      group: autoBinding,
    }
  }

  return {
    source: ACTION_GROUP_BINDING_SOURCE.UNRESOLVED,
    group: null,
  }
}

export interface StateChangeEvent {
  state: TAgentState
  agent?: AgentType
  session_id?: string
  tool_name?: string
  summary?: string
  working_directory?: string
  session_title?: string
  needs_attention?: boolean
  server_port?: number
}

export interface NavigatorStatus {
  current: StateChangeEvent
  server_port?: number
}

export interface NavigatorSessionInfo {
  agent: AgentType
  session_id: string
  phase: SessionPhase
  state: TAgentState
  tool_name?: string
  summary?: string
  working_directory?: string
  session_title?: string
  needs_attention?: boolean
}

export interface NavigatorSessionsPayload {
  sessions: NavigatorSessionInfo[]
  server_port?: number
}
