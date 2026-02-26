<template>
  <div class="settings-panel">
    <div class="settings-header">
      <h2>设置</h2>
      <button class="icon-btn" @click="emit('close')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </button>
    </div>

    <div class="settings-body">

      <!-- 外观主题 -->
      <section class="settings-section">
        <h3 class="section-title">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="4"/>
            <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41"/>
          </svg>
          外观主题
        </h3>
        <div class="radio-group theme-group">
          <label class="radio-card" :class="{ selected: local.theme === 'auto' }">
            <input type="radio" value="auto" v-model="local.theme" />
            <div class="radio-content">
              <span class="radio-icon theme-icon auto">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="12" cy="12" r="4"/>
                  <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41"/>
                </svg>
              </span>
              <div>
                <span class="radio-label">跟随系统</span>
                <span class="radio-desc">自动匹配系统外观设置</span>
              </div>
            </div>
          </label>
          <label class="radio-card" :class="{ selected: local.theme === 'light' }">
            <input type="radio" value="light" v-model="local.theme" />
            <div class="radio-content">
              <span class="radio-icon theme-icon light">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="12" cy="12" r="5"/>
                  <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/>
                </svg>
              </span>
              <div>
                <span class="radio-label">亮色</span>
                <span class="radio-desc">始终使用明亮界面</span>
              </div>
            </div>
          </label>
          <label class="radio-card" :class="{ selected: local.theme === 'dark' }">
            <input type="radio" value="dark" v-model="local.theme" />
            <div class="radio-content">
              <span class="radio-icon theme-icon dark">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
                </svg>
              </span>
              <div>
                <span class="radio-label">暗色</span>
                <span class="radio-desc">始终使用深色界面</span>
              </div>
            </div>
          </label>
        </div>
      </section>

      <!-- API Key -->
      <section class="settings-section">
        <h3 class="section-title">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 1 1-7.778 7.778 5.5 5.5 0 0 1 7.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4" />
          </svg>
          TinyPNG API Key
        </h3>
        <div class="api-key-input-wrapper">
          <input
            v-model="local.apiKey"
            :type="showKey ? 'text' : 'password'"
            placeholder="输入您的 TinyPNG API Key"
            class="text-input"
          />
          <button class="toggle-key-btn" @click="showKey = !showKey">
            <svg v-if="!showKey" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
              <circle cx="12" cy="12" r="3" />
            </svg>
            <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24" />
              <line x1="1" y1="1" x2="23" y2="23" />
            </svg>
          </button>
        </div>
        <p class="hint">
          前往
          <a href="https://tinypng.com/developers" target="_blank">tinypng.com/developers</a>
          获取 API Key（免费版每月 500 张）
        </p>
      </section>

      <!-- 压缩成功通知方式 -->
      <section class="settings-section">
        <h3 class="section-title">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9" />
            <path d="M13.73 21a2 2 0 0 1-3.46 0" />
          </svg>
          压缩完成通知
        </h3>
        <div class="radio-group">
          <label class="radio-card" :class="{ selected: local.notifyMode === 'dialog' }">
            <input type="radio" value="dialog" v-model="local.notifyMode" />
            <div class="radio-content">
              <span class="radio-icon">💬</span>
              <div>
                <span class="radio-label">弹窗提示</span>
                <span class="radio-desc">在应用内显示对话框</span>
              </div>
            </div>
          </label>
          <label class="radio-card" :class="{ selected: local.notifyMode === 'notification' }">
            <input type="radio" value="notification" v-model="local.notifyMode" />
            <div class="radio-content">
              <span class="radio-icon">🔔</span>
              <div>
                <span class="radio-label">系统通知</span>
                <span class="radio-desc">使用系统原生通知</span>
              </div>
            </div>
          </label>
          <label class="radio-card" :class="{ selected: local.notifyMode === 'silent' }">
            <input type="radio" value="silent" v-model="local.notifyMode" />
            <div class="radio-content">
              <span class="radio-icon">🔇</span>
              <div>
                <span class="radio-label">静默</span>
                <span class="radio-desc">不显示任何通知</span>
              </div>
            </div>
          </label>
        </div>
      </section>

      <!-- 输出方式 -->
      <section class="settings-section">
        <h3 class="section-title">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
          </svg>
          压缩后文件输出
        </h3>
        <div class="radio-group">
          <label class="radio-card" :class="{ selected: local.outputMode === 'alongside' }">
            <input type="radio" value="alongside" v-model="local.outputMode" />
            <div class="radio-content">
              <span class="radio-icon">📁</span>
              <div>
                <span class="radio-label">原路径输出</span>
                <span class="radio-desc">生成 filename-tiny.jpg，不覆盖原文件</span>
              </div>
            </div>
          </label>
          <label class="radio-card" :class="{ selected: local.outputMode === 'overwrite' }">
            <input type="radio" value="overwrite" v-model="local.outputMode" />
            <div class="radio-content">
              <span class="radio-icon">♻️</span>
              <div>
                <span class="radio-label">覆盖原图</span>
                <span class="radio-desc">直接替换原始文件</span>
              </div>
            </div>
          </label>
          <label class="radio-card" :class="{ selected: local.outputMode === 'directory' }">
            <input type="radio" value="directory" v-model="local.outputMode" />
            <div class="radio-content">
              <span class="radio-icon">📂</span>
              <div>
                <span class="radio-label">指定目录</span>
                <span class="radio-desc">输出到指定文件夹</span>
              </div>
            </div>
          </label>
        </div>

        <div v-if="local.outputMode === 'directory'" class="dir-picker">
          <input
            :value="local.outputDirectory || '未选择目录'"
            readonly
            class="text-input"
            :class="{ placeholder: !local.outputDirectory }"
          />
          <button class="action-btn secondary" @click="pickDirectory">浏览</button>
        </div>
      </section>

      <!-- 右键菜单集成 -->
      <section class="settings-section">
        <h3 class="section-title">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="2" y="3" width="20" height="14" rx="2" ry="2" />
            <line x1="8" y1="21" x2="16" y2="21" />
            <line x1="12" y1="17" x2="12" y2="21" />
          </svg>
          右键菜单集成
        </h3>
        <div class="toggle-row">
          <div>
            <span class="toggle-label">启用右键菜单 "用TinyImage压缩"</span>
            <p class="hint" v-html="contextMenuHint"></p>
          </div>
          <button
            class="toggle-switch"
            :class="{ on: local.contextMenuEnabled }"
            @click="local.contextMenuEnabled = !local.contextMenuEnabled"
          >
            <span class="toggle-thumb" />
          </button>
        </div>
      </section>
    </div>

    <div class="settings-footer">
      <button class="action-btn secondary" @click="emit('close')">取消</button>
      <button
        class="action-btn primary"
        @click="handleSave"
        :disabled="saving || (local.outputMode === 'directory' && !local.outputDirectory.trim())"
        :title="(local.outputMode === 'directory' && !local.outputDirectory.trim()) ? '请先选择输出目录' : ''"
      >
        {{ saving ? '保存中...' : '保存设置' }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, watch, computed } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { platform } from '@tauri-apps/plugin-os'
