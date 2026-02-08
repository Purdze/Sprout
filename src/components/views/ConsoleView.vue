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
  height: 100%;
}

.logs {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  background: #0d0d0d;
}

.logs-empty {
  color: #444;
  font-style: italic;
}

.log-line {
  color: #ccc;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-all;
}

.command-bar {
  display: flex;
  align-items: center;
  padding: 8px 12px;
  background: #252525;
  border-top: 1px solid #333;
  gap: 8px;
}

.prompt {
  color: #f97316;
  font-family: monospace;
  font-weight: bold;
}

.command-bar input {
  flex: 1;
  padding: 8px 12px;
  background: #1a1a1a;
  border: 1px solid #333;
  border-radius: 6px;
  color: #fff;
  font-family: monospace;
  font-size: 14px;
}

.command-bar input:disabled {
  opacity: 0.5;
}

.send-btn {
  padding: 8px 16px;
  background: #f97316;
  border: none;
  border-radius: 6px;
  color: #fff;
  font-size: 14px;
  cursor: pointer;
}

.send-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
