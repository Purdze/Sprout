<script setup lang="ts">
import { ref } from 'vue';
import type { Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

const props = defineProps<{
  show: boolean;
  version: string;
  releaseNotes: string;
  update: Update | null;
}>();

const emit = defineEmits<{
  'update:show': [value: boolean];
}>();

const state = ref<'idle' | 'downloading' | 'done' | 'error'>('idle');
const downloadPercent = ref(0);
const errorMessage = ref('');

async function installUpdate() {
  if (!props.update) return;
  state.value = 'downloading';
  downloadPercent.value = 0;
  errorMessage.value = '';

  try {
    let contentLength = 0;
    let downloaded = 0;
    await props.update.downloadAndInstall((event) => {
      if (event.event === 'Started') {
        contentLength = event.data.contentLength ?? 0;
        downloaded = 0;
        downloadPercent.value = 0;
      } else if (event.event === 'Progress') {
        downloaded += event.data.chunkLength;
        downloadPercent.value =
          contentLength > 0 ? Math.min(100, Math.round((downloaded / contentLength) * 100)) : 0;
      } else if (event.event === 'Finished') {
        downloadPercent.value = 100;
      }
    });
    state.value = 'done';
    await relaunch();
  } catch (e: any) {
    state.value = 'error';
    errorMessage.value = typeof e === 'string' ? e : e.message || 'Update failed';
  }
}

function close() {
  if (state.value === 'downloading') return;
  state.value = 'idle';
  downloadPercent.value = 0;
  errorMessage.value = '';
  emit('update:show', false);
}
</script>

<template>
  <div v-if="show" class="dialog-overlay" @click="close">
    <div class="dialog" @click.stop>
      <h3>Update Available</h3>

      <div class="version-info">
        <span class="version-badge">v{{ version }}</span>
      </div>

      <div v-if="releaseNotes" class="release-notes">
        <label>What's new</label>
        <div class="notes-content">{{ releaseNotes }}</div>
      </div>

      <div v-if="state === 'downloading' || state === 'done'" class="download-progress">
        <div class="progress-bar">
          <div class="progress-fill" :style="{ width: downloadPercent + '%' }"></div>
        </div>
        <span class="progress-text">
          {{ state === 'done' ? 'Installing...' : `Downloading... ${downloadPercent}%` }}
        </span>
      </div>

      <div v-if="state === 'error'" class="error-message">
        {{ errorMessage }}
      </div>

      <div class="dialog-actions">
        <button class="btn secondary" :disabled="state === 'downloading'" @click="close">
          Later
        </button>
        <button
          class="btn primary"
          :disabled="state === 'downloading' || state === 'done'"
          @click="installUpdate"
        >
          {{ state === 'error' ? 'Retry' : 'Update & Restart' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dialog {
  max-width: 480px;
}

.version-info {
  margin-bottom: 16px;
}

.version-badge {
  display: inline-block;
  padding: 3px 10px;
  background: var(--color-info);
  color: #fff;
  border-radius: 99px;
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.01em;
}

.release-notes {
  margin-bottom: 18px;
}

.release-notes label {
  display: block;
  margin-bottom: 6px;
  color: var(--text-tertiary);
  font-size: 12px;
  font-weight: 500;
}

.notes-content {
  padding: 10px 14px;
  background: var(--bg-base);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  font-size: 13px;
  line-height: 1.5;
  max-height: 160px;
  overflow-y: auto;
  white-space: pre-wrap;
}

.download-progress {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 18px;
}

.error-message {
  color: #f87171;
  font-size: 12px;
  margin-bottom: 18px;
}
</style>
