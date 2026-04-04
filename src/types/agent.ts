export type AgentType = 'claude-code' | 'copilot' | 'codex'

export type AgentState = 'idle' | 'thinking' | 'tool_use' | 'error' | 'waiting_permission'

export interface StateChangeEvent {
  state: AgentState
  agent?: AgentType
  session_id?: string
  tool_name?: string
  summary?: string
  server_port?: number
}

export interface PermissionRequest {
  permission_id: string
  agent: AgentType
  session_id: string
  tool_name: string
  summary: string
}

export interface PermissionResolvedEvent {
  permission_id: string
  status: 'pending' | 'approved' | 'denied'
}

export interface NavigatorStatus {
  current: StateChangeEvent
  active_permission?: PermissionRequest
  server_port?: number
}
