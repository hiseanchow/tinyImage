<template>
  <div class="file-list-container" v-if="store.files.length > 0">
    <div class="list-header">
      <div class="list-stats">
        <span class="stat">
          <span class="stat-num">{{ store.totalFiles }}</span> 个文件
        </span>
        <span v-if="store.doneFiles > 0" class="stat success">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <polyline points="20 6 9 17 4 12" />
          </svg>
          {{ store.doneFiles }} 成功
        </span>
        <span v-if="store.errorFiles > 0" class="stat error">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
          {{ store.errorFiles }} 失败
        </span>
        <span v-if="store.totalSaved > 0" class="stat saved">
          节省 {{ formatSize(store.totalSaved) }}
        </span>
      </div>
      <div class="list-actions">
        <button
          class="action-btn secondary"
          @click="store.clearFiles()"
          :disabled="store.isCompressing"
          title="清空列表"
        >
          清空
        </button>
        <button
          class="action-btn primary"
          @click="handleCompress"
          :disabled="store.isCompressing || !hasPending"
        >
          <span v-if="store.isCompressing" class="btn-spinner" />
          {{ store.isCompressing ? '压缩中...' : '开始压缩' }}
        </button>
      </div>
    </div>

    <div class="file-list">
      <TransitionGroup name="file-item">
        <div
          v-for="file in store.files"
          :key="file.id"
          class="file-item"
          :class="file.status"
        >
          <div class="file-icon">
            <template v-if="file.status !== 'compressing'">
              <img
                v-if="thumbCache[file.path]"
                :src="thumbCache[file.path]"
                :alt="file.name"
                class="file-thumb"
              />
              <span v-else class="fallback-icon">🖼</span>
            </template>
            <div v-else class="file-spinner" />
          </div>

          <div class="file-info">
            <span class="file-name" :title="file.path">{{ file.name }}</span>
            <div class="file-meta">
              <span v-if="file.status === 'done'" class="file-sizes">
                {{ formatSize(file.originalSize) }}
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <line x1="5" y1="12" x2="19" y2="12" />
                  <polyline points="12 5 19 12 12 19" />
                </svg>
                {{ formatSize(file.compressedSize) }}
                <span class="ratio success">-{{ calcRatio(file.originalSize, file.compressedSize) }}%</span>
              </span>
              <span v-else-if="file.status === 'error'" class="file-error" :title="file.errorMessage">
                {{ file.errorMessage }}
              </span>
              <template v-else-if="file.status === 'compressing'">
                <div class="progress-bar">
                  <div
                    class="progress-fill"
                    :style="{ width: (file.progress ?? 0) + '%' }"
                    :class="file.phase"
                  />
                </div>
                <span class="progress-label">
                  {{ phaseLabel(file.phase) }}
                  <span v-if="file.phase === 'downloading' && file.progress">
                    {{ file.progress }}%
                  </span>
                </span>
              </template>
              <span v-else class="file-status-text muted">等待中</span>
            </div>
          </div>

          <div class="file-badge">
            <span v-if="file.status === 'done'" class="badge success">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                <polyline points="20 6 9 17 4 12" />
              </svg>
            </span>
            <!-- 失败时显示重试按钮替代错误图标 -->
            <button
              v-else-if="file.status === 'error'"
              class="retry-btn"
              @click="store.retryFile(file.id)"
              title="重试"
              :disabled="store.isCompressing"
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="1 4 1 10 7 10" />
                <path d="M3.51 15a9 9 0 1 0 .49-3" />
              </svg>
              重试
            </button>
          </div>

          <button
            class="remove-btn"
            @click="store.removeFile(file.id)"
            :disabled="store.isCompressing"
            title="移除"
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>
      </TransitionGroup>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '@/stores/app'

const store = useAppStore()

// 缩略图缓存：path → base64 data URL
const thumbCache = ref<Record<string, string>>({})

