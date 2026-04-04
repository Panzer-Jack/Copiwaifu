import { ref } from 'vue'

const MAX_TEXT_LENGTH = 100
const TYPING_SPEED = 60
const DEFAULT_DURATION = 3000

export function useSpeechBubble() {
  const isVisible = ref(false)
  const displayedText = ref('')

  let fullText = ''
  let charIndex = 0
  let typingTimer: ReturnType<typeof setInterval> | null = null
  let hideTimer: ReturnType<typeof setTimeout> | null = null

  function clearTimers() {
    if (typingTimer) {
      clearInterval(typingTimer)
      typingTimer = null
    }
    if (hideTimer) {
      clearTimeout(hideTimer)
      hideTimer = null
    }
  }

  function hide() {
    clearTimers()
    isVisible.value = false
    displayedText.value = ''
  }

  function say(text: string, duration = DEFAULT_DURATION) {
    clearTimers()

    fullText = text.length > MAX_TEXT_LENGTH
      ? `${text.slice(0, MAX_TEXT_LENGTH)}…`
      : text
    charIndex = 0
    displayedText.value = ''
    isVisible.value = true

    typingTimer = setInterval(() => {
      if (charIndex < fullText.length) {
        charIndex++
        displayedText.value = fullText.slice(0, charIndex)
      }
      else {
        clearInterval(typingTimer!)
        typingTimer = null
        hideTimer = setTimeout(hide, duration)
      }
    }, TYPING_SPEED)
  }

  return { isVisible, displayedText, say, hide }
}
