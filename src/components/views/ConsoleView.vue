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
        No logs yet. Start the server to see output.
      </div>
      <div v-for="(log, i) in server.logs" :key="i" class="log-line">
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
        Send
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
  padding: 12px;
  font-family: var(--font-mono);
  font-size: 13px;
  background: var(--bg-darkest);
}

.logs-empty {
  color: var(--text-faint);
  font-style: italic;
}

.log-line {
  color: var(--text-secondary);
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-all;
}

.command-bar {
  display: flex;
  align-items: center;
  padding: 8px 12px;
  background: var(--bg-medium);
  border-top: 1px solid var(--bg-light);
  gap: 8px;
}

.prompt {
  color: var(--color-primary);
  font-family: var(--font-mono);
  font-weight: bold;
}

.command-bar input {
  flex: 1;
  padding: 8px 12px;
  background: var(--bg-dark);
  border: 1px solid var(--bg-light);
  border-radius: 6px;
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 14px;
}

.command-bar input:disabled {
  opacity: 0.5;
}

.send-btn {
  padding: 8px 16px;
  background: var(--color-primary);
  border: none;
  border-radius: 6px;
  color: var(--text-primary);
  font-size: 14px;
  cursor: pointer;
}

.send-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