// 监听文件列表变化，按需加载缩略图
watch(
  () => store.files.map(f => f.path),
  async (paths) => {
    for (const path of paths) {
      if (thumbCache.value[path]) continue
      try {
        const dataUrl = await invoke<string>('get_image_preview', { path })
        thumbCache.value[path] = dataUrl
      } catch {
        thumbCache.value[path] = ''
      }
    }
  },
  { immediate: true }
)

const hasPending = computed(() =>
  store.files.some(f => f.status === 'pending' || f.status === 'error')
)

function formatSize(bytes: number): string {
  if (bytes === 0) return '0 B'
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1024 / 1024).toFixed(2)} MB`
}

function calcRatio(original: number, compressed: number): number {
  if (original === 0) return 0
  return Math.round((1 - compressed / original) * 100)
}

function phaseLabel(phase?: string): string {
  switch (phase) {
    case 'uploading':   return '上传中...'
    case 'processing':  return 'TinyPNG 处理中...'
    case 'downloading': return '下载中'
    default:            return '压缩中...'
  }
}

async function handleCompress() {
  try {
    await store.compressAll()
  } catch (e) {
    alert(String(e))
  }
}
</script>

<style scoped>
.file-list-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: var(--bg-card);
  border-radius: var(--radius);
  border: 1px solid var(--border);
  overflow: hidden;
  min-height: 0;
}

.list-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px;
  border-bottom: 1px solid var(--border);
  flex-shrink: 0;
}

.list-stats {
  display: flex;
  align-items: center;
  gap: 14px;
  font-size: 13px;
}

.stat {
  display: flex;
  align-items: center;
  gap: 4px;
  color: var(--text-muted);
}

.stat svg {
  width: 13px;
  height: 13px;
}

.stat-num {
  font-weight: 600;
  color: var(--text);
}

.stat.success {
  color: var(--success);
}

.stat.error {
  color: var(--error);
}

.stat.saved {
  color: var(--accent);
  font-weight: 500;
}

.list-actions {
  display: flex;
  gap: 8px;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 7px 14px;
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

.btn-spinner {
  width: 12px;
  height: 12px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: #fff;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

.file-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.file-list::-webkit-scrollbar {
  width: 4px;
}

.file-list::-webkit-scrollbar-track {
  background: transparent;
}

.file-list::-webkit-scrollbar-thumb {
  background: var(--border);
  border-radius: 2px;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border-radius: var(--radius-sm);
  transition: background 0.15s;
}

.file-item:hover {
  background: var(--bg-hover);
}

.file-item:hover .remove-btn {
  opacity: 1;
}

.file-icon {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  font-size: 24px;
  border-radius: 6px;
  overflow: hidden;
  background: var(--bg-hover);
}

.file-thumb {
  width: 100%;
  height: 100%;
  object-fit: cover;
  border-radius: 6px;
}

.fallback-icon {
  font-size: 22px;
}

.file-spinner {
  width: 24px;
  height: 24px;
  border: 2px solid var(--border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.7s linear infinite;
}

.file-info {
  flex: 1;
  min-width: 0;
}

.file-name {
  display: block;
  font-size: 13px;
  font-weight: 500;
  color: var(--text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-meta {
  margin-top: 3px;
  font-size: 12px;
}

.file-sizes {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--text-muted);
}

.file-sizes svg {
  width: 12px;
  height: 12px;
}

.ratio.success {
  color: var(--success);
  font-weight: 600;
}

.file-error {
  color: var(--error);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 300px;
  display: block;
}

.file-status-text {
  color: var(--text-muted);
}

.file-status-text.muted {
  opacity: 0.6;
}

/* 进度条 */
.progress-bar {
  height: 3px;
  background: var(--border);
  border-radius: 2px;
  overflow: hidden;
  margin-bottom: 4px;
}

.progress-fill {
  height: 100%;
  border-radius: 2px;
  background: var(--accent);
  transition: width 0.3s ease;
}

/* 上传阶段：蓝色条纹动画 */
.progress-fill.uploading {
  background: repeating-linear-gradient(
    90deg,
    var(--accent) 0%,
    color-mix(in srgb, var(--accent) 60%, transparent) 50%,
    var(--accent) 100%
  );
  background-size: 200% 100%;
  animation: stripe-move 1.2s linear infinite;
}

/* 处理阶段：脉冲动画 */
.progress-fill.processing {
  background: var(--accent);
  animation: pulse-width 1.5s ease-in-out infinite;
}

@keyframes stripe-move {
  0%   { background-position: 200% 0; }
  100% { background-position: 0% 0; }
}

@keyframes pulse-width {
  0%, 100% { opacity: 1; }
  50%       { opacity: 0.5; }
}

.progress-label {
  font-size: 11px;
  color: var(--text-muted);
  display: flex;
  gap: 4px;
}

/* ── 压缩进度条 ─────────────────────────────────────────────── */
.progress-bar {
  height: 3px;
  background: var(--border);
  border-radius: 2px;
  overflow: hidden;
  margin-bottom: 3px;
}

.progress-fill {
  height: 100%;
  border-radius: 2px;
  background: var(--accent);
  transition: width 0.3s ease;
}

.progress-fill.uploading {
  background: var(--warning);
}

.progress-fill.processing {
  background: var(--accent);
  /* 处理阶段进度不确定，用动画表示 */
  animation: processing-pulse 1.2s ease-in-out infinite;
  width: 100% !important;
  opacity: 0.6;
}

.progress-fill.downloading {
  background: var(--success);
}

@keyframes processing-pulse {
  0%, 100% { opacity: 0.4; }
  50% { opacity: 1; }
}

.progress-label {
  font-size: 11px;
  color: var(--text-muted);
  display: flex;
  gap: 4px;
}

/* ── 重试按钮 ────────────────────────────────────────────────── */
.retry-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 3px 8px;
  border-radius: 6px;
  border: 1px solid color-mix(in srgb, var(--accent) 40%, transparent);
  background: color-mix(in srgb, var(--accent) 10%, transparent);
  color: var(--accent);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
  white-space: nowrap;
}

.retry-btn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--accent) 20%, transparent);
  border-color: var(--accent);
}

.retry-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.retry-btn svg {
  width: 12px;
  height: 12px;
}

.file-badge {
  flex-shrink: 0;
}

.badge {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border-radius: 50%;
}

.badge svg {
  width: 12px;
  height: 12px;
}

.badge.success {
  background: color-mix(in srgb, var(--success) 15%, transparent);
  color: var(--success);
}

.badge.error {
  background: color-mix(in srgb, var(--error) 15%, transparent);
  color: var(--error);
}

.remove-btn {
  width: 26px;
  height: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  color: var(--text-muted);
  border-radius: 6px;
  cursor: pointer;
  opacity: 0;
  transition: all 0.15s;
  flex-shrink: 0;
}

.remove-btn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--error) 15%, transparent);
  color: var(--error);
}

.remove-btn:disabled {
  cursor: not-allowed;
}

.remove-btn svg {
  width: 14px;
  height: 14px;
}

/* 重试按钮 */
.retry-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  border-radius: 6px;
  border: 1px solid color-mix(in srgb, var(--error) 40%, transparent);
  background: color-mix(in srgb, var(--error) 10%, transparent);
  color: var(--error);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
  white-space: nowrap;
  flex-shrink: 0;
}

.retry-btn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--error) 20%, transparent);
}

.retry-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.retry-btn svg {
  width: 12px;
  height: 12px;
}

/* Animations */
.file-item-enter-active,
.file-item-leave-active {
  transition: all 0.2s ease;
}

.file-item-enter-from {
  opacity: 0;
  transform: translateY(-8px);
}

.file-item-leave-to {
  opacity: 0;
  transform: translateX(20px);
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
