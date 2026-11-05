<script setup lang="ts">
import type { ToastMessage } from "../types";

defineProps<{ toasts: ToastMessage[] }>();
</script>

<template>
  <div class="toast-stack" aria-live="polite">
    <div v-for="toast in toasts" :key="toast.id" class="toast" :class="toast.tone">
      <span class="toast-icon">
        <svg v-if="toast.tone === 'danger'" viewBox="0 0 14 14" width="12" height="12" aria-hidden="true">
          <path d="M7 3.4v4.2M7 10.3v.3" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
        </svg>
        <svg v-else-if="toast.tone === 'info'" viewBox="0 0 14 14" width="12" height="12" aria-hidden="true">
          <path d="M7 6.4v3.4M7 3.9v.3" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
        </svg>
        <svg v-else viewBox="0 0 14 14" width="12" height="12" aria-hidden="true">
          <path d="M2.6 7.4 5.5 10.2 11.4 4" stroke="currentColor" stroke-width="1.9" fill="none" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </span>
      <span class="toast-text">{{ toast.text }}</span>
    </div>
  </div>
</template>

<style scoped>
.toast-stack {
  position: fixed;
  left: 50%;
  bottom: 24px;
  z-index: 60;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  transform: translateX(-50%);
  pointer-events: none;
}

.toast {
  display: flex;
  align-items: center;
  gap: 9px;
  max-width: 360px;
  padding: 10px 14px;
  border-radius: 10px;
  border: 1px solid var(--line);
  background: rgba(255, 255, 255, 0.96);
  box-shadow: 0 12px 32px -12px rgba(16, 24, 40, 0.28);
  font-size: 12.5px;
  color: var(--text);
  animation: toast-in 0.22s ease;
  backdrop-filter: blur(6px);
}

.toast-icon {
  flex: none;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  color: #fff;
  background: var(--success);
}

.toast.info .toast-icon {
  background: var(--accent);
}

.toast.danger .toast-icon {
  background: var(--danger);
}

.toast-text {
  line-height: 1.4;
}

@keyframes toast-in {
  from {
    opacity: 0;
    transform: translateY(8px) scale(0.98);
  }
}
</style>
