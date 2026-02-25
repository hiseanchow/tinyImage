<template>
  <div
    class="drop-zone"
    :class="{ 'drag-over': isDragOver, 'has-files': hasFiles }"
    @click="openFilePicker"
    @dragover.prevent="onDragOver"
    @dragleave="onDragLeave"
    @drop.prevent="onDrop"
  >
    <div v-if="!hasFiles" class="drop-content">
      <div class="drop-icon">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
          <polyline points="17 8 12 3 7 8" />
          <line x1="12" y1="3" x2="12" y2="15" />
        </svg>
      </div>
      <p class="drop-text">拖拽图片到此处</p>
      <p class="drop-hint">或点击选择文件 · 支持 PNG、JPG、WebP</p>
    </div>

    <div v-else class="drop-mini">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="mini-icon">
        <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
        <polyline points="17 8 12 3 7 8" />
        <line x1="12" y1="3" x2="12" y2="15" />
      </svg>
      <span>继续添加图片</span>
    </div>

    <div v-if="isDragOver" class="drag-overlay">
      <div class="drag-overlay-content">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
          <polyline points="17 8 12 3 7 8" />
          <line x1="12" y1="3" x2="12" y2="15" />
        </svg>
        <span>松开鼠标添加图片</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { useAppStore } from '@/stores/app'

const store = useAppStore()
const isDragOver = ref(false)
const hasFiles = computed(() => store.files.length > 0)

// Tauri v2 原生拖拽事件（可获取真实文件路径）
let unlistenDrop: (() => void) | null = null

onMounted(async () => {
  const appWindow = getCurrentWebviewWindow()
  unlistenDrop = await appWindow.onDragDropEvent((event) => {
    if (event.payload.type === 'over') {
      isDragOver.value = true
    } else if (event.payload.type === 'leave') {
      isDragOver.value = false
    } else if (event.payload.type === 'drop') {
      isDragOver.value = false
      const paths = event.payload.paths ?? []
      if (paths.length > 0) {
        store.addFiles(paths)
      }
    }
  })
})

onUnmounted(() => {
  unlistenDrop?.()
})

// 保留标准 HTML drag 事件以支持浏览器预览模式的视觉反馈
function onDragOver() {
  isDragOver.value = true
}

function onDragLeave() {
  isDragOver.value = false
}

function onDrop(e: DragEvent) {
  // Tauri 环境由 onDragDropEvent 处理；浏览器 fallback 处理
  isDragOver.value = false
  const paths: string[] = []
  if (e.dataTransfer?.files) {
    for (const file of e.dataTransfer.files) {
      const path = (file as unknown as { path?: string }).path
      if (path) paths.push(path)
    }
  }
  if (paths.length > 0) {
    store.addFiles(paths)
  }
}

async function openFilePicker() {
  if (store.isCompressing) return
  try {
    const selected = await open({
      multiple: true,
      filters: [{ name: '图片', extensions: ['png', 'jpg', 'jpeg', 'webp'] }],
    })
    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected]
      store.addFiles(paths)
    }
  } catch (e) {
    console.error('打开文件选择器失败:', e)
  }
}
</script>

<style scoped>
.drop-zone {
  position: relative;
  border: 2px dashed var(--border);
  border-radius: var(--radius);
  background: var(--bg-card);
  cursor: pointer;
  transition: all 0.2s;
  flex-shrink: 0;
}

.drop-zone:not(.has-files) {
  padding: 48px 24px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.drop-zone.has-files {
  padding: 12px 20px;
}

.drop-zone:hover,
.drop-zone.drag-over {
  border-color: var(--accent);
  background: color-mix(in srgb, var(--accent) 5%, var(--bg-card));
}

.drop-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.drop-icon svg {
  width: 48px;
  height: 48px;
  color: var(--text-muted);
}

.drag-over .drop-icon svg {
  color: var(--accent);
}

.drop-text {
  font-size: 16px;
  font-weight: 500;
  color: var(--text);
}

.drop-hint {
  font-size: 13px;
  color: var(--text-muted);
}

.drop-mini {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text-muted);
  font-size: 13px;
  transition: color 0.2s;
}

.drop-zone:hover .drop-mini {
  color: var(--accent);
}

.mini-icon {
  width: 16px;
  height: 16px;
}

.drag-overlay {
  position: absolute;
  inset: 0;
  background: color-mix(in srgb, var(--accent) 10%, var(--bg));
  border-radius: calc(var(--radius) - 2px);
  display: flex;
  align-items: center;
  justify-content: center;
}

.drag-overlay-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  color: var(--accent);
}

.drag-overlay-content svg {
  width: 40px;
  height: 40px;
}

.drag-overlay-content span {
  font-size: 15px;
  font-weight: 500;
}
</style>
