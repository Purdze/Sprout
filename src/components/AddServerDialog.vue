<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

defineProps<{
  show: boolean;
}>();

const emit = defineEmits<{
  'update:show': [value: boolean];
  add: [name: string, path: string];
}>();

const newServerName = ref('');
const newServerPath = ref('');

const platformInfo = ref('');
const downloadState = ref<'idle' | 'downloading' | 'done' | 'error'>('idle');
const downloadPercent = ref(0);
const downloadError = ref('');

let unlisten: UnlistenFn | null = null;

onMounted(async () => {
  try {
    platformInfo.value = await invoke<string>('get_platform_info');
  } catch {
    platformInfo.value = '';
  }

  unlisten = await listen<{ percent: number; downloaded: number; total: number }>(
    'download-progress',
    (event) => {
      downloadPercent.value = Math.round(event.payload.percent);
    }
  );
});

onUnmounted(() => {
  if (unlisten) unlisten();
});

async function browseServerPath() {
  const selected = await open({
    directory: true,
    multiple: false,
    title: 'Select Server Directory',
  });
  if (selected) {
    newServerPath.value = selected as string;
  }
}

async function downloadLatest() {
  if (!newServerPath.value) {
    await browseServerPath();
    if (!newServerPath.value) return;
  }

  downloadState.value = 'downloading';
  downloadPercent.value = 0;
  downloadError.value = '';

  try {
    await invoke('download_server', { path: newServerPath.value });
    downloadState.value = 'done';
  } catch (e: any) {
    downloadState.value = 'error';
    downloadError.value = typeof e === 'string' ? e : e.message || 'Download failed';
  }
}

function addServer() {
  if (!newServerName.value || !newServerPath.value) return;
  emit('add', newServerName.value, newServerPath.value);
  newServerName.value = '';
  newServerPath.value = '';
  downloadState.value = 'idle';
  downloadPercent.value = 0;
  emit('update:show', false);
}

function close() {
  downloadState.value = 'idle';
  downloadPercent.value = 0;
  downloadError.value = '';
  emit('update:show', false);
}
</script>

<template>
  <div v-if="show" class="dialog-overlay" @click="close">
    <div class="dialog" @click.stop>
      <h3>Add Server</h3>
      <div class="form-group">
        <label>Server Name</label>
        <input v-model="newServerName" placeholder="My Server" />
      </div>
      <div class="form-group">
        <label>Server Path</label>
        <div class="path-input">
          <input v-model="newServerPath" placeholder="C:\path\to\server" />
          <button class="btn browse" @click="browseServerPath">Browse</button>
        </div>
      </div>

      <div class="form-group download-section">
        <button
          v-if="downloadState === 'idle' || downloadState === 'error'"
          class="btn download"
          @click="downloadLatest"
        >
          Download Latest{{ platformInfo ? ` (${platformInfo})` : '' }}
        </button>

        <div v-if="downloadState === 'downloading'" class="download-progress">
          <div class="progress-bar">
            <div class="progress-fill" :style="{ width: downloadPercent + '%' }"></div>
          </div>
          <span class="progress-text">Downloading... {{ downloadPercent }}%</span>
        </div>

        <div v-if="downloadState === 'done'" class="download-done">Download complete</div>

        <div v-if="downloadState === 'error'" class="download-error">
          {{ downloadError }}
        </div>
      </div>

      <div class="dialog-actions">
        <button class="btn secondary" @click="close">Cancel</button>
        <button class="btn primary" @click="addServer">Add</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.65);
  backdrop-filter: blur(6px);
  display: flex;
  align-items: center;
  justify-content: center;
  animation: fade-in 150ms ease;
}

.dialog {
  background: var(--bg-overlay);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-lg);
  padding: 28px;
  min-width: 420px;
  box-shadow: var(--shadow-lg);
}

.dialog h3 {
  margin: 0 0 20px;
  color: var(--text-primary);
  font-size: 18px;
  font-weight: 700;
  letter-spacing: -0.02em;
}

.form-group {
  margin-bottom: 18px;
}

.form-group label {
  display: block;
  margin-bottom: 6px;
  color: var(--text-tertiary);
  font-size: 12px;
  font-weight: 500;
  letter-spacing: 0.01em;
}

.form-group input {
  width: 100%;
  padding: 9px 14px;
  background: var(--bg-base);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-size: 13px;
  font-family: var(--font-ui);
  transition: border-color var(--transition-fast);
}

.form-group input:focus {
  border-color: var(--accent);
}

.path-input {
  display: flex;
  gap: 8px;
}

.path-input input {
  flex: 1;
}

.btn {
  padding: 9px 18px;
  border-radius: var(--radius-sm);
  border: none;
  cursor: pointer;
  font-size: 13px;
  font-weight: 600;
  font-family: var(--font-ui);
  transition: all var(--transition-fast);
}

.btn.browse {
  background: var(--bg-active);
  color: var(--text-primary);
  white-space: nowrap;
}

.btn.browse:hover {
  background: var(--bg-hover);
}

.btn.primary {
  background: var(--accent);
  color: #fff;
}

.btn.primary:hover {
  background: var(--accent-hover);
  box-shadow: 0 0 14px var(--accent-muted);
}

.btn.secondary {
  background: var(--bg-surface);
  color: var(--text-secondary);
  border: 1px solid var(--border-default);
}

.btn.secondary:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.btn.download {
  width: 100%;
  background: #1a7d45;
  color: #fff;
  padding: 10px 18px;
}

.btn.download:hover {
  background: #1e9050;
  box-shadow: 0 0 12px rgba(26, 125, 69, 0.25);
}

.download-section {
  margin-top: 8px;
}

.download-progress {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.progress-bar {
  width: 100%;
  height: 6px;
  background: var(--bg-base);
  border-radius: 99px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--accent);
  border-radius: 99px;
  transition: width 0.3s ease;
  box-shadow: 0 0 8px var(--accent-muted);
}

.progress-text {
  color: var(--text-secondary);
  font-size: 12px;
  font-variant-numeric: tabular-nums;
}

.download-done {
  color: var(--color-success);
  font-size: 13px;
  font-weight: 500;
}

.download-error {
  color: #f87171;
  font-size: 12px;
  margin-top: 6px;
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 24px;
}
</style>
