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
    <div class="update compact-dialog" @click.stop>
      <div class="top">
        <span class="title">Update available</span>
        <span class="meta">v{{ version }}</span>
      </div>

      <div v-if="releaseNotes && state === 'idle'" class="notes">{{ releaseNotes }}</div>

      <div v-if="state === 'downloading' || state === 'done'" class="progress-section">
        <div class="progress-bar">
          <div class="progress-fill" :style="{ width: downloadPercent + '%' }"></div>
        </div>
        <span class="progress-text">
          {{ state === 'done' ? 'Installing...' : `${downloadPercent}%` }}
        </span>
      </div>

      <div v-if="state === 'error'" class="error">{{ errorMessage }}</div>

      <div class="actions">
        <button class="link" :disabled="state === 'downloading'" @click="close">Later</button>
        <span class="spacer" />
        <button
          class="update-link"
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
.update {
  max-width: 400px;
}

.top {
  display: flex;
  align-items: baseline;
  gap: 8px;
  margin-bottom: 10px;
}

.title {
  font-weight: 700;
  font-size: 14px;
  color: var(--text-primary);
}

.notes {
  font-size: 12px;
  color: var(--text-tertiary);
  line-height: 1.5;
  margin-bottom: 10px;
  max-height: 80px;
  overflow-y: auto;
  white-space: pre-wrap;
}

.progress-section {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
}

.progress-bar {
  flex: 1;
  height: 4px;
  background: var(--bg-base);
  border-radius: 99px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--accent);
  border-radius: 99px;
  transition: width 0.3s ease;
}

.progress-text {
  color: var(--text-faint);
  font-size: 11px;
  font-variant-numeric: tabular-nums;
  min-width: 32px;
  text-align: right;
}

.error {
  color: #f87171;
  font-size: 12px;
  margin-bottom: 10px;
}

.link {
  color: var(--text-faint);
}

.link:hover:not(:disabled) {
  color: var(--text-secondary);
}
</style>
