import { defineStore } from 'pinia'
import { ref, computed, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { AppSettings, FileItem, CompressResult } from '@/types'

export const useAppStore = defineStore('app', () => {
  const settings = ref<AppSettings>({
    apiKey: '',
    notifyMode: 'notification',
    outputMode: 'alongside',
    outputDirectory: '',
    contextMenuEnabled: false,
    theme: 'auto',
  })

  const files = ref<FileItem[]>([])
  const isCompressing = ref(false)

  const totalFiles = computed(() => files.value.length)
  const doneFiles = computed(() => files.value.filter(f => f.status === 'done').length)
  const errorFiles = computed(() => files.value.filter(f => f.status === 'error').length)
  const totalSaved = computed(() => {
    return files.value
      .filter(f => f.status === 'done')
      .reduce((sum, f) => sum + (f.originalSize - f.compressedSize), 0)
  })

  async function loadSettings() {
    try {
      const saved = await invoke<AppSettings>('load_settings')
      settings.value = { ...settings.value, ...saved }
    } catch (e) {
      console.error('加载设置失败:', e)
    }
  }

  async function saveSettings() {
    try {
      await invoke('save_settings', { settings: settings.value })
    } catch (e) {
      console.error('保存设置失败:', e)
      throw e
    }
  }

  function addFiles(paths: string[]) {
    const imageExts = ['.png', '.jpg', '.jpeg', '.webp']
    for (const path of paths) {
      const lower = path.toLowerCase()
      if (!imageExts.some(ext => lower.endsWith(ext))) continue
      if (files.value.some(f => f.path === path)) continue

      const name = path.split(/[\\/]/).pop() ?? path
      files.value.push({
        id: `${Date.now()}-${Math.random()}`,
        path,
        name,
        originalSize: 0,
        compressedSize: 0,
        status: 'pending',
      })
    }
  }

  function clearFiles() {
    if (isCompressing.value) return
    files.value = []
  }

  function removeFile(id: string) {
    if (isCompressing.value) return
    files.value = files.value.filter(f => f.id !== id)
  }

  async function compressAll() {
    if (!settings.value.apiKey) {
      throw new Error('请先配置 API Key')
    }

    const pending = files.value.filter(f => f.status === 'pending' || f.status === 'error')
    if (pending.length === 0) return

    isCompressing.value = true

    // 标记所有待压缩文件，让 UI 立即更新
    pending.forEach(f => { f.status = 'compressing' })
    await nextTick()

    // 并发队列：最多同时 3 个请求，受 TinyPNG 并发限制和带宽综合影响
    const CONCURRENCY = 3
    const queue = [...pending]
    const currentSettings = { ...settings.value }

    async function processOne(file: FileItem) {
      try {
        const result = await invoke<CompressResult>('compress_image', {
          filePath: file.path,
          settings: currentSettings,
        })
        file.originalSize = result.input_size
        file.compressedSize = result.output_size
        file.outputPath = result.output_path
        file.status = 'done'
      } catch (e) {
        file.status = 'error'
        file.errorMessage = String(e)
      }
    }

    async function worker() {
      while (queue.length > 0) {
        const file = queue.shift()!
        await processOne(file)
      }
    }

    // 启动 N 个 worker，各自从队列取任务，自然形成并发控制
    await Promise.all(
      Array.from({ length: Math.min(CONCURRENCY, pending.length) }, worker)
    )

    isCompressing.value = false

    const done = files.value.filter(f => f.status === 'done').length
    const errors = files.value.filter(f => f.status === 'error').length
    try {
      await invoke('notify_result', {
        settings: settings.value,
        successCount: done,
        errorCount: errors,
      })
    } catch (e) {
      console.error('通知失败:', e)
    }
  }

  return {
    settings,
    files,
    isCompressing,
    totalFiles,
    doneFiles,
    errorFiles,
    totalSaved,
    loadSettings,
    saveSettings,
    addFiles,
    clearFiles,
    removeFile,
    compressAll,
  }
})
