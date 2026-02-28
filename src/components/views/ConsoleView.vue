<script setup lang="ts">
import { ref, watch, nextTick } from 'vue';
import type { Server } from '../../types';

const props = defineProps<{
  server: Server;
}>();

const emit = defineEmits<{
  command: [cmd: string];
}>();

const commandInput = ref('');
const logsContainer = ref<HTMLElement | null>(null);

function sendCommand() {
  if (!commandInput.value.trim()) return;
  emit('command', commandInput.value);
  commandInput.value = '';
}

function getLogLevel(log: string): string {
  if (/\[(ERROR|FATAL)]/i.test(log)) return 'error';
  if (/\[WARN]/i.test(log)) return 'warn';
  if (/\[DEBUG]|\[TRACE]/i.test(log)) return 'debug';
  return '';
}

watch(
  () => props.server.logs.length,
  async () => {
    await nextTick();
    if (logsContainer.value) {
      logsContainer.value.scrollTop = logsContainer.value.scrollHeight;
    }
  }
);
</script>

<template>
  <div class="console-view">
    <div ref="logsContainer" class="logs">
      <div v-if="server.logs.length === 0" class="logs-empty">
        <svg
          width="32"
          height="32"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <polyline points="4 17 10 11 4 5" />
          <line x1="12" y1="19" x2="20" y2="19" />
        </svg>
        <span>No logs yet. Start the server to see output.</span>
      </div>
      <div v-for="(log, i) in server.logs" :key="i" :class="['log-line', getLogLevel(log)]">
        {{ log }}
      </div>
    </div>

    <div class="command-bar">
      <span class="prompt">&gt;</span>
      <input
        v-model="commandInput"
        placeholder="Enter command..."
        :disabled="server.status !== 'running'"
        @keyup.enter="sendCommand"
      />
      <button class="send-btn" :disabled="server.status !== 'running'" @click="sendCommand">
        <svg
          width="15"
          height="15"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M5 12h14M12 5l7 7-7 7" />
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.console-view {
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.logs {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 14px 16px;
  font-family: var(--font-mono);
  font-size: 12.5px;
  background: var(--bg-base);
  line-height: 1.7;
}

.logs-empty {
  color: var(--text-faint);
  font-style: italic;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  height: 100%;
  opacity: 0.5;
}

.log-line.error {
  color: var(--color-danger);
}

.log-line.warn {
  color: var(--color-warning);
}

.log-line.debug {
  color: var(--text-faint);
  opacity: 0.7;
}

.log-line {
  color: var(--text-secondary);
  line-height: 1.65;
  white-space: pre-wrap;
  word-break: break-all;
}

.command-bar {
  display: flex;
  align-items: center;
  padding: 10px 14px;
  background: var(--bg-overlay);
  border-top: 1px solid var(--border-default);
  gap: 10px;
}

.prompt {
  color: var(--accent);
  font-family: var(--font-mono);
  font-weight: 700;
  font-size: 15px;
}

.command-bar input {
  flex: 1;
  padding: 9px 14px;
  background: var(--bg-base);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 13px;
  transition: border-color var(--transition-fast);
}

.command-bar input:focus {
  border-color: var(--accent);
}

.command-bar input:disabled {
  opacity: 0.4;
}

.send-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  background: none;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  color: var(--text-tertiary);
  cursor: pointer;
  flex-shrink: 0;
  transition:
    color var(--transition-fast),
    border-color var(--transition-fast),
    background var(--transition-fast);
}

.send-btn:hover:not(:disabled) {
  color: var(--accent);
  border-color: var(--accent);
  background: var(--accent-muted);
}

.send-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}
</style>
