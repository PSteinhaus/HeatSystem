<script setup lang="ts">
import { toasts } from '../store'
</script>

<template>
  <div class="toast-container">
    <transition-group name="toast">
      <div
        v-for="toast in toasts"
        :key="toast.id"
        class="toast"
        :class="toast.type"
      >
        {{ toast.message }}
      </div>
    </transition-group>
  </div>
</template>

<style scoped>
.toast-container {
  position: fixed;
  top: 1rem;
  right: 1rem;
  z-index: 200;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  pointer-events: none;
}
.toast {
  background: var(--bg-panel);
  border: 1px solid var(--border-light);
  border-radius: 10px;
  padding: 0.7em 1.1em;
  font-size: 0.9rem;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
  animation: slideIn 0.3s ease;
  max-width: 320px;
}
.toast.success {
  border-color: var(--primary);
  color: var(--primary);
}
.toast.failure {
  border-color: var(--error);
  color: var(--error);
}
.toast.info {
  border-color: var(--accent);
  color: var(--accent);
}
.toast-enter-active, .toast-leave-active {
  transition: all 0.3s ease;
}
.toast-enter-from {
  opacity: 0;
  transform: translateX(20px);
}
.toast-leave-to {
  opacity: 0;
  transform: translateX(20px);
}
</style>
