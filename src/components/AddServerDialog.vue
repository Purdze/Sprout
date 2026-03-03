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

.btn.browse {
  background: var(--bg-active);
  color: var(--text-primary);
  white-space: nowrap;
}

.btn.browse:hover {
  background: var(--bg-hover);
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
</style>
