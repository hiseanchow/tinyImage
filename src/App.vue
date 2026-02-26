<template>
  <div class="app" :class="{ 'settings-open': showSettings }">
    <header class="app-header" data-tauri-drag-region>
      <div class="header-left" data-tauri-drag-region>
        <img src="./assets/logo.svg" class="logo" alt="TinyImage" draggable="false" />
        <span class="app-title" data-tauri-drag-region>TinyImage</span>
      </div>
      <div class="header-actions">
        <!-- 主题切换按钮 -->
        <button
          class="icon-btn"
          :title="themeLabel"
          @click="handleThemeCycle"
        >
          <!-- 自动（跟随系统） -->
          <svg v-if="theme === 'auto'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="4"/>
            <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41"/>
            <path d="M2 12a10 10 0 0 1 10-10" stroke-dasharray="4 2"/>
          </svg>
          <!-- 亮色 -->
          <svg v-else-if="theme === 'light'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="5"/>
            <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/>
          </svg>
          <!-- 暗色 -->
          <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
          </svg>
        </button>

        <!-- 设置按钮 -->
        <button class="icon-btn" title="设置" @click="showSettings = !showSettings">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="3" />
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
          </svg>
        </button>
      </div>
    </header>

    <main class="app-main">
      <DropZone />
      <FileList />
    </main>

    <Transition name="slide">
      <Settings v-if="showSettings" @close="showSettings = false" />
    </Transition>

    <ResultDialog />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '@/stores/app'
import { useTheme } from '@/composables/useTheme'
import DropZone from '@/components/DropZone.vue'
import FileList from '@/components/FileList.vue'
import Settings from '@/components/Settings.vue'
import ResultDialog from '@/components/ResultDialog.vue'

const showSettings = ref(false)
const store = useAppStore()
const { theme, setTheme, cycleTheme } = useTheme()

const themeLabel = computed(() => {
  return { auto: '跟随系统', light: '亮色模式', dark: '暗色模式' }[theme.value]
})

function handleThemeCycle() {
  cycleTheme()
  store.settings.theme = theme.value
  store.saveSettings().catch(console.error)
}

onMounted(async () => {
  await store.loadSettings()

  // 恢复已保存的主题
  setTheme(store.settings.theme ?? 'auto')

  // 初始化窗口：如果不是后台启动模式，Rust 会把窗口显示出来
  await invoke('init_window').catch(console.error)

  // macOS overlay 标题栏：给 html 加 class
  if (navigator.userAgent.includes('Macintosh')) {
    document.documentElement.classList.add('platform-mac')
  }

  // 检查通过「打开方式」传入的启动文件
  const startupFiles = await invoke<[string, boolean][]>('get_startup_files').catch(() => [])
  if (startupFiles.length > 0) {
    const openPaths = startupFiles.filter(item => !item[1]).map(item => item[0])
    const compressPaths = startupFiles.filter(item => item[1]).map(item => item[0])
    
    if (openPaths.length > 0) {
      store.addFiles(openPaths)
    }
    if (compressPaths.length > 0) {
      store.addFiles(compressPaths)
      store.compressAll().catch(console.error)
    }
  }

  // 监听单纯添加文件的事件 (打开方式 / 拖拽)
  await listen<string[]>('add-files', (event) => {
    store.addFiles(event.payload)
  })

  // 监听右键菜单 / deep link 触发的文件压缩事件
  await listen<string[]>('compress-files', (event) => {
    store.addFiles(event.payload)
    store.compressAll().catch(console.error)
  })

  // 监听 Rust 发来的实时压缩进度，更新对应文件的 progress 和 phase
  await listen<{ path: string; percent: number; phase: string }>('compress-progress', (event) => {
    const { path, percent, phase } = event.payload
    const file = store.files.find(f => f.path === path)
    if (file && file.status === 'compressing') {
      file.progress = percent
      file.phase = phase as any
    }
  })
})

// 设置面板保存主题时同步到 composable
watch(() => store.settings.theme, (t) => setTheme(t))
</script>

<style>
/* ── 暗色主题（默认） ─────────────────────────────────────── */
:root,
[data-theme="dark"] {
  --bg: #0f1117;
  --bg-card: #1a1d27;
  --bg-hover: #22263a;
  --border: #2a2d3e;
  --accent: #4f9cf9;
  --accent-hover: #6db3ff;
  --success: #34d399;
  --error: #f87171;
  --warning: #fbbf24;
  --text: #e2e8f0;
  --text-muted: #64748b;
  --shadow: rgba(0, 0, 0, 0.5);
  --radius: 12px;
  --radius-sm: 8px;
}

/* ── 亮色主题 ─────────────────────────────────────────────── */
[data-theme="light"] {
  --bg: #f2f2f7;
  --bg-card: #ffffff;
  --bg-hover: #e5e5ea;
  --border: #c6c6c8;
  --accent: #007aff;
  --accent-hover: #0051d5;
  --success: #28cd41;
  --error: #ff3b30;
  --warning: #ff9500;
  --text: #1c1c1e;
  --text-muted: #8e8e93;
  --shadow: rgba(0, 0, 0, 0.12);
  --radius: 12px;
  --radius-sm: 8px;
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body, #app {
  height: 100%;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

/* 主题切换过渡 */
html {
  transition: background-color 0.25s, color 0.25s;
}

.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--bg);
  color: var(--text);
  overflow: hidden;
  position: relative;
  transition: background-color 0.25s, color 0.25s;
}

.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  height: 52px;
  background: var(--bg-card);
  border-bottom: 1px solid var(--border);
  -webkit-app-region: drag;
  user-select: none;
  flex-shrink: 0;
  transition: background-color 0.25s, border-color 0.25s;
}

/* macOS overlay 标题栏：交通灯区域约 82px */
.platform-mac .app-header {
  padding-left: 82px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.logo {
  width: 26px;
  height: 26px;
}

.app-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text);
  letter-spacing: -0.01em;
}

.header-actions {
  -webkit-app-region: no-drag;
  display: flex;
  gap: 4px;
}

.icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: background 0.15s, color 0.15s;
}

.icon-btn:hover {
  background: var(--bg-hover);
  color: var(--text);
}

.icon-btn svg {
  width: 17px;
  height: 17px;
}

.app-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding: 20px;
  gap: 16px;
}

/* Settings panel slide animation */
.slide-enter-active,
.slide-leave-active {
  transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.slide-enter-from,
.slide-leave-to {
  transform: translateX(100%);
}
</style>
