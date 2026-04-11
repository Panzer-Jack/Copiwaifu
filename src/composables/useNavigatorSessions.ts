import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { onMounted, onUnmounted, ref } from 'vue'
import type { NavigatorSessionInfo, NavigatorSessionsPayload } from '../types/agent'

export function useNavigatorSessions() {
  const sessions = ref<NavigatorSessionInfo[]>([])
  const serverPort = ref<number | null>(null)
  let unlisten: UnlistenFn | null = null

  function applyPayload(payload: NavigatorSessionsPayload) {
    sessions.value = payload.sessions
    serverPort.value = payload.server_port ?? null
  }

  onMounted(async () => {
    unlisten = await listen<NavigatorSessionsPayload>('navigator:sessions-changed', (event) => {
      applyPayload(event.payload)
    })

    try {
      const payload = await invoke<NavigatorSessionsPayload>('get_navigator_sessions')
      applyPayload(payload)
    }
    catch (error) {
      console.warn('failed to fetch navigator sessions', error)
    }
  })

  onUnmounted(() => {
    if (unlisten) {
      void unlisten()
    }
  })

  return {
    sessions,
    serverPort,
  }
}
