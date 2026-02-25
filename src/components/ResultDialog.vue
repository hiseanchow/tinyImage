<template>
  <Transition name="fade">
    <div v-if="visible" class="dialog-overlay" @click.self="close">
      <div class="dialog">
        <div class="dialog-icon" :class="isError ? 'error' : 'success'">
          <svg v-if="!isError" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <polyline points="20 6 9 17 4 12" />
          </svg>
          <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <circle cx="12" cy="12" r="10" />
            <line x1="12" y1="8" x2="12" y2="12" />
            <line x1="12" y1="16" x2="12.01" y2="16" />
          </svg>
        </div>
        <p class="dialog-message">{{ message }}</p>
        <button class="dialog-btn" @click="close">确定</button>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { listen } from '@tauri-apps/api/event'

const visible = ref(false)
const message = ref('')
const isError = computed(() => message.value.includes('失败'))

listen<string>('show-result-dialog', (event) => {
  message.value = event.payload
  visible.value = true
})

function close() {
  visible.value = false
}
</script>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.dialog {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 32px 28px 24px;
  max-width: 320px;
  width: 90%;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
}

.dialog-icon {
  width: 56px;
  height: 56px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.dialog-icon.success {
  background: color-mix(in srgb, var(--success) 15%, transparent);
  color: var(--success);
}

.dialog-icon.error {
  background: color-mix(in srgb, var(--error) 15%, transparent);
  color: var(--error);
}

.dialog-icon svg {
  width: 28px;
  height: 28px;
}

.dialog-message {
  font-size: 14px;
  color: var(--text);
  text-align: center;
  line-height: 1.6;
}

.dialog-btn {
  padding: 8px 32px;
  background: var(--accent);
  color: #fff;
  border: none;
  border-radius: var(--radius-sm);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.2s;
}

.dialog-btn:hover {
  background: var(--accent-hover);
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