import { useAppStore } from '@/stores/app'
import { useTheme } from '@/composables/useTheme'
import type { AppSettings } from '@/types'

const emit = defineEmits<{ close: [] }>()
const store = useAppStore()
const { setTheme } = useTheme()
const showKey = ref(false)
const saving = ref(false)
const currentPlatform = platform()

const local = reactive<AppSettings>({ ...store.settings })

watch(() => store.settings, (s) => Object.assign(local, s), { deep: true })

// 主题选项变化时立即预览，无需等保存
watch(() => local.theme, (t) => setTheme(t))

// 根据平台显示不同的右键菜单提示
const contextMenuHint = computed(() => {
  if (currentPlatform === 'macos') {
    return '启用后，在 Finder 中右键图片，选择<strong>服务</strong> → <strong>用TinyImage压缩</strong>。'
  } else if (currentPlatform === 'windows') {
    return '启用后，在资源管理器中右键图片，直接选择<strong>用TinyImage压缩</strong>。'
  }
  return '在文件管理器中右键图片即可使用。'
})

async function pickDirectory() {
  const dir = await open({ directory: true, multiple: false })
  if (dir && typeof dir === 'string') {
    local.outputDirectory = dir
  }
}

async function handleSave() {
  if (local.outputMode === 'directory' && !local.outputDirectory.trim()) {
    alert('请先选择输出目录')
    return
  }

  saving.value = true
  try {
    const contextMenuChanged = local.contextMenuEnabled !== store.settings.contextMenuEnabled

    Object.assign(store.settings, local)
    setTheme(local.theme)
    await store.saveSettings()

    // 仅在右键菜单开关状态发生变化时才注册/注销（Windows 注销需重启资源管理器）
    if (contextMenuChanged) {
      try {
        if (local.contextMenuEnabled) {
          await invoke('register_context_menu')
        } else {
          await invoke('unregister_context_menu')
        }
      } catch (e) {
        console.warn('右键菜单注册失败:', e)
      }
    }

    emit('close')
  } catch (e) {
    alert('保存失败: ' + String(e))
  } finally {
    saving.value = false
  }
}
</script>

