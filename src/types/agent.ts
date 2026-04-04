export type AgentType = 'claude-code' | 'copilot' | 'codex'

export const AGENT_STATE = {
  IDLE: 'idle',
  THINKING: 'thinking',
  TOOL_USE: 'tool_use',
  ERROR: 'error',
} as const

export type TAgentState = typeof AGENT_STATE[keyof typeof AGENT_STATE]

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

export interface AppSettings {
  name: string
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

export interface AppBootstrap {
  settings: AppSettings
  modelScan: ModelScanResult
  modelUrl: string
  mainWindowVisible: boolean
}

export interface WindowVisibilityPayload {
  visible: boolean
}

export const AGENT_STATE_ORDER = [
  AGENT_STATE.IDLE,
  AGENT_STATE.THINKING,
  AGENT_STATE.TOOL_USE,
  AGENT_STATE.ERROR,
] as const

export const AGENT_STATE_LABEL: Record<TAgentState, string> = {
  [AGENT_STATE.IDLE]: 'Idle',
  [AGENT_STATE.THINKING]: 'Thinking',
  [AGENT_STATE.TOOL_USE]: 'Tool Use',
  [AGENT_STATE.ERROR]: 'Error',
}

export function createEmptyActionGroupBindings(): Record<TAgentState, string | null> {
  return {
    [AGENT_STATE.IDLE]: null,
    [AGENT_STATE.THINKING]: null,
    [AGENT_STATE.TOOL_USE]: null,
    [AGENT_STATE.ERROR]: null,
  }
}

export interface StateChangeEvent {
  state: TAgentState
  agent?: AgentType
  session_id?: string
  tool_name?: string
  summary?: string
  server_port?: number
}

export interface NavigatorStatus {
  current: StateChangeEvent
  server_port?: number
}
