import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { onMounted, onUnmounted, ref } from 'vue'
import type { AgentState, AgentType, NavigatorStatus, StateChangeEvent } from '../types/agent'

export function useAgentState() {
  const currentState = ref<AgentState>('idle')
  const activeAgent = ref<AgentType | null>(null)
  const serverPort = ref<number | null>(null)
  const sessionInfo = ref<{
    sessionId: string | null
    toolName: string | null
    summary: string | null
  }>({
    sessionId: null,
    toolName: null,
    summary: null,
  })

  let unlisten: UnlistenFn | null = null

  function applyState(payload: StateChangeEvent) {
    currentState.value = payload.state
    activeAgent.value = payload.agent ?? null
    serverPort.value = payload.server_port ?? null
    sessionInfo.value = {
      sessionId: payload.session_id ?? null,
      toolName: payload.tool_name ?? null,
      summary: payload.summary ?? null,
    }
  }

  onMounted(async () => {
    unlisten = await listen<StateChangeEvent>('agent:state-change', event => applyState(event.payload))

    try {
      const status = await invoke<NavigatorStatus>('get_agent_status')
      applyState(status.current)
    }
    catch (error) {
      console.warn('failed to fetch navigator status', error)
    }
  })

  onUnmounted(() => {
    if (unlisten) {
      void unlisten()
    }
  })

  return {
    currentState,
    activeAgent,
    serverPort,
    sessionInfo,
  }
}
