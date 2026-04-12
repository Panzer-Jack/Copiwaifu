import { ref } from 'vue'

export interface UseContextMenuOptions {
  width: number
  height: number
  edgeGap: number
}

export function useContextMenu(options: UseContextMenuOptions) {
  const menuState = ref({
    visible: false,
    x: 0,
    y: 0,
  })

  function closeMenu() {
    menuState.value.visible = false
  }

  function openMenu(event: MouseEvent) {
    event.preventDefault()

    const maxX = Math.max(
      options.edgeGap,
      window.innerWidth - options.width - options.edgeGap,
    )
    const maxY = Math.max(
      options.edgeGap,
      window.innerHeight - options.height - options.edgeGap,
    )

    menuState.value = {
      visible: true,
      x: Math.min(Math.max(event.clientX, options.edgeGap), maxX),
      y: Math.min(Math.max(event.clientY, options.edgeGap), maxY),
    }
  }

  return {
    menuState,
    closeMenu,
    openMenu,
  }
}
