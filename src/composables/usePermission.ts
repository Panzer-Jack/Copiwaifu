import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { onMounted, onUnmounted, ref, watch } from 'vue'
import type { NavigatorStatus, PermissionRequest, PermissionResolvedEvent } from '../types/agent'

const AUTO_DENY_MS = 30000

export function usePermission() {
  const pendingRequest = ref<PermissionRequest | null>(null)
  const isSubmitting = ref(false)

  let unlistenRequest: UnlistenFn | null = null
  let unlistenResolved: UnlistenFn | null = null
  let timeoutId: ReturnType<typeof setTimeout> | null = null

  async function resolve(approved: boolean) {
    if (!pendingRequest.value || isSubmitting.value) {
      return
    }

    const request = pendingRequest.value
    isSubmitting.value = true

    try {
      await invoke('respond_permission', {
        permissionId: request.permission_id,
        approved,
      })
    }
    finally {
      isSubmitting.value = false
    }
  }

  function approve() {
    void resolve(true)
  }

  function deny() {
    void resolve(false)
  }

  function clearTimeoutIfNeeded() {
    if (timeoutId) {
      clearTimeout(timeoutId)
      timeoutId = null
    }
  }

  onMounted(async () => {
    unlistenRequest = await listen<PermissionRequest>('permission:request', (event) => {
      pendingRequest.value = event.payload
    })

    unlistenResolved = await listen<PermissionResolvedEvent>('permission:resolved', (event) => {
      if (pendingRequest.value?.permission_id === event.payload.permission_id) {
        pendingRequest.value = null
      }
    })

    try {
      const status = await invoke<NavigatorStatus>('get_agent_status')
      pendingRequest.value = status.active_permission ?? null
    }
    catch (error) {
      console.warn('failed to fetch permission status', error)
    }
  })

  onUnmounted(() => {
    clearTimeoutIfNeeded()
    if (unlistenRequest) {
      void unlistenRequest()
    }
    if (unlistenResolved) {
      void unlistenResolved()
    }
  })

  watch(pendingRequest, (request) => {
    clearTimeoutIfNeeded()
    if (!request) {
      return
    }

    timeoutId = setTimeout(() => {
      void resolve(false)
    }, AUTO_DENY_MS)
  }, { immediate: true })

  return {
    pendingRequest,
    isSubmitting,
    approve,
    deny,
  }
}