<style scoped>
.settings-panel {
  position: absolute;
  top: 0;
  right: 0;
  bottom: 0;
  width: 420px;
  background: var(--bg-card);
  border-left: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  z-index: 100;
  box-shadow: -8px 0 32px rgba(0, 0, 0, 0.4);
}

.settings-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.settings-header h2 {
  font-size: 16px;
  font-weight: 600;
}

.settings-body {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 28px;
}

.settings-body::-webkit-scrollbar {
  width: 4px;
}

.settings-body::-webkit-scrollbar-thumb {
  background: var(--border);
  border-radius: 2px;
}

.settings-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.section-title svg {
  width: 15px;
  height: 15px;
  color: var(--accent);
}

.api-key-input-wrapper {
  position: relative;
}

.text-input {
  width: 100%;
  padding: 10px 40px 10px 12px;
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--text);
  font-size: 13px;
  outline: none;
  transition: border-color 0.2s;
}

.text-input:focus {
  border-color: var(--accent);
}

.text-input.placeholder {
  color: var(--text-muted);
}

.toggle-key-btn {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  display: flex;
  align-items: center;
  padding: 4px;
  border-radius: 4px;
  transition: color 0.2s;
}

.toggle-key-btn:hover {
  color: var(--text);
}

.toggle-key-btn svg {
  width: 16px;
  height: 16px;
}

.hint {
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.5;
}

.hint a {
  color: var(--accent);
  text-decoration: none;
}

.hint a:hover {
  text-decoration: underline;
}

.radio-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.theme-group {
  flex-direction: row;
  gap: 8px;
}

.theme-group .radio-card {
  flex: 1;
  flex-direction: column;
  align-items: center;
  padding: 14px 10px;
}

.theme-group .radio-content {
  flex-direction: column;
  align-items: center;
  gap: 8px;
  text-align: center;
}

.theme-icon {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 0;
}

.theme-icon svg {
  width: 18px;
  height: 18px;
}

.theme-icon.auto {
  background: linear-gradient(135deg, #f2f2f7 50%, #1c1c1e 50%);
  color: #007aff;
}

.theme-icon.light {
  background: #fff9e6;
  color: #ff9500;
}

.theme-icon.dark {
  background: #1c1c2e;
  color: #a78bfa;
}

.radio-card {
  display: flex;
  align-items: center;
  padding: 12px 14px;
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all 0.2s;
}

.radio-card:hover {
  border-color: var(--accent);
}

.radio-card.selected {
  border-color: var(--accent);
  background: color-mix(in srgb, var(--accent) 8%, var(--bg));
}

.radio-card input[type="radio"] {
  display: none;
}

.radio-content {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
}

.radio-icon {
  font-size: 20px;
  flex-shrink: 0;
}

.radio-label {
  display: block;
  font-size: 13px;
  font-weight: 500;
  color: var(--text);
}

.radio-desc {
  display: block;
  font-size: 12px;
  color: var(--text-muted);
  margin-top: 2px;
}

.dir-picker {
  display: flex;
  gap: 8px;
  align-items: center;
}

.dir-picker .text-input {
  flex: 1;
  padding-right: 12px;
}

.toggle-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 14px;
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
}

.toggle-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text);
}

.toggle-switch {
  position: relative;
  width: 44px;
  height: 24px;
  background: var(--border);
  border-radius: 12px;
  border: none;
  cursor: pointer;
  transition: background 0.2s;
  flex-shrink: 0;
}

.toggle-switch.on {
  background: var(--accent);
}

.toggle-thumb {
  position: absolute;
  top: 3px;
  left: 3px;
  width: 18px;
  height: 18px;
  background: white;
  border-radius: 50%;
  transition: transform 0.2s;
  display: block;
}

.toggle-switch.on .toggle-thumb {
  transform: translateX(20px);
}

.settings-footer {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
  padding: 16px 20px;
  border-top: 1px solid var(--border);
  flex-shrink: 0;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border-radius: var(--radius-sm);
  border: none;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.action-btn.secondary {
  background: transparent;
  color: var(--text-muted);
  border: 1px solid var(--border);
}

.action-btn.secondary:hover:not(:disabled) {
  background: var(--bg-hover);
  color: var(--text);
}

.action-btn.primary {
  background: var(--accent);
  color: #fff;
}

.action-btn.primary:hover:not(:disabled) {
  background: var(--accent-hover);
}

.icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all 0.2s;
}

.icon-btn:hover {
  background: var(--bg-hover);
  color: var(--text);
}

.icon-btn svg {
  width: 16px;
  height: 16px;
}
</style>
