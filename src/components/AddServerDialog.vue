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
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
}

.dialog {
  background: #252525;
  border-radius: 12px;
  padding: 24px;
  min-width: 400px;
}

.dialog h3 {
  margin: 0 0 16px;
  color: #fff;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  margin-bottom: 4px;
  color: #888;
  font-size: 14px;
}

.form-group input {
  width: 100%;
  padding: 8px 12px;
  background: #1a1a1a;
  border: 1px solid #333;
  border-radius: 6px;
  color: #fff;
  font-size: 14px;
}

.path-input {
  display: flex;
  gap: 8px;
}

.path-input input {
  flex: 1;
}

.btn {
  padding: 8px 16px;
  border-radius: 6px;
  border: none;
  cursor: pointer;
  font-size: 14px;
}

.btn.browse {
  background: #444;
  color: #fff;
  white-space: nowrap;
}

.btn.browse:hover {
  background: #555;
}

.btn.primary {
  background: #f97316;
  color: #fff;
}

.btn.secondary {
  background: #333;
  color: #fff;
}

.btn.download {
  width: 100%;
  background: #1a6b3c;
  color: #fff;
  padding: 10px 16px;
}

.btn.download:hover {
  background: #1e7d45;
}

.download-section {
  margin-top: 8px;
}

.download-progress {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.progress-bar {
  width: 100%;
  height: 8px;
  background: #1a1a1a;
  border-radius: 4px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: #f97316;
  border-radius: 4px;
  transition: width 0.2s ease;
}

.progress-text {
  color: #888;
  font-size: 13px;
}

.download-done {
  color: #4ade80;
  font-size: 14px;
}

.download-error {
  color: #f87171;
  font-size: 13px;
  margin-top: 6px;
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 24px;
}
</style>
