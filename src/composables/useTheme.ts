import { ref, watch } from 'vue'
import type { Theme } from '@/types'

const theme = ref<Theme>('auto')
const systemDark = ref(window.matchMedia('(prefers-color-scheme: dark)').matches)

function applyTheme() {
  const isDark =
    theme.value === 'dark' ||
    (theme.value === 'auto' && systemDark.value)
  document.documentElement.setAttribute('data-theme', isDark ? 'dark' : 'light')
}

// 监听系统主题变化
const mq = window.matchMedia('(prefers-color-scheme: dark)')
const onSystemChange = (e: MediaQueryListEvent) => {
  systemDark.value = e.matches
}
mq.addEventListener('change', onSystemChange)

watch([theme, systemDark], applyTheme, { immediate: true })

export function useTheme() {
  function setTheme(t: Theme) {
    theme.value = t
  }

  function cycleTheme() {
    const order: Theme[] = ['auto', 'light', 'dark']
    const next = order[(order.indexOf(theme.value) + 1) % order.length]
    theme.value = next
  }

  return { theme, systemDark, setTheme, cycleTheme }
}
