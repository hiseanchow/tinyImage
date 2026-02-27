import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { AppSettings, FileItem, CompressResult } from '@/types'

export const useAppStore = defineStore('app', () => {
  const settings = ref<AppSettings>({
    apiKey: '',
    notifyMode: 'notification',
    outputMode: 'alongside',
    outputDirectory: '',
    contextMenuEnabled: true,
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

    // 并发队列：最多同时 3 个，剩余文件保持 pending 状态直到 worker 取到
    const CONCURRENCY = 3
    const queue = [...pending]
    const currentSettings = { ...settings.value }

    async function processOne(file: FileItem) {
      // 只有 worker 实际取到该文件时才更新为压缩中
      file.status = 'compressing'
      file.progress = 0
      file.phase = undefined
      file.errorMessage = undefined
      try {
        const result = await invoke<CompressResult>('compress_image', {
          filePath: file.path,
          settings: currentSettings,
        })
        file.originalSize = result.input_size
        file.compressedSize = result.output_size
        file.outputPath = result.output_path
        file.status = 'done'
        file.progress = 100
        file.phase = undefined
      } catch (e) {
        file.status = 'error'
        file.errorMessage = String(e)
        file.progress = undefined
        file.phase = undefined
      }
    }

    async function worker() {
      while (queue.length > 0) {
        const file = queue.shift()!
        await processOne(file)
      }
    }

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

  async function retryFile(id: string) {
    const file = files.value.find(f => f.id === id)
    if (!file || file.status !== 'error') return
    // 重置为待压缩，复用 compressAll 的并发逻辑
    file.status = 'pending'
    file.errorMessage = undefined
    file.progress = undefined
    file.phase = undefined
    await compressAll()
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
    retryFile,
  }
})
