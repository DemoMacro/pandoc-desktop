<template>
  <div
    v-if="showMessage"
    class="message-toast"
    :class="`message-${messageType}`"
    role="alert"
    :aria-live="messageType === 'error' ? 'assertive' : 'polite'"
  >
    <div class="message-content">
      <span class="message-icon">{{ getMessageIcon(messageType) }}</span>
      <span class="message-text">{{ message }}</span>
      <button
        @click="clearMessage"
        class="message-close"
        aria-label="Close message"
      >
        ×
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useMessages } from "../composables/useMessages";

const { message, messageType, showMessage, clearMessage } = useMessages();

const getMessageIcon = (type: string): string => {
  switch (type) {
    case "success":
      return "✅";
    case "error":
      return "❌";
    case "warning":
      return "⚠️";
    case "info":
    default:
      return "ℹ️";
  }
};
</script>

<style scoped>
.message-toast {
  position: fixed;
  top: 20px;
  right: 20px;
  max-width: 400px;
  padding: var(--pico-spacing);
  border-radius: var(--pico-border-radius);
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
  z-index: 1000;
  animation: slideIn 0.3s ease-out;
}

.message-content {
  display: flex;
  align-items: center;
  gap: var(--pico-spacing);
}

.message-text {
  flex: 1;
  font-size: var(--pico-font-size);
}

.message-close {
  background: transparent;
  border: none;
  font-size: 1.2rem;
  cursor: pointer;
  padding: 0.25rem;
  border-radius: var(--pico-border-radius);
  transition: background-color 0.2s;
}

.message-close:hover {
  background: rgba(255, 255, 255, 0.2);
}

.message-success {
  background: #10b981;
  color: white;
}

.message-error {
  background: #ef4444;
  color: white;
}

.message-warning {
  background: #f59e0b;
  color: white;
}

.message-info {
  background: #3b82f6;
  color: white;
}

@keyframes slideIn {
  from {
    transform: translateX(100%);
    opacity: 0;
  }
  to {
    transform: translateX(0);
    opacity: 1;
  }
}
</style>
